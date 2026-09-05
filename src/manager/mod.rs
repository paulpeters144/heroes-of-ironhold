use crate::access::ids::{shader, AssetId};
use crate::scene::{
    ChangeSceneEvent, LoadingScene, SceneFactory, SceneFuture, SceneId, SceneState,
};
use crate::util::camera::GameCamera;
use crate::util::view_scale;
use crate::{Assets, Config, Context, DiContainer, SubCollection};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Manager {
    pub cfg: Rc<Config>,
    pub camera: Rc<GameCamera>,
    pixel_snap: Option<Material>,
    jitter_free: Option<Material>,
    scene_state: SceneState,
    pending_scene: Rc<Cell<Option<SceneId>>>,
    scene_factory: SceneFactory,
    _subs: SubCollection,
}

impl Manager {
    pub async fn new(di: Rc<DiContainer>) -> Self {
        let bus = di.event_bus();
        let camera = di.camera();
        let cfg = di.config();

        let pixel_snap = if cfg.pixel_snap {
            let params = MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                uniforms: vec![UniformDesc::new("viewport", UniformType::Float2)],
                ..Default::default()
            };
            Self::load_shader_material(
                shader::Shader::PixelSnapVert,
                shader::Shader::PixelSnapFrag,
                params,
            )
            .await
        } else {
            None
        };

        let jitter_free = Self::load_shader_material(
            shader::Shader::JitterFreeVert,
            shader::Shader::JitterFreeFrag,
            MaterialParams {
                uniforms: vec![UniformDesc::new("texture_size", UniformType::Float2)],
                ..Default::default()
            },
        )
        .await;

        let pending: Rc<Cell<Option<SceneId>>> = Rc::new(Cell::new(None));
        let pending_for_handler = pending.clone();
        let subs = SubCollection::new();
        subs.on::<ChangeSceneEvent>(&bus, move |event: &ChangeSceneEvent| {
            pending_for_handler.set(Some(event.0));
        });

        let scene_factory = SceneFactory::new(di);
        bus.fire(&ChangeSceneEvent(SceneId::BattleTest));

        let loading_scene = Box::new(LoadingScene::new(cfg.clone()));

        Manager {
            cfg,
            camera,
            pixel_snap,
            jitter_free,
            scene_state: SceneState::Idle {
                scene: loading_scene,
            },
            pending_scene: pending,
            scene_factory,
            _subs: subs,
        }
    }

    async fn load_shader_material(
        vert: shader::Shader,
        frag: shader::Shader,
        params: MaterialParams,
    ) -> Option<Material> {
        let mut assets = Assets::new();
        assets.preload(&[&vert, &frag]).await;

        let (Some(vertex), Some(fragment)) = (
            assets.shaders.get(&vert.path()),
            assets.shaders.get(&frag.path()),
        ) else {
            warn!("shader sources missing; material disabled");
            return None;
        };

        match load_material(ShaderSource::Glsl { vertex, fragment }, params) {
            Ok(material) => Some(material),
            Err(err) => {
                warn!("failed to compile shader: {}", err);
                None
            }
        }
    }
    pub fn update(&mut self, ctx: &mut Context) {
        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }

        if let Some(id) = self.pending_scene.take() {
            let factory = self.scene_factory.clone();
            let loader = SceneFuture::new(Box::pin(async move {
                let mut scene = factory.create(id);
                scene.load().await;
                scene
            }));

            if let SceneState::Idle { scene } = &mut self.scene_state {
                scene.dispose();
                self.scene_state = SceneState::Transition {
                    scene: Box::new(LoadingScene::new(self.cfg.clone())),
                    next_scene_loader: loader,
                    elapsed: 0.0,
                    min_wait: 0.3,
                };
            }
        }

        ctx.dt = get_frame_time();

        match &mut self.scene_state {
            SceneState::Idle { scene } => {
                scene.update(ctx);
            }
            SceneState::Transition {
                scene: loading_scene,
                next_scene_loader,
                elapsed,
                min_wait,
            } => {
                loading_scene.update(ctx);
                *elapsed += ctx.dt;
                next_scene_loader.poll();

                if next_scene_loader.is_done() && *elapsed >= *min_wait {
                    if let Some(new_scene) = next_scene_loader.retrieve() {
                        self.scene_state = SceneState::Idle { scene: new_scene };
                    }
                }
            }
        }
    }

    pub fn draw(&self, ctx: &Context) {
        self.begin_scene_pass(ctx);
        self.draw_scene(ctx);
        self.begin_screen_pass();

        self.blit_target();
        self.draw_ui(ctx);
    }

    fn begin_scene_pass(&self, ctx: &Context) {
        let camera = self.camera.camera_at(ctx.cam_target);
        set_camera(&camera);
        clear_background(BLACK);
    }

    fn draw_scene(&self, ctx: &Context) {
        if let Some(material) = &self.pixel_snap {
            material.set_uniform("viewport", vec2(self.cfg.rt_width(), self.cfg.rt_height()));
            gl_use_material(material);
        }
        match &self.scene_state {
            SceneState::Idle { scene } => scene.draw(ctx),
            SceneState::Transition { scene, .. } => scene.draw(ctx),
        }
        if self.pixel_snap.is_some() {
            gl_use_default_material();
        }
    }

    fn begin_screen_pass(&self) {
        set_default_camera();
        clear_background(BLACK);
    }

    fn draw_ui(&self, ctx: &Context) {
        let (scale, offset) = view_scale::view_scale(self.cfg.v_width, self.cfg.v_height);

        let cam = Camera2D {
            target: vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5),
            zoom: vec2(2.0 / self.cfg.v_width, 2.0 / self.cfg.v_height),
            offset: vec2(0.0, 0.0),
            rotation: 0.0,
            render_target: None,
            viewport: Some((
                offset.x as i32,
                offset.y as i32,
                (self.cfg.v_width * scale) as i32,
                (self.cfg.v_height * scale) as i32,
            )),
        };
        set_camera(&cam);

        match &self.scene_state {
            SceneState::Idle { scene } => scene.draw_ui(ctx),
            SceneState::Transition { scene, .. } => scene.draw_ui(ctx),
        }

        set_default_camera();
    }

    fn blit_target(&self) {
        let (scale, offset) = view_scale::view_scale(self.cfg.v_width, self.cfg.v_height);
        let src = Rect::new(
            (self.cfg.rt_width() - self.cfg.v_width) * 0.5,
            (self.cfg.rt_height() - self.cfg.v_height) * 0.5,
            self.cfg.v_width,
            self.cfg.v_height,
        );

        let dest_size = Some(vec2(self.cfg.v_width * scale, self.cfg.v_height * scale));
        if let Some(material) = &self.jitter_free {
            material.set_uniform(
                "texture_size",
                vec2(self.cfg.rt_width(), self.cfg.rt_height()),
            );
            gl_use_material(material);
        }
        draw_texture_ex(
            &self.camera.render_target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size,
                source: Some(src),
                flip_y: true,
                ..Default::default()
            },
        );
        if self.jitter_free.is_some() {
            gl_use_default_material();
        }
    }
}
