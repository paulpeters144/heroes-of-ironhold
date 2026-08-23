use super::{cmd::Cmd, core::UI};
use crate::Context;
use macroquad::prelude::*;

pub struct RectBuilder<'a> {
    pub(super) ui: &'a mut UI,
}

impl RectBuilder<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn solid(
        self,
        _ctx: &Context,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        radius: f32,
    ) {
        self.ui
            .cmds
            .push(Cmd::Solid {
                x,
                y,
                w,
                h,
                color,
                radius,
            });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn outlined(
        self,
        _ctx: &Context,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Color,
        border: Color,
        thickness: f32,
        radius: f32,
    ) {
        self.ui.cmds.push(Cmd::Outlined {
            x,
            y,
            w,
            h,
            fill,
            border,
            thickness,
            radius,
        });
    }

    pub fn centered_solid(self, _ctx: &Context, w: f32, h: f32, color: Color, radius: f32) {
        let x = (self.ui.v_width - w) * 0.5;
        let y = (self.ui.v_height - h) * 0.5;
        self.ui.cmds.push(Cmd::Solid {
            x,
            y,
            w,
            h,
            color,
            radius,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn centered_outlined(
        self,
        _ctx: &Context,
        w: f32,
        h: f32,
        fill: Color,
        border: Color,
        thickness: f32,
        radius: f32,
    ) {
        let x = (self.ui.v_width - w) * 0.5;
        let y = (self.ui.v_height - h) * 0.5;
        self.ui.cmds.push(Cmd::Outlined {
            x,
            y,
            w,
            h,
            fill,
            border,
            thickness,
            radius,
        });
    }

    pub fn default_outlined(self, _ctx: &Context, x: f32, y: f32, w: f32, h: f32) {
        let fill = self.ui.style.fill;
        let border = self.ui.style.border;
        let thickness = self.ui.style.border_width;
        let radius = self.ui.style.radius;
        self.ui.cmds.push(Cmd::Outlined {
            x,
            y,
            w,
            h,
            fill,
            border,
            thickness,
            radius,
        });
    }

    pub fn default_centered_outlined(self, _ctx: &Context, w: f32, h: f32) {
        let fill = self.ui.style.fill;
        let border = self.ui.style.border;
        let thickness = self.ui.style.border_width;
        let radius = self.ui.style.radius;
        let x = (self.ui.v_width - w) * 0.5;
        let y = (self.ui.v_height - h) * 0.5;
        self.ui.cmds.push(Cmd::Outlined {
            x,
            y,
            w,
            h,
            fill,
            border,
            thickness,
            radius,
        });
    }
}
