use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Row, StatefulWidgetRef, Table, TableState},
};
use std::{collections::VecDeque, sync::atomic::AtomicU16};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::Context;

use super::ContextualWidget;

/// Two-column fixed-width paragraph display.
pub struct LogBuffer {
    buf_limit: u16,
    tz: Tz,
    scroll: u16,
    last_frame_height: AtomicU16,
    raw: VecDeque<(DateTime<Utc>, Line<'static>, Line<'static>)>,
}

impl LogBuffer {
    pub fn new(buf_limit: u16, tz: Tz) -> Self {
        Self {
            tz,
            buf_limit,
            scroll: 0,
            last_frame_height: AtomicU16::new(0),
            raw: VecDeque::new(),
        }
    }

    pub fn push_line(
        &mut self,
        timestamp: DateTime<Utc>,
        tag: Line<'static>,
        content: Line<'static>,
    ) {
        self.raw.push_back((timestamp, tag, content));

        // If scroll is zero, do not update scroll so as to 
        // auto-follow new messages.
        // But if it's nonzero, we want to stay where the
        // "camera" is and not disrupt the user's scroll.
        if self.scroll() != 0 {
            self.inc_scroll();
        }

        // Discard any older messages we need to in order to get to within the buffer limit.
        while self.raw.len() >= self.buf_limit.into() {
            self.raw.pop_front();
        }
    }

    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    pub fn set_scroll(&mut self, val: u16) {
        self.scroll = self.clamp_scroll(val);
    }

    pub fn inc_scroll(&mut self) {
        self.scroll = self.clamp_scroll(self.scroll.saturating_add(1));
    }

    pub fn dec_scroll(&mut self) {
        self.scroll = self.clamp_scroll(self.scroll.saturating_sub(1));
    }

    fn clamp_scroll(&self, value: u16) -> u16 {
        value.clamp(
            0,
            self.count().try_into().unwrap_or(u16::MAX).saturating_add(2).saturating_sub(
                self.last_frame_height
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
        )
    }

    pub fn count(&self) -> usize {
        self.raw.len()
    }

    pub fn lines(&self) -> &VecDeque<(DateTime<Utc>, Line<'static>, Line<'static>)> {
        &self.raw
    }

    pub fn line_height(line: &Line<'static>, max_width: u16) -> usize {
        line.width()
            .checked_div(max_width as usize)
            .unwrap_or(1)
            .saturating_add(1)
    }

    pub fn rows(&self, content_width: u16) -> impl Iterator<Item = Row<'static>> + '_ {
        self.lines().iter().map(move |(timestamp, tag, content)| {
            let row_height: u16;
            Row::new::<_>([
                Text::from(format!(
                    "[{}]",
                    self.tz
                        .from_utc_datetime(&timestamp.naive_utc())
                        .format("%H:%M:%S")
                ))
                .left_aligned(),
                {
                    let mut t = Text::default();
                    t.push_line(tag.clone());
                    t.right_aligned()
                },
                {
                    let mut t = Text::default();
                    row_height = Self::line_height(content, content_width)
                        .clamp(0, u16::MAX as usize) as u16;

                    let mut acc = Line::default();
                    for span in &content.spans {
                        let mut buf = String::new();
                        for grapheme in span.content.graphemes(true) {
                            // If adding the grapheme would overflow the current line...
                            if acc.width() + buf.width() + grapheme.width() > content_width as usize
                            {
                                // Then push the line and clear the buffer for the next line!
                                acc.push_span(Span::styled(std::mem::take(&mut buf), span.style));
                                t.push_line(std::mem::take(&mut acc));
                            }
                            // In both cases, make sure to push the grapheme to the buffer!
                            buf.push_str(grapheme);
                        }

                        // Cleanup in case there is any extra in the buffer.
                        if !buf.is_empty() {
                            // Then push the line and clear the buffer for the next line!
                            acc.push_span(Span::styled(std::mem::take(&mut buf), span.style));
                        }
                    }

                    t.push_line(acc);
                    t.left_aligned()
                },
            ])
            .height(row_height)
        })
    }
}

impl ContextualWidget for LogBuffer {
    fn render_ref(&self, ctx: &Context, area: Rect, buf: &mut Buffer) {
        // Determine how many characters wide the content buffer is, in order to properly
        // apply line wrap.
        // content_width = area.width - TIMESTAMP_WIDTH - CUMULATIVE_BORDER_WIDTH - LCOL_WIDTH;
        let content_width = area.width.saturating_sub(14).saturating_sub(ctx.lcol_width);

        let content = self.rows(content_width);

        self.last_frame_height.store(area.height, std::sync::atomic::Ordering::Relaxed);

        let t = Table::new(
            content,
            vec![
                Constraint::Length(10),
                Constraint::Length(ctx.lcol_width),
                Constraint::Fill(1),
            ],
        )
        .block(
            Block::new()
                .borders(Borders::ALL ^ Borders::BOTTOM)
                .title_alignment(Alignment::Center)
                .title(format!(
                    "Rect: {:?} | Scroll: {} | LC: {} | LLH: {:?}",
                    (area.width, area.height),
                    self.scroll,
                    self.count(),
                    Self::line_height(
                        self.lines()
                            .get(self.count().saturating_sub(1))
                            .map(|i| &i.2)
                            .unwrap_or(&Line::default()),
                        content_width
                    ),
                )),
        );

        let mut t_state = TableState::default().with_offset(
            self.count()
                .saturating_sub(area.height.saturating_sub(2) as usize)
                .saturating_sub(self.scroll() as usize),
        );
        StatefulWidgetRef::render_ref(&t, area, buf, &mut t_state)
    }
}
