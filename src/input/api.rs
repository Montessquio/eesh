use color_eyre::eyre::Result;
use irc::proto::Message;
use ratatui::widgets::ScrollDirection;

/// Software primitives for changing the
/// UI state.
pub trait Api {
    /// Gracefully exit the application at the earliest possible time.
    fn exit(&mut self);

    /// Shift the focused viewport.
    fn scroll(&mut self, direction: ScrollDirection);
    
    /// Clear the user input buffer and prime it to receive new commands.
    fn clear_input_buffer(&mut self);

    /// Send a message to a given channel.
    fn send_message<M: Into<Message>>(&mut self, server: &str, channel: &str, message: M) -> Result<()>;
}
