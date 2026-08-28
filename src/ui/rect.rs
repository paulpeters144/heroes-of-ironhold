use super::{
    cmd::Cmd,
    core::UI,
    params::{
        CenteredOutlined, CenteredSolid, DefaultCenteredOutlined, DefaultOutlined, Outlined, Solid,
    },
};
use macroquad::prelude::*;

pub struct RectBuilder<'a> {
    pub(super) ui: &'a mut UI,
}

impl RectBuilder<'_> {
    pub fn solid(self, params: Solid<'_>) -> Rect {
        let Solid {
            x,
            y,
            w,
            h,
            color,
            radius,
            ..
        } = params;
        self.ui.cmds.push(Cmd::Solid {
            x,
            y,
            w,
            h,
            color,
            radius,
        });
        Rect::new(x, y, w, h)
    }

    pub fn outlined(self, params: Outlined<'_>) -> Rect {
        let Outlined {
            x,
            y,
            w,
            h,
            fill,
            border,
            thickness,
            radius,
            ..
        } = params;
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
        Rect::new(x, y, w, h)
    }

    pub fn centered_solid(self, params: CenteredSolid<'_>) -> Rect {
        let CenteredSolid { w, h, color, radius, .. } = params;
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
        Rect::new(x, y, w, h)
    }

    pub fn centered_outlined(self, params: CenteredOutlined<'_>) -> Rect {
        let CenteredOutlined {
            w,
            h,
            fill,
            border,
            thickness,
            radius,
            ..
        } = params;
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
        Rect::new(x, y, w, h)
    }

    pub fn default_outlined(self, params: DefaultOutlined<'_>) -> Rect {
        let DefaultOutlined { x, y, w, h, .. } = params;
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
        Rect::new(x, y, w, h)
    }

    pub fn default_centered_outlined(self, params: DefaultCenteredOutlined<'_>) -> Rect {
        let DefaultCenteredOutlined { w, h, .. } = params;
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
        Rect::new(x, y, w, h)
    }
}
