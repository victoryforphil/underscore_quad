use std::io::{self, IsTerminal, Write};

use anyhow::{bail, Context, Result};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{self, ClearType};
use crossterm::{execute, queue};
use nu_ansi_term::{Color, Style};

use crate::terminal_ui::{default_stdout_context, ActionLine, Panel, SummaryFooter, Tone};

use super::state::{PickerItem, PickerOutcome, PickerState};

const MAX_VISIBLE_ITEMS: usize = 10;

pub fn run_picker(title: &str, items: Vec<PickerItem>) -> Result<PickerOutcome> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("interactive picker requires a terminal (TTY)");
    }

    if items.is_empty() {
        bail!("no available values to pick from");
    }

    let mut state = PickerState::new(title.to_owned(), items);
    let mut stdout = io::stdout();

    terminal::enable_raw_mode().context("failed to enable raw mode for picker")?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        MoveTo(0, 0),
        Hide
    )
    .context("failed to initialize picker terminal state")?;

    let result = run_event_loop(&mut state, &mut stdout);
    terminal::disable_raw_mode().context("failed to disable raw mode after picker")?;
    let _ = execute!(stdout, Show, terminal::LeaveAlternateScreen);

    result
}

fn run_event_loop(state: &mut PickerState, stdout: &mut io::Stdout) -> Result<PickerOutcome> {
    draw_frame(state, stdout)?;

    loop {
        let ev = event::read().context("failed to read terminal event")?;

        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                return Ok(PickerOutcome::Cancelled);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let selected = state.selected_key().map(|k| k.to_owned());
                return match selected {
                    Some(key) => Ok(PickerOutcome::Selected(key)),
                    None => Ok(PickerOutcome::Cancelled),
                };
            }

            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => state.move_up(),

            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => state.move_down(),

            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => state.pop_char(),

            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            }) => state.push_char(ch),

            _ => continue,
        }

        draw_frame(state, stdout)?;
    }
}

fn draw_frame(state: &PickerState, stdout: &mut io::Stdout) -> Result<()> {
    let context = default_stdout_context();
    let frame = build_frame(state, &context);
    let lines: Vec<&str> = frame.lines().collect();

    queue!(stdout, MoveTo(0, 0), terminal::Clear(ClearType::All), Hide)?;
    for (idx, line) in lines.iter().enumerate() {
        let is_last = idx + 1 == lines.len();
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            Print(*line)
        )?;
        if !is_last {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    stdout.flush().context("failed to flush picker output")
}

fn build_frame(state: &PickerState, context: &crate::terminal_ui::RenderContext) -> String {
    let mut panel_lines = Vec::new();
    panel_lines.push(render_filter_line(state));
    panel_lines.push(render_separator());
    panel_lines.extend(render_items(state));
    panel_lines.push(String::new());
    panel_lines.push(render_help_line(state));

    let action = ActionLine::new("SELECT", state.title.as_str(), Tone::Info).render(context);
    let picker_panel = Panel::new(panel_lines.join("\n"))
        .with_title("Selection")
        .with_tone(Tone::Info)
        .render(context);

    let total = state.visible_count();
    let summary_text = if state.query.is_empty() {
        format!("{total} option(s)")
    } else {
        format!("{total}/{} matching", state.items.len())
    };
    let summary = SummaryFooter::new(summary_text).render(context);

    format!("{action}\n\n{picker_panel}\n\n{summary}")
}

fn render_filter_line(state: &PickerState) -> String {
    let prompt = Style::new().fg(Color::Cyan).bold().paint(">");
    if state.query.is_empty() {
        let placeholder = Style::new().dimmed().paint("type to filter...");
        return format!("{prompt} {placeholder}");
    }

    format!("{prompt} {}", state.query)
}

fn render_separator() -> String {
    Style::new()
        .dimmed()
        .paint("──────────────────────────────")
        .to_string()
}

fn render_items(state: &PickerState) -> Vec<String> {
    let total = state.visible_count();
    if total == 0 {
        return vec![Style::new()
            .fg(Color::Yellow)
            .paint("No matches")
            .to_string()];
    }

    let (start, end) = scroll_window(state.cursor, total, MAX_VISIBLE_ITEMS);
    let dim = Style::new().dimmed();
    let mut lines = Vec::new();

    if start > 0 {
        lines.push(dim.paint(format!("... {start} more above")).to_string());
    }

    for visible_index in start..end {
        let item_index = state.filtered_indices[visible_index];
        let label = &state.items[item_index].label;
        let is_selected = visible_index == state.cursor;
        if is_selected {
            let arrow = Style::new().fg(Color::Cyan).bold().paint("❯");
            let text = Style::new().fg(Color::White).bold().paint(label.as_str());
            lines.push(format!("{arrow} {text}"));
        } else {
            lines.push(format!("  {label}"));
        }
    }

    if end < total {
        let remaining = total - end;
        lines.push(dim.paint(format!("... {remaining} more below")).to_string());
    }

    lines
}

fn render_help_line(state: &PickerState) -> String {
    let base = "[Up/Down] navigate  [Enter] select  [Esc] cancel";
    let help = if state.visible_count() == 1 {
        format!("{base}  [Enter] confirm only match")
    } else {
        base.to_owned()
    };

    Style::new().dimmed().paint(help).to_string()
}

fn scroll_window(cursor: usize, total: usize, max_visible: usize) -> (usize, usize) {
    if total <= max_visible {
        return (0, total);
    }

    let half = max_visible / 2;
    let start = if cursor <= half {
        0
    } else if cursor + half >= total {
        total.saturating_sub(max_visible)
    } else {
        cursor - half
    };

    let end = (start + max_visible).min(total);
    (start, end)
}
