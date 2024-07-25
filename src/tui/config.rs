use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct UIConfig {
    /// How many Lines to keep in the scrollback buffer in-app.
    /// This is PER channel!
    pub scrollbuffer: u16,

    /// Width of the left pane containing
    /// usernames in the chat log, or log
    /// targets in the debug log.
    pub lcol_width: u16,

    /// Time zone to format timestamps for, expressed
    /// as a UTC offset.
    pub tz: Tz,
}

impl Default for UIConfig {
    fn default() -> Self {
        UIConfig {
            scrollbuffer: 1024,
            lcol_width: 12,
            tz: chrono_tz::Tz::UTC,
        }
    }
}
