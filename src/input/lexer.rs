use std::iter::Peekable;

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::CommandAliases;

pub enum MotionToken {
    /// Corresponds to the leader-key, default ","
    ClientCommand,
    /// Corresponds to the commander-key, default "/"
    ServerCommand,
    /// Corresponds to the enter-key
    Submit,
    /// A string surrounded in double-quotes which
    /// may have any unicode grapheme inside it.
    StringLiteral(String),
    /// Numeric literal which may be negative, expressed
    /// as a base-10 integer.
    Number(i64),
    /// Any string of non-whitespace characters that does NOT
    /// start with [\-0-9"]
    Identifier(String),
    /// Any non-printable string of key events such as "^A^D" or "Shift+PageUp".
    Chord(KeyEvent),
}

pub struct MotionTokenizer<'a, I: Iterator<Item = &'a KeyEvent>> {
    input: Peekable<I>,
    aliases: &'a CommandAliases,
}

impl<'a, I> Iterator for MotionTokenizer<'a, I>
where
    I: Iterator<Item = &'a KeyEvent>,
{
    type Item = MotionToken;

    fn next(&mut self) -> Option<Self::Item> {
        'mainloop: loop {
            return match self.input.next() {
                None => None,
                Some(ke) => match ke.code {
                    KeyCode::Enter => Some(MotionToken::Submit),
                    KeyCode::Char(c) => match c {
                        // Parse string literal
                        '"' => {
                            todo!()
                        }
                        // Parse numeric literal
                        '-' | '0'..='9' => {
                            todo!()
                        }
                        // This level of evaluation we're just
                        // trying to determine what the next token
                        // kind is, so whitespace isn't significant.
                        c if c.is_whitespace() => continue 'mainloop,
                        // Parse Identifier
                        c => {
                            todo!()
                        }
                    },
                    _ => Some(MotionToken::Chord(*ke)),
                },
            };
        }
    }
}
