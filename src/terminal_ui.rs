use std::borrow::Cow;
use std::env;
use std::io::{self, IsTerminal};

use nu_ansi_term::{Color, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Plain,
    Ansi,
}

pub struct RenderContext {
    pub mode: RenderMode,
    pub width: Option<usize>,
}

pub fn default_stdout_context() -> RenderContext {
    RenderContext {
        mode: if io::stdout().is_terminal() {
            RenderMode::Ansi
        } else {
            RenderMode::Plain
        },
        width: env::var("COLUMNS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0),
    }
}

pub fn render_table(headers: &[&str], rows: &[Vec<String>], context: &RenderContext) -> String {
    let column_count = headers.len();
    if column_count == 0 {
        return String::new();
    }

    let mut widths = headers
        .iter()
        .map(|header| visible_width(header))
        .collect::<Vec<_>>();

    for row in rows {
        for (idx, cell) in row.iter().enumerate().take(column_count) {
            widths[idx] = widths[idx].max(visible_width(cell));
        }
    }

    let chars = match context.mode {
        RenderMode::Ansi => TableChars {
            top_left: '┌',
            top_join: '┬',
            top_right: '┐',
            mid_left: '├',
            mid_join: '┼',
            mid_right: '┤',
            bot_left: '└',
            bot_join: '┴',
            bot_right: '┘',
            vertical: '│',
            horizontal: '─',
        },
        RenderMode::Plain => TableChars {
            top_left: '+',
            top_join: '+',
            top_right: '+',
            mid_left: '+',
            mid_join: '+',
            mid_right: '+',
            bot_left: '+',
            bot_join: '+',
            bot_right: '+',
            vertical: '|',
            horizontal: '-',
        },
    };

    let border_style = Style::new().fg(Color::Cyan).bold();
    let header_style = Style::new().fg(Color::Cyan).bold();

    let mut output = Vec::new();
    output.push(render_table_border(
        &widths,
        chars.top_left,
        chars.top_join,
        chars.top_right,
        chars.horizontal,
        context,
        border_style,
    ));

    let header_cells = headers
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    output.push(render_table_row(
        &widths,
        &header_cells,
        chars.vertical,
        context,
        border_style,
        Some(header_style),
    ));
    output.push(render_table_border(
        &widths,
        chars.mid_left,
        chars.mid_join,
        chars.mid_right,
        chars.horizontal,
        context,
        border_style,
    ));

    for row in rows {
        output.push(render_table_row(
            &widths,
            row,
            chars.vertical,
            context,
            border_style,
            None,
        ));
    }

    output.push(render_table_border(
        &widths,
        chars.bot_left,
        chars.bot_join,
        chars.bot_right,
        chars.horizontal,
        context,
        border_style,
    ));
    output.join("\n")
}

struct TableChars {
    top_left: char,
    top_join: char,
    top_right: char,
    mid_left: char,
    mid_join: char,
    mid_right: char,
    bot_left: char,
    bot_join: char,
    bot_right: char,
    vertical: char,
    horizontal: char,
}

fn render_table_border(
    widths: &[usize],
    left: char,
    join: char,
    right: char,
    horizontal: char,
    context: &RenderContext,
    border_style: Style,
) -> String {
    let mut line = String::new();
    line.push(left);
    for (idx, width) in widths.iter().enumerate() {
        if idx > 0 {
            line.push(join);
        }
        line.push_str(&horizontal.to_string().repeat(*width + 2));
    }
    line.push(right);
    styled_border(line, context, border_style)
}

fn render_table_row(
    widths: &[usize],
    cells: &[String],
    vertical: char,
    context: &RenderContext,
    border_style: Style,
    cell_style: Option<Style>,
) -> String {
    let vertical_str = styled_border(vertical.to_string(), context, border_style);
    let mut line = String::new();
    line.push_str(&vertical_str);

    for (idx, width) in widths.iter().enumerate() {
        let cell = cells.get(idx).map(String::as_str).unwrap_or("");
        let pad = width.saturating_sub(visible_width(cell));
        let text = format!(" {cell}{} ", " ".repeat(pad));
        if let Some(style) = cell_style {
            match context.mode {
                RenderMode::Ansi => line.push_str(&style.paint(text).to_string()),
                RenderMode::Plain => line.push_str(&text),
            }
        } else {
            line.push_str(&text);
        }
        line.push_str(&vertical_str);
    }

    line
}

pub struct ActionLine<'a> {
    label: Cow<'a, str>,
    message: Cow<'a, str>,
    tone: Tone,
}

impl<'a> ActionLine<'a> {
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        message: impl Into<Cow<'a, str>>,
        tone: Tone,
    ) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
            tone,
        }
    }

    pub fn render(&self, context: &RenderContext) -> String {
        let badge_text = format!("{:<7}", self.label);
        match context.mode {
            RenderMode::Plain => format!("[{badge_text}] {}", self.message),
            RenderMode::Ansi => {
                let badge = tone_badge_style(self.tone).paint(badge_text);
                let message = tone_text_style(self.tone).paint(self.message.as_ref());
                format!("[{badge}] {message}")
            }
        }
    }
}

pub struct SummaryFooter<'a> {
    message: Cow<'a, str>,
}

impl<'a> SummaryFooter<'a> {
    pub fn new(message: impl Into<Cow<'a, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn render(&self, context: &RenderContext) -> String {
        match context.mode {
            RenderMode::Plain => format!("Summary: {}", self.message),
            RenderMode::Ansi => {
                let prefix = Style::new().fg(Color::Cyan).paint("Summary:");
                format!("{prefix} {}", self.message)
            }
        }
    }
}

pub struct Panel<'a> {
    title: Option<Cow<'a, str>>,
    tone: Option<Tone>,
    body: Cow<'a, str>,
}

impl<'a> Panel<'a> {
    pub fn new(body: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: None,
            tone: None,
            body: body.into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn render(&self, context: &RenderContext) -> String {
        let body_lines: Vec<&str> = self.body.lines().collect();
        let body_width = body_lines
            .iter()
            .map(|line| visible_width(line))
            .max()
            .unwrap_or(0);

        let title_width = self
            .title
            .as_ref()
            .map(|title| title.chars().count() + 3)
            .unwrap_or(0);

        let mut width = body_width.max(title_width).max(1);
        if let Some(total_width) = context.width {
            width = width.min(total_width.saturating_sub(4).max(1));
        }

        let (tl, tr, bl, br, h, v) = match context.mode {
            RenderMode::Ansi => ('╭', '╮', '╰', '╯', '─', '│'),
            RenderMode::Plain => ('+', '+', '+', '+', '-', '|'),
        };

        let border_style = panel_border_style(self.tone);
        let top_inner = render_top_inner(self.title.as_deref(), width + 2, h);
        let top = styled_border(format!("{tl}{top_inner}{tr}"), context, border_style);

        let mut lines = vec![top];
        if body_lines.is_empty() {
            lines.push(render_body_line("", width, v, context, border_style));
        } else {
            for line in body_lines {
                lines.push(render_body_line(line, width, v, context, border_style));
            }
        }

        let bottom = styled_border(
            format!("{bl}{}{br}", h.to_string().repeat(width + 2)),
            context,
            border_style,
        );
        lines.push(bottom);

        lines.join("\n")
    }
}

fn render_top_inner(title: Option<&str>, width: usize, h: char) -> String {
    if let Some(title) = title {
        let max_title = width.saturating_sub(3);
        let title_text = truncate_with_ellipsis(title, max_title);
        let prefix = format!("{h} {title_text} ");
        let fill = width.saturating_sub(prefix.chars().count());
        return format!("{prefix}{}", h.to_string().repeat(fill));
    }

    h.to_string().repeat(width)
}

fn render_body_line(
    line: &str,
    width: usize,
    v: char,
    context: &RenderContext,
    border_style: Style,
) -> String {
    let border = styled_border(v.to_string(), context, border_style);
    let fill = width.saturating_sub(visible_width(line));
    format!("{border} {line}{} {border}", " ".repeat(fill))
}

fn styled_border(value: String, context: &RenderContext, style: Style) -> String {
    match context.mode {
        RenderMode::Plain => value,
        RenderMode::Ansi => style.paint(value).to_string(),
    }
}

fn panel_border_style(tone: Option<Tone>) -> Style {
    match tone.unwrap_or(Tone::Info) {
        Tone::Info => Style::new().fg(Color::Blue).bold(),
        Tone::Warning => Style::new().fg(Color::Yellow).bold(),
        Tone::Error => Style::new().fg(Color::Red).bold(),
    }
}

fn tone_badge_style(tone: Tone) -> Style {
    match tone {
        Tone::Info => Style::new().on(Color::Blue).fg(Color::White).bold(),
        Tone::Warning => Style::new().on(Color::Yellow).fg(Color::Black).bold(),
        Tone::Error => Style::new().on(Color::Red).fg(Color::White).bold(),
    }
}

fn tone_text_style(tone: Tone) -> Style {
    match tone {
        Tone::Info => Style::new().fg(Color::Blue),
        Tone::Warning => Style::new().fg(Color::Yellow),
        Tone::Error => Style::new().fg(Color::Red).bold(),
    }
}

fn truncate_with_ellipsis(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let chars = value.chars().count();
    if chars <= max_width {
        return value.to_owned();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let keep = max_width - 3;
    let mut truncated = String::with_capacity(max_width);
    truncated.extend(value.chars().take(keep));
    truncated.push_str("...");
    truncated
}

fn visible_width(value: &str) -> usize {
    let mut chars = value.chars().peekable();
    let mut width = 0usize;

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for esc in chars.by_ref() {
                if esc.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        width += 1;
    }

    width
}
