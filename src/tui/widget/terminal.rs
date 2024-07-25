use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::ContextualWidget;
use super::RenderContext;

pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Terminal {}
    }
}

impl ContextualWidget for Terminal {
    fn render_ref(&self, ctx: &RenderContext, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(2)]).split(area);

        if let Some(tb) = &ctx.text_buffer {
            tb.lock()
                .expect("Screenbuffer mutex was poisoned!")
                .with_context(ctx)
                .render(layout[0], buf);
        }

        Paragraph::new(Text::from(vec![Line::from(ctx.user_line.as_str())]))
            .left_aligned()
            .block(Block::new().borders(Borders::ALL ^ Borders::TOP))
            .render(layout[1], buf);
    }
}
