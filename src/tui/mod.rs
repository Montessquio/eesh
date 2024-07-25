use ratatui::{
    buffer::Buffer, layout::{Constraint, Layout, Rect}, widgets::Widget, Frame
};

mod config;
mod tuiwrapper;
pub mod widget;

pub use config::UIConfig;
pub use tuiwrapper::Tui;
pub use widget::{RenderContext, ContextualWidget};

pub struct StatelessView<'a> {
    ctx: &'a RenderContext,
}

// Builder impls
impl<'a> StatelessView<'a> {
    pub fn new(ctx: &'a RenderContext) -> Self {
        StatelessView { ctx }
    }
}

// Runner Impls
impl<'a> StatelessView<'a> {
    pub fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
}

impl<'a> Widget for &StatelessView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Percentage(75),
            Constraint::Fill(1),
        ])
        .split(area);

        //tui::NetList::default().render(layout[0], buf);
        widget::Terminal::new()
            .with_context(self.ctx)
            .render(layout[1], buf);
        //tui::UserList::default().render(layout[2], buf);
    }
}

/*
#[derive(Default)]
pub struct NetList<'a> {
    entries: Vec<Text<'a>>,
}

impl<'a> Widget for &NetList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)]).split(area);

        //List::new(&self.entries)
        //    .block(Block::bordered().title("List"))
        //    .style(Style::default().fg(Color::White))
        //    .highlight_style(Style::default())
        //    .repeat_highlight_symbol(true)
        //    .direction(ListDirection::BottomToTop)
        //    .render(layout[0], buf);

        Paragraph::new(Text::from(vec![Line::from(vec!["Bottom".into()])]))
            .centered()
            .block(Block::new().borders(Borders::ALL))
            .render(layout[1], buf);
    }
}

#[derive(Default)]
pub struct UserList {}

impl Widget for &UserList {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)]).split(area);

        Paragraph::new(Text::from(vec![Line::from(vec!["Top".into()])]))
            .centered()
            .block(Block::new().borders(Borders::ALL ^ Borders::BOTTOM))
            .render(layout[0], buf);

        Paragraph::new(Text::from(vec![Line::from(vec!["Bottom".into()])]))
            .centered()
            .block(Block::new().borders(Borders::ALL))
            .render(layout[1], buf);
    }
}
*/
