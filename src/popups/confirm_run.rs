use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::app::{Cmd, Ctx};
use crate::profile::Task;
use crate::ui::{accent, deleted, hint_line, secondary, with_footer};

pub struct ConfirmRun {
    batch: Vec<Task>,
}

impl ConfirmRun {
    pub fn new(batch: Vec<Task>) -> Self {
        Self { batch }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, cx: &Ctx) {
        let n = self.batch.len();
        let dels = self.batch.iter().filter(|t| t.flags.delete).count();

        let max_rows = (area.height as usize).saturating_sub(8).clamp(3, n.max(3));

        let mut lines = vec![Line::from("")];
        for t in self.batch.iter().take(max_rows) {
            let mut spans = vec![" • ".fg(secondary()), t.id.clone().fg(Color::Reset)];
            if t.flags.delete {
                spans.push("  --delete".fg(deleted()).bold());
            }
            lines.push(Line::from(spans));
        }
        if n > max_rows {
            lines.push(Line::from(format!("   …and {} more", n - max_rows).dim()));
        }
        if dels > 0 {
            let warning = if n == 1 {
                "⚠ this uses --delete — destination files will be removed.".to_string()
            } else {
                format!("⚠ {dels} of these use --delete — destination files will be removed.")
            };
            lines.push(Line::from(""));
            lines.push(Line::from(warning.fg(deleted())));
        }

        let run_label = if n == 1 { "Run" } else { "Run all" };
        let footer = if cx.settings.hints {
            vec![hint_line(&[("<enter>", run_label), ("<esc>", "Cancel")])]
        } else {
            vec![]
        };
        let box_h = lines.len() as u16 + 2;
        let area = with_footer(frame, cx, area, 64, box_h, footer);
        frame.render_widget(
            Paragraph::new(Text::from(lines)).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(accent()).bold())
                    .title(
                        format!(" Run {n} task{} ", if n == 1 { "" } else { "s" })
                            .fg(accent())
                            .bold(),
                    ),
            ),
            area,
        );
    }

    pub fn on_key(&mut self, key: KeyEvent, _cx: &mut Ctx) -> Cmd {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => Cmd::StartRun(std::mem::take(&mut self.batch)),
            KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => Cmd::Close,
            _ => Cmd::None,
        }
    }
}
