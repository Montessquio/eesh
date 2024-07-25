use clap::Parser;
use client::{conf::ClientConfig, ConnectedClient, DisconnectedClient};
use color_eyre::Result;
use hashbrown::HashMap;
use input::{CommandAliases, InputHandler};
use ratatui::crossterm::event::{self, Event};
use ratatui::widgets::ScrollDirection;
use serde::Deserialize;
use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{OnceCell, RwLock};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tui::{RenderContext, StatelessView, UIConfig};

mod client;
mod input;
mod logging;
mod tui;

use tui::widget::LogBuffer;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the application configuration file.
    #[arg(short, long, default_value = "~/.eeshrc")]
    config: PathBuf,

    /// Path to the application log file.
    #[arg(short, long, default_value = "/var/log/eesh.log")]
    log_path: PathBuf,
}

#[cfg(target_os = "windows")]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the application configuration file.
    #[arg(short, long, default_value = ".\\.eeshrc")]
    config: PathBuf,

    /// Path to the application log file.
    #[arg(short, long, default_value = ".\\eesh.log")]
    log_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Command aliases such as SEND, ME, QUERY, and LEADER
    pub alias: CommandAliases,

    /// User preferences to dictate how the TUI renders.
    pub ui: UIConfig,

    /// Configurations for connecting to IRC.
    pub clients: HashMap<String, ClientConfig>,
}

impl Config {
    pub fn parse_str(raw: &str) -> Result<Config> {
        Ok(toml::from_str(raw)?)
    }

    pub fn parse(path: impl AsRef<Path>) -> Result<Config> {
        Self::parse_str(&std::fs::read_to_string(path.as_ref())?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    App::new(Config::parse(&args.config)?)
        .run(tui::Tui::acquire()?)
        .await
}

pub struct App {
    /// Configuration loaded from the user's "eeshrc" file.
    cfg: Config,

    /// Setting this flag to `true` will cause the application
    /// to gracefully exit at the end of the current frame.
    exit: AtomicBool,

    #[allow(unused)]
    clients: Vec<ConnectedClient>,

    #[allow(unused)]
    disconnected: Vec<DisconnectedClient>,

    /// This context represents the application state
    /// shared with the UI. Updates to this member
    /// should be the only thing that will mutate
    /// UI state.
    shared_context: Arc<RwLock<RenderContext>>,

    /// This represents every text-buffer
    /// for every channel currently open.
    /// Channels may not necessarily be IRC
    /// channels but may be produced by
    /// scripts or logging commands.
    logbuffers: Vec<Arc<Mutex<LogBuffer>>>,
    logbuffer_cursor: u16,

    /// This struct manages user input.
    /// See struct-level docs for more.
    input_handler: InputHandler,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        App {
            cfg: cfg.clone(),

            exit: AtomicBool::new(false),

            clients: Vec::new(),
            disconnected: Vec::new(),

            shared_context: Arc::new(RwLock::new(RenderContext::default())),
            logbuffers: vec![Arc::new(Mutex::new(LogBuffer::new(
                cfg.ui.scrollbuffer,
                cfg.ui.tz,
            )))],
            logbuffer_cursor: 0,

            input_handler: InputHandler::new(),
        }
    }

    /// Run the application's main loop until the user quits
    pub async fn run(&mut self, terminal: tui::Tui) -> Result<()> {
        tracing_subscriber::registry()
            .with(logging::LogBufferLayer::new(Arc::clone(
                &self.logbuffers[0],
            )))
            .init();
        debug!("Strike the Earth!");
        info!("Welcome to eesh, the Extra Extensible IRC Shell.");
        info!(version = built::PKG_VERSION);
        info!("");
        info!(
            "Don't know where to start? Type {0}h<enter> for help or {0}q<enter> to quit.",
            self.cfg.alias.get("leader").expect("No leader key was configured!"),
        );

        // Launch the UI thread.
        let ui_exit = {
            let shared_context = Arc::clone(&self.shared_context);
            let stop_signal: Arc<OnceCell<()>> = Arc::new(OnceCell::new());
            let thread_local_stop = Arc::clone(&stop_signal);
            tokio::spawn(async move {
                let mut t = terminal;
                let sc = shared_context;
                while !thread_local_stop.initialized() {
                    if let Err(e) = Self::render_frame(&sc, &mut t).await {
                        error!(error = e.to_string(), "UI Thread Error");
                    }
                }
                t.release().unwrap();
            });
            stop_signal
        };

        // Main thread event loop
        while !self.exit.load(Ordering::Relaxed) {
            *self.shared_context.write().await = self.create_render_context();
            self.handle_events()?;
            self.process_user_input()?;
        }

        // Setting this OnceCell terminates the UI thread.
        ui_exit.set(())?;

        Ok(())
    }

    /// Produce a snapshot of the current application state
    /// as it is relevant to the UI subsystem.
    fn create_render_context(&self) -> RenderContext {
        RenderContext {
            user_line: self.input_handler.to_string(),
            lcol_width: self.cfg.ui.lcol_width,
            text_buffer: Some(Arc::clone(&self.logbuffers[self.logbuffer_cursor as usize])),
        }
    }

    async fn render_frame(ctx: &RwLock<RenderContext>, terminal: &mut tui::Tui) -> io::Result<()> {
        let context = ctx.read().await;
        let view = StatelessView::new(&context);

        terminal.as_mut().draw(|frame| view.render_frame(frame))?;

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => self.input_handler.append(key_event),
                e => debug!(event = format!("{e:?}")),
            };
        }
        Ok(())
    }

    fn process_user_input(&mut self) -> Result<()> {
        todo!()
    }
}

impl input::Api for App {
    fn exit(&mut self) {
        self.exit.store(true, Ordering::Relaxed)
    }

    fn scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Forward => {
                self.logbuffers[self.logbuffer_cursor as usize]
                    .lock()
                    .expect("Logbuffer mutex was poisoned!")
                    .inc_scroll();
            }
            ScrollDirection::Backward => {
                self.logbuffers[self.logbuffer_cursor as usize]
                    .lock()
                    .expect("Logbuffer mutex was poisoned!")
                    .dec_scroll();
            }
        }
    }

    fn clear_input_buffer(&mut self) {
        self.input_handler.clear();
    }

    fn send_message<M: Into<irc::proto::Message>>(
        &mut self,
        server: &str,
        channel: &str,
        message: M,
    ) -> Result<()> {
        todo!()
    }
}

// Generated by build script.
pub mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));

    pub const SAMPLE_LOG: &str = r#"
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
}
