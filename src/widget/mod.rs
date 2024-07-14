use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
pub use terminal::Terminal;
pub use logbuffer::LogBuffer;

use crate::Context;

mod logbuffer;
mod terminal;

pub struct ContextualRender<'a, T> where T: ContextualWidget {
    lb: &'a T,
    ctx: &'a Context,
}

impl<'a, T> Widget for ContextualRender<'a, T> where T: ContextualWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        self.lb.render_ref(self.ctx, area, buf)
    }
}

pub trait ContextualWidget: Sized {
    fn with_context<'a>(&'a self, ctx: &'a Context) -> ContextualRender<'a, Self> {
        ContextualRender { lb: self, ctx }
    }

    fn render_ref<'a>(&'a self, ctx: &'a Context, area: Rect, buf: &mut Buffer);
}