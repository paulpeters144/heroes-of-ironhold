use super::Drawable;
use macroquad::prelude::*;

const AREA_WIDTH: f32 = 140.0;

const GOLD: Color = Color::new(0.95, 0.82, 0.35, 1.0);
const AMBER: Color = Color::new(0.75, 0.55, 0.2, 1.0);
const BRIGHT_GOLD: Color = Color::new(1.0, 0.92, 0.6, 1.0);

const APPEAR_SECS: f32 = 0.5;
const FADE_OUT_SECS: f32 = 0.5;
const RUNE_ROT_SPEED: f32 = 0.4;
const RUNE_ROT_SPEED_REVERSE: f32 = -0.25;

#[derive(Clone, Debug)]
pub struct Spark {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: f32,
    pub life: f32,
}

#[derive(Clone, Debug)]
pub struct RuneShard {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: f32,
    pub rotation: f32,
    pub rot_speed: f32,
    pub life: f32,
    pub max_life: f32,
}

#[derive(Clone, Debug)]
pub struct ConsecrationData {
    pub center: Vec2,
    pub area_life: f32,
    pub duration: f32,
    pub sparks: Vec<Spark>,
    pub shards: Vec<RuneShard>,
}

#[derive(Clone, Debug)]
pub enum ProceduralEffect {
    Consecration(ConsecrationData),
}

#[derive(Clone, Debug)]
pub struct ProceduralDrawable {
    pub effect: ProceduralEffect,
    pub z_idx: f32,
    pub visible: bool,
}

fn hexagon_points(center: Vec2, radius: f32, rotation: f32) -> [Vec2; 6] {
    let mut points = [Vec2::ZERO; 6];
    for (i, point) in points.iter_mut().enumerate() {
        let angle = rotation + i as f32 * std::f32::consts::TAU / 6.0;
        *point = center + vec2(angle.cos() * radius, angle.sin() * radius * 0.65);
    }
    points
}

fn draw_hexagon(points: &[Vec2; 6], thickness: f32, color: Color) {
    for i in 0..6 {
        let a = points[i];
        let b = points[(i + 1) % 6];
        draw_line(a.x, a.y, b.x, b.y, thickness, color);
    }
}

fn draw_hexagon_filled(points: &[Vec2; 6], color: Color) {
    let center = vec2(
        points.iter().map(|p| p.x).sum::<f32>() / 6.0,
        points.iter().map(|p| p.y).sum::<f32>() / 6.0,
    );
    for i in 0..6 {
        let a = points[i];
        let b = points[(i + 1) % 6];
        draw_triangle(center, a, b, color);
    }
}

fn draw_diamond(pos: Vec2, size: f32, rotation: f32, color: Color) {
    let cos = rotation.cos();
    let sin = rotation.sin();
    let points = [
        pos + vec2(0.0, -size).rotate(vec2(cos, sin)),
        pos + vec2(size * 0.5, 0.0).rotate(vec2(cos, sin)),
        pos + vec2(0.0, size).rotate(vec2(cos, sin)),
        pos + vec2(-size * 0.5, 0.0).rotate(vec2(cos, sin)),
    ];
    draw_triangle(points[0], points[1], points[2], color);
    draw_triangle(points[0], points[2], points[3], color);
}

fn draw_rune_symbol(pos: Vec2, size: f32, rotation: f32, color: Color) {
    let cos = rotation.cos();
    let sin = rotation.sin();
    let rotate = |v: Vec2| pos + v.rotate(vec2(cos, sin));

    let s = size * 0.4;
    let p1 = rotate(vec2(-s, -s));
    let p2 = rotate(vec2(s, -s));
    let p3 = rotate(vec2(s, s));
    let p4 = rotate(vec2(-s, s));
    draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, color);
    draw_line(p2.x, p2.y, p3.x, p3.y, 1.0, color);
    draw_line(p3.x, p3.y, p4.x, p4.y, 1.0, color);
    draw_line(p4.x, p4.y, p1.x, p1.y, 1.0, color);

    let mid = rotate(vec2(0.0, 0.0));
    let top = rotate(vec2(0.0, -s * 0.6));
    let bot = rotate(vec2(0.0, s * 0.6));
    draw_line(mid.x, mid.y, top.x, top.y, 0.8, color);
    draw_line(mid.x, mid.y, bot.x, bot.y, 0.8, color);
}

impl ConsecrationData {
    fn draw_ground_sheen(&self, appear: f32, fade_out: f32) {
        let alpha = 0.15 * appear * (1.0 - fade_out);
        if alpha <= 0.0 {
            return;
        }

        let radius = AREA_WIDTH * 0.45 * appear;
        let points = hexagon_points(self.center, radius, 0.0);

        let sheen_color = Color::new(AMBER.r, AMBER.g, AMBER.b, alpha * 0.6);
        draw_hexagon_filled(&points, sheen_color);

        let inner_radius = radius * 0.7;
        let inner_points = hexagon_points(self.center, inner_radius, 0.0);
        let bright_sheen = Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, alpha * 0.3);
        draw_hexagon_filled(&inner_points, bright_sheen);
    }

    fn draw_rune_circles(&self, vis: f32) {
        if vis <= 0.0 {
            return;
        }

        let outer_radius = AREA_WIDTH * 0.48;
        let inner_radius = AREA_WIDTH * 0.35;

        let outer_rot = self.area_life * RUNE_ROT_SPEED;
        let inner_rot = self.area_life * RUNE_ROT_SPEED_REVERSE;

        let outer_points = hexagon_points(self.center, outer_radius, outer_rot);
        let inner_points = hexagon_points(self.center, inner_radius, inner_rot);

        let pulse = 0.7 + 0.3 * (self.area_life * 3.0).sin();
        let outer_alpha = vis * pulse;

        draw_hexagon(
            &outer_points,
            2.0,
            Color::new(GOLD.r, GOLD.g, GOLD.b, outer_alpha * 0.8),
        );
        draw_hexagon(
            &outer_points,
            1.0,
            Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, outer_alpha),
        );

        let inner_alpha = vis * (1.0 - pulse * 0.3);
        draw_hexagon(
            &inner_points,
            1.5,
            Color::new(AMBER.r, AMBER.g, AMBER.b, inner_alpha * 0.7),
        );

        for i in 0..6 {
            let outer_p = outer_points[i];
            let inner_p = inner_points[i];
            draw_line(
                outer_p.x,
                outer_p.y,
                inner_p.x,
                inner_p.y,
                0.8,
                Color::new(GOLD.r, GOLD.g, GOLD.b, vis * 0.4),
            );
        }

        for i in 0..6 {
            let mid = (outer_points[i] + inner_points[i]) * 0.5;
            let symbol_rot = self.area_life * 1.5 + i as f32;
            draw_rune_symbol(
                mid,
                6.0,
                symbol_rot,
                Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, vis * 0.5),
            );
        }
    }

    fn draw_burst_sparks(&self) {
        for spark in self.sparks.iter() {
            let life_fade = (spark.life / 0.6).clamp(0.0, 1.0);
            let alpha = life_fade;
            let size = spark.size * (0.5 + 0.5 * life_fade);

            draw_diamond(
                spark.pos,
                size,
                self.area_life * 5.0 + spark.size,
                Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, alpha),
            );
        }
    }

    fn draw_shards(&self, vis: f32) {
        if vis <= 0.0 {
            return;
        }

        for shard in self.shards.iter() {
            let life_fade = (shard.life / shard.max_life).clamp(0.0, 1.0);
            let alpha = life_fade * vis;

            let tone = 0.5 + 0.5 * (shard.rotation * 2.0).sin();
            let color = if tone > 0.5 {
                Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, alpha)
            } else {
                Color::new(GOLD.r, GOLD.g, GOLD.b, alpha)
            };

            draw_diamond(shard.pos, shard.size, shard.rotation, color);

            draw_circle(
                shard.pos.x,
                shard.pos.y,
                shard.size * 0.3,
                Color::new(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, alpha * 0.8),
            );
        }
    }

    fn draw(&self) {
        let appear = (self.area_life / APPEAR_SECS).min(1.0);
        let fade_out =
            ((self.area_life - (self.duration - FADE_OUT_SECS)) / FADE_OUT_SECS).clamp(0.0, 1.0);
        let vis = appear * (1.0 - fade_out);

        self.draw_ground_sheen(appear, fade_out);
        self.draw_rune_circles(vis);
        self.draw_burst_sparks();
        self.draw_shards(vis);
    }
}

impl ProceduralEffect {
    fn draw(&self) {
        match self {
            ProceduralEffect::Consecration(data) => data.draw(),
        }
    }
}

impl Drawable for ProceduralDrawable {
    fn draw(&self) {
        if !self.visible {
            return;
        }
        self.effect.draw();
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}
