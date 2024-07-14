use color_eyre::{config::HookBuilder, eyre};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::{
    io::{self, stdout, Stdout},
    panic,
};

pub type Tui = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    ratatui::Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        self::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            self::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

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
