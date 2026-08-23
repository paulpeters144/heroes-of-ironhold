use crate::Context;
use std::future::Future;
use std::pin::Pin;

pub trait Scene: 'static {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn update(&mut self, ctx: &mut Context);
    fn draw(&self, ctx: &Context);
    fn draw_ui(&self, _ctx: &Context) {}
    fn dispose(&mut self) {}
}
