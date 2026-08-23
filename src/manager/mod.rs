use crate::scene::{
    ChangeSceneEvent, LoadingScene, SceneFactory, SceneFuture, SceneId, SceneState,
};
use crate::systems::SystemAgg;
use crate::util::camera::GameCamera;
use crate::{Assets, Config, Context, EStore, EventBus, SubCollection};
use di_container::Container;
use macroquad::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Manager {
    pub cfg: &'static Config,
    pub assets: &'static Assets,
    pub camera: &'static GameCamera,
    scene_state: SceneState,
    pending_scene: Rc<Cell<Option<SceneId>>>,
    scene_factory: SceneFactory,
    _subs: SubCollection,
}

impl Manager {
    pub fn new(container: &'static Container) -> Self {
        let bus = container.get::<EventBus>().unwrap();
        let camera = container.get::<GameCamera>().unwrap();

        let pending: Rc<Cell<Option<SceneId>>> = Rc::new(Cell::new(None));
        let pending_for_handler = pending.clone();
        let subs = SubCollection::new();
        subs.on::<ChangeSceneEvent>(bus, move |event: &ChangeSceneEvent| {
            pending_for_handler.set(Some(event.0));
        });

        let cfg = container.get::<Config>().unwrap();
        let bus = container.get::<EventBus>().unwrap();
        let assets = container.get::<Assets>().unwrap();
        let estore = container.get::<EStore>().unwrap();
        let system_agg = container.get::<SystemAgg>().unwrap();

        let scene_factory = SceneFactory::new(bus, cfg, assets, estore, system_agg);
        bus.fire(&ChangeSceneEvent(SceneId::Opening));

        Manager {
            cfg,
            assets,
            camera,
            scene_state: SceneState::Idle {
                scene: Box::new(LoadingScene::new(cfg, assets)),
            },
            pending_scene: pending,
            scene_factory,
            _subs: subs,
        }
    }
    pub fn update(&mut self, ctx: &mut Context) {
        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }

        if let Some(id) = self.pending_scene.take() {
            let factory = self.scene_factory;
            let loader = SceneFuture::new(Box::pin(async move {
                let mut scene = factory.create(id);
                scene.load().await;
                scene
            }));

            if let SceneState::Idle { scene } = &mut self.scene_state {
                scene.dispose();
                self.scene_state = SceneState::Transition {
                    scene: Box::new(LoadingScene::new(self.cfg, self.assets)),
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
        self.begin_scene_pass();
        self.draw_scene(ctx);
        self.begin_screen_pass();

        self.blit_target(ctx);
        self.draw_ui(ctx);
    }

    fn begin_scene_pass(&self) {
        set_camera(&self.camera.camera);
        clear_background(BLACK);
    }

    fn draw_scene(&self, ctx: &Context) {
        // gl_use_material(&self.pixel_snap.material);
        match &self.scene_state {
            SceneState::Idle { scene } => scene.draw(ctx),
            SceneState::Transition { scene, .. } => scene.draw(ctx),
        }
        // gl_use_default_material();
    }

    fn begin_screen_pass(&self) {
        set_default_camera();
        clear_background(BLACK);
    }

    fn draw_ui(&self, ctx: &Context) {
        let scale = f32::min(
            screen_width() / self.cfg.v_width,
            screen_height() / self.cfg.v_height,
        );

        let offset = vec2(
            (screen_width() - self.cfg.v_width * scale) * 0.5,
            (screen_height() - self.cfg.v_height * scale) * 0.5,
        );

        let mut cam =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.cfg.v_width, self.cfg.v_height));
        cam.viewport = Some((
            offset.x as i32,
            offset.y as i32,
            (self.cfg.v_width * scale) as i32,
            (self.cfg.v_height * scale) as i32,
        ));
        set_camera(&cam);

        match &self.scene_state {
            SceneState::Idle { scene } => scene.draw_ui(ctx),
            SceneState::Transition { scene, .. } => scene.draw_ui(ctx),
        }

        set_default_camera();
    }

    fn blit_target(&self, ctx: &Context) {
        let scale = f32::min(
            screen_width() / self.cfg.v_width,
            screen_height() / self.cfg.v_height,
        );

        let offset = vec2(
            (screen_width() - self.cfg.v_width * scale) * 0.5,
            (screen_height() - self.cfg.v_height * scale) * 0.5,
        );
        let view = vec2(
            self.cfg.v_width / ctx.cam_zoom,
            self.cfg.v_height / ctx.cam_zoom,
        );
        let src = Rect::new(
            self.cfg.rt_width() * 0.5 + ctx.cam_pan.x - view.x * 0.5,
            self.cfg.rt_height() * 0.5 - ctx.cam_pan.y - view.y * 0.5,
            view.x,
            view.y,
        );

        let dest_size = Some(vec2(self.cfg.v_width * scale, self.cfg.v_height * scale));
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
    }
}
