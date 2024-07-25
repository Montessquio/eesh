use color_eyre::{
    config::HookBuilder,
    eyre::{bail, Result},
};
use lazy_static::lazy_static;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
};
use std::{
    io::{stdout, Stdout},
    panic,
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
};

lazy_static! {
    static ref TERMINAL_ACQUIRED: AtomicBool = AtomicBool::new(false);
}

pub struct Tui {
    term: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// RAII initializer to get a handle to the immediate
    /// mode TUI to use with ratatui.
    /// Will return an error if the terminal has already
    /// been acquired.
    pub fn acquire() -> Result<Tui> {
        if TERMINAL_ACQUIRED.load(Ordering::SeqCst) {
            bail!("The terminal has already been aquired!");
        }

        Self::install_hooks()?;
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        let term = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;

        unsafe {
            Self::set_acquired(true);
        }

        Ok(Tui { term })
    }

    /// Public wrapper for this function that
    /// allows API consumers to safely release
    /// the Terminal to reuse it for other
    /// tasks.
    pub fn release(self) -> Result<()> {
        unsafe {
            Self::restore()?;
            Self::set_acquired(false);
        }

        Ok(())
    }

    /// Performs the same tasks as `Tui::release()` without
    /// modifying the resource lock state. This is marked
    /// unsafe because in order to maintain proper state the
    /// caller is also responsible for calling `Tui::set_acquired(false)`.
    pub unsafe fn restore() -> Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn is_acquired() -> bool {
        TERMINAL_ACQUIRED.load(Ordering::SeqCst)
    }

    pub unsafe fn set_acquired(state: bool) {
        TERMINAL_ACQUIRED.store(state, Ordering::SeqCst);
    }

    /// This replaces the standard color_eyre panic and error hooks with hooks that
    /// restore the terminal before printing the panic or error.
    fn install_hooks() -> color_eyre::Result<()> {
        let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

        // convert from a color_eyre PanicHook to a standard panic hook
        let panic_hook = panic_hook.into_panic_hook();
        panic::set_hook(Box::new(move |panic_info| {
            unsafe {
                Self::restore().unwrap();
                Self::set_acquired(false);
            }
            panic_hook(panic_info);
        }));

        // convert from a color_eyre EyreHook to a eyre ErrorHook
        let eyre_hook = eyre_hook.into_eyre_hook();
        color_eyre::eyre::set_hook(Box::new(
            move |error: &(dyn std::error::Error + 'static)| {
                unsafe {
                    Self::restore().unwrap();
                    Self::set_acquired(false);
                }
                eyre_hook(error)
            },
        ))?;

        Ok(())
    }
}

impl AsRef<ratatui::Terminal<CrosstermBackend<Stdout>>> for Tui {
    fn as_ref(&self) -> &ratatui::Terminal<CrosstermBackend<Stdout>> {
        &self.term
    }
}

impl AsMut<ratatui::Terminal<CrosstermBackend<Stdout>>> for Tui {
    fn as_mut(&mut self) -> &mut ratatui::Terminal<CrosstermBackend<Stdout>> {
        &mut self.term
    }
}
