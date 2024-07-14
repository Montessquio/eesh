use chrono_tz::Tz;
use client::ClientBridge;
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
    Frame,
};
use std::{
    io,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Duration,
};

mod client;
mod tui;
mod widget;

use widget::{ContextualWidget, LogBuffer};

lazy_static::lazy_static! {
    pub static ref GLOBAL_TICK: AtomicUsize = AtomicUsize::new(0);
}

pub struct Config {
    /// How many Lines to keep in the scrollback buffer in-app.
    /// This is PER channel!
    scrollbuffer: u16,

    /// Width of the left pane containing
    /// usernames in the chat log, or log
    /// targets in the debug log.
    lcol_width: u16,

    /// Time zone to format timestamps for, expressed
    /// as a UTC offset.
    tz: Tz,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scrollbuffer: 1024,
            lcol_width: 12,
            tz: chrono_tz::Tz::UTC,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tui::install_hooks()?;
    let mut terminal = tui::init()?;
    let r = App::new(Config::default()).run(&mut terminal);
    tui::restore()?;
    r.map_err(|e| e.into())
}

#[derive(Default)]
pub struct App {
    cfg: Config,

    /* Program Logic */
    exit: bool,
    #[allow(dead_code)]
    clients: Vec<ClientBridge>,

    /* UI State */
    logbuffer: Vec<Arc<Mutex<LogBuffer>>>,
    logbuffer_cursor: u16,

    user_lines: [String; 10],
    user_line_cursor: u8,
    user_line_input_offset: u16,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        App {
            logbuffer: vec![Arc::new(Mutex::new(LogBuffer::new(
                cfg.scrollbuffer,
                cfg.tz,
            )))],
            cfg,
            ..Default::default()
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let asv = Arc::clone(&self.logbuffer[self.logbuffer_cursor as usize]);
        let _test_thread = std::thread::spawn(move || {
            for line in SAMPLE_LOG.lines() {
                match line.split_once('>') {
                    Some((l, r)) => asv.lock().unwrap().push_line(
                        chrono::Utc::now(),
                        format!("{}>", l).into(),
                        r[1..r.len()].into(),
                    ),
                    None => {
                        if line.is_empty() {
                            continue;
                        } else if line.starts_with('*') {
                            asv.lock().unwrap().push_line(
                                chrono::Utc::now(),
                                "*".into(),
                                (line[2..line.len()]).into(),
                            )
                        } else {
                            asv.lock().unwrap().push_line(
                                chrono::Utc::now(),
                                "".into(),
                                line.into(),
                            )
                        }
                    }
                };
                //std::thread::sleep(Duration::from_millis(1000));
            }
        });

        while !self.exit {
            crate::GLOBAL_TICK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Impart application state into the current frame
            let ctx = Context {
                user_line: self.user_lines[self.user_line_cursor as usize].clone(),
                user_line_input_offset: self.user_line_input_offset,
                lcol_width: self.cfg.lcol_width,
                text_buffer: Arc::clone(&self.logbuffer[self.logbuffer_cursor as usize]),
            };
            let view = StatelessView::new(&ctx);

            // Render the application state into a single frame
            terminal.draw(|frame| view.render_frame(frame))?;

            // Read user input and update application state

            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => panic!("In'tentional Crash!"),
            KeyCode::PageUp => {
                self.logbuffer[self.logbuffer_cursor as usize]
                    .lock()
                    .expect("Logbuffer mutex was poisoned!")
                    .inc_scroll();
            }
            KeyCode::PageDown => {
                self.logbuffer[self.logbuffer_cursor as usize]
                    .lock()
                    .expect("Logbuffer mutex was poisoned!")
                    .dec_scroll();
            }
            _ => {}
        };

        self.user_lines[0] = format!("{:?}", key_event);
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub struct Context {
    pub user_line: String,
    pub user_line_input_offset: u16,
    pub lcol_width: u16,

    pub text_buffer: Arc<Mutex<LogBuffer>>,
}

pub struct StatelessView<'a> {
    ctx: &'a Context,
}

// Builder impls
impl<'a> StatelessView<'a> {
    pub fn new(ctx: &'a Context) -> Self {
        StatelessView { ctx }
    }
}

// Runner Impls
impl<'a> StatelessView<'a> {
    fn render_frame(&self, frame: &mut Frame) {
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

        tui::NetList::default().render(layout[0], buf);
        widget::Terminal::new()
            .with_context(self.ctx)
            .render(layout[1], buf);
        tui::UserList::default().render(layout[2], buf);
    }
}

const SAMPLE_LOG: &str = r#"
<Cthon98> hey, if you type in your pw, it will show as stars
<Cthon98> ********* see!
<AzureDiamond> hunter2
<AzureDiamond> doesnt look like stars to me
<Cthon98> <AzureDiamond> *******
<Cthon98> thats what I see
<AzureDiamond> oh, really?
<Cthon98> Absolutely
<AzureDiamond> you can go hunter2 my hunter2-ing hunter2
<AzureDiamond> haha, does that look funny to you?
<Cthon98> lol, yes. See, when YOU type hunter2, it shows to us as *******
<AzureDiamond> thats neat, I didnt know IRC did that
<Cthon98> yep, no matter how many times you type hunter2, it will show to us as *******
<AzureDiamond> awesome!
<AzureDiamond> wait, how do you know my pw?
<Cthon98> er, I just copy pasted YOUR ******'s and it appears to YOU as hunter2 cause its your pw
<AzureDiamond> oh, ok.
<Donut[AFK]> HEY EURAKARTE
<Donut[AFK]> INSULT
<Eurakarte> RETORT
<Donut[AFK]> COUNTER-RETORT
<Eurakarte> QUESTIONING OF SEXUAL PREFERENCE
<Donut[AFK]> SUGGESTION TO SHUT THE FUCK UP
<Eurakarte> NOTATION THAT YOU CREATE A VACUUM
<Donut[AFK]> RIPOSTE
<Donut[AFK]> ADDON RIPOSTE
<Eurakarte> COUNTER-RIPOSTE
<Donut[AFK]> COUNTER-COUNTER RIPOSTE
<Eurakarte> NONSENSICAL STATEMENT INVOLVING PLANKTON
<Miles_Prower> RESPONSE TO RANDOM STATEMENT AND THREAT TO BAN OPPOSING SIDES
<Eurakarte> WORDS OF PRAISE FOR FISHFOOD
<Miles_Prower> ACKNOWLEDGEMENT AND ACCEPTENCE OF TERMS
<t0rbad> so there i was in this hallway right
<BlackAdder> i believe i speak for all of us when i say...
<BlackAdder> WRONG BTICH
<BlackAdder> IM SICK OF YOU
<BlackAdder> AND YOUR LAME STORIES
<BlackAdder> NOBODY  HERE THINKS YOURE FUNNY
<BlackAdder> NOBODY HERE WANTS TO HEAR YOUR STORIES
<BlackAdder> IN FACT
<BlackAdder> IF YOU DIED RIGHT NOW
<BlackAdder> I  DON"T THINK NOBODY WOULD CARE
<BlackAdder> SO WHAT DO YOU SAY TO THAT FAG
* t0rbad sets mode: +b BlackAdder*!*@*.*
* BlackAdder has been kicked my t0rbad ( )
<t0rbad> so there i was in this hallway right
<CRCError> right
<heartless> Right.
<Zybl0re> get up
<Zybl0re> get on up
<Zybl0re> get up
<Zybl0re> get on up
<phxl|paper> and DANCE
* nmp3bot dances :D-<
* nmp3bot dances :D|-<
* nmp3bot dances :D/-<
<[SA]HatfulOfHollow> i'm going to become rich and famous after i invent a device that allows you to stab people in the face over the internet
<Guo_Si> Hey, you know what sucks?
<TheXPhial> vaccuums
<Guo_Si> Hey, you know what sucks in a metaphorical sense?
<TheXPhial> black holes
<Guo_Si> Hey, you know what just isn't cool?
<TheXPhial> lava?
"#;
