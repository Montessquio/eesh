use hashbrown::HashMap;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt::{Display, Write};

mod api;
mod lexer;
pub use api::Api;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CommandAliases(HashMap<String, String>);

impl CommandAliases {
    /// Returns the user-configured alias value if there is one,
    /// or the hard-coded default value if there is one for the
    /// given key, or else None.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        let key = key.as_ref().to_lowercase();
        self.0.get(&key).map(|x| x.as_str()).or(match key.as_str() {
            "leader" => Some(","),
            "commander" => Some("/"),
            _ => None,
        })
    }
}

/// This struct is responsible for
/// converting keypresses into application
/// commands which in turn change the state
/// of the application.
pub struct InputHandler {
    motion: Vec<KeyEvent>,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler { motion: Vec::new() }
    }

    /// Resets the motion recording to EMPTY.
    pub fn clear(&mut self) {
        self.motion.clear();
    }

    /// Push a new key event to the stream.
    /// Usually followed by a call to InputHandler::evaluate.
    pub fn append(&mut self, ev: KeyEvent) {
        match ev.code {
            KeyCode::Esc => self.motion.clear(),
            KeyCode::Backspace => {
                self.motion.pop();
            }
            _ => self.motion.push(ev),
        };
    }

    /// Parse the current input buffer and execute any changes
    /// to the app state it defines.
    fn evaluate(&mut self, api: &mut impl api::Api) {
        todo!()
    }
}

impl Display for InputHandler {
    /// Display the user-line, the current input
    /// buffer as a string of text recognizable
    /// to the user.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ke in &self.motion {
            if let KeyCode::Char(c) = &ke.code {
                if ke.modifiers.contains(KeyModifiers::CONTROL) {
                    f.write_char('^')?;
                }
                f.write_char(*c)?;
            };
        }

        Ok(())
    }
}

/*
impl Motion {
    fn evaluate(&self) -> Result<Option<AppEvent>> {
        match self {
            Motion::Empty => Ok(None),
            Motion::Dynamic(v) => Self::eval_dyn(v),
            Motion::Compiled(v) => Self::eval_compile(v),
        }
    }      if v.iter().any(|ke| ke.code == KeyCode::Char('c')) {
            return Ok(Some(AppEvent::Quit));
        }
        Ok(None)
    }

    fn eval_compile(v: &[KeyEvent]) -> Result<Option<AppEvent>> {
        match v.last().map(|ke| ke.code) {
            Some(KeyCode::Enter) => {
                // If the buffer is ,q<enter> then force a quit state.
                // In this case, the q is the 0th character because the
                // leader character is implicit in the motion enum variant.
                let q_char = v.first().and_then(|ke| {
                    if let KeyCode::Char(c) = ke.code {
                        Some(c)
                    } else {
                        None
                    }
                });

                match q_char {
                    Some('q') if v.len() == 2 => Ok(Some(AppEvent::Quit)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}
*/
