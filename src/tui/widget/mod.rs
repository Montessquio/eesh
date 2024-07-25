use std::sync::{Arc, Mutex};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
pub use terminal::Terminal;
pub use logbuffer::LogBuffer;

mod logbuffer;
mod terminal;

#[derive(Default)]
pub struct RenderContext {
    pub user_line: String,
    pub lcol_width: u16,

    pub text_buffer: Option<Arc<Mutex<LogBuffer>>>,
}

pub struct ContextualRender<'a, T> where T: ContextualWidget {
    lb: &'a T,
    ctx: &'a RenderContext,
}

impl<'a, T> Widget for ContextualRender<'a, T> where T: ContextualWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        self.lb.render_ref(self.ctx, area, buf)
    }
}

pub trait ContextualWidget: Sized {
    fn with_context<'a>(&'a self, ctx: &'a RenderContext) -> ContextualRender<'a, Self> {
        ContextualRender { lb: self, ctx }
    }

    fn render_ref<'a>(&'a self, ctx: &'a RenderContext, area: Rect, buf: &mut Buffer);
}