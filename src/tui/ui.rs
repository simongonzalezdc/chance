use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use serde_json::Value;

use super::app::{App, Popup};

// -----------------------------------------------------------------------------
// Color palette: warm, dark, expensive-terminal vibe
// -----------------------------------------------------------------------------
pub const BG: Color = Color::Rgb(14, 14, 20);
pub const SURFACE: Color = Color::Rgb(26, 26, 36);
pub const BORDER: Color = Color::Rgb(62, 62, 82);
pub const TEXT: Color = Color::Rgb(236, 236, 242);
pub const TEXT_DIM: Color = Color::Rgb(140, 140, 158);
pub const ACCENT: Color = Color::Rgb(255, 199, 102); // warm amber
pub const ACCENT_DARK: Color = Color::Rgb(28, 22, 14);
pub const CYAN: Color = Color::Rgb(125, 211, 224);
pub const GREEN: Color = Color::Rgb(158, 222, 176);
pub const MAGENTA: Color = Color::Rgb(222, 165, 222);
pub const RED: Color = Color::Rgb(242, 135, 135);

struct Theme {
    bg: Style,
    surface: Style,
    border: Style,
    text: Style,
    dim: Style,
    accent: Style,
    accent_bold: Style,
    selected: Style,
    key: Style,
    string: Style,
    number: Style,
    bool_null: Style,
    punct: Style,
    error: Style,
}

impl Theme {
    fn new() -> Self {
        Self {
            bg: Style::default().bg(BG),
            surface: Style::default().bg(SURFACE).fg(TEXT),
            border: Style::default().fg(BORDER),
            text: Style::default().fg(TEXT),
            dim: Style::default().fg(TEXT_DIM),
            accent: Style::default().fg(ACCENT),
            accent_bold: Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            selected: Style::default()
                .bg(ACCENT)
                .fg(ACCENT_DARK)
                .add_modifier(Modifier::BOLD),
            key: Style::default().fg(CYAN),
            string: Style::default().fg(GREEN),
            number: Style::default().fg(ACCENT),
            bool_null: Style::default().fg(MAGENTA),
            punct: Style::default().fg(TEXT_DIM),
            error: Style::default().fg(RED).add_modifier(Modifier::BOLD),
        }
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = Theme::new();
    frame.render_widget(Paragraph::new("").style(theme.bg), frame.area());

    if app.popup == Popup::Viz {
        crate::tui::viz::draw_viz(frame, app);
        return;
    }

    let area = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(main_chunks[1])
        .to_vec();

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(body_chunks[1])
        .to_vec();

    draw_header(frame, main_chunks[0], &theme);
    draw_methods(frame, app, body_chunks[0], &theme);
    draw_result(frame, app, right_chunks[0], &theme);
    draw_provenance(frame, app, right_chunks[1], &theme);
    draw_footer(frame, app, main_chunks[2], &theme);

    match app.popup {
        Popup::Source => draw_source_popup(frame, app, &theme),
        Popup::Seed => draw_seed_popup(frame, app, &theme),
        Popup::Viz => {}
        Popup::None => {}
    }
}

fn draw_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .style(theme.bg);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let title = Line::from(vec![
        Span::styled("chance", theme.accent_bold),
        Span::styled("   ·   ", theme.dim),
        Span::styled("multi-source randomness studio", theme.dim),
    ]);
    let paragraph = Paragraph::new(Text::from(title))
        .alignment(Alignment::Left)
        .style(theme.bg);
    frame.render_widget(paragraph, inner);
}

fn draw_methods(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(title(" Methods ", theme))
        .style(theme.surface);

    let items: Vec<ListItem> = app
        .methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = i == app.selected_method;
            let bullet = if is_selected { "▸" } else { "·" };
            let name_style = if is_selected {
                theme.selected
            } else {
                theme.accent_bold
            };
            let desc_style = if is_selected {
                theme.selected
            } else {
                theme.dim
            };
            let line = Line::from(vec![
                Span::styled(format!(" {} ", bullet), name_style),
                Span::styled(format!("{:<10}", m.name), name_style),
                Span::styled(m.description.to_string(), desc_style),
            ]);
            ListItem::new(line).style(if is_selected {
                theme.selected
            } else {
                Style::default().bg(SURFACE)
            })
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_result(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(title(" Result ", theme))
        .style(theme.surface);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = match &app.last_result {
        Some(value) => render_value(value, 0, theme),
        None => {
            let line = Line::from(vec![
                Span::styled("✦ ", theme.accent),
                Span::styled("Press ", theme.dim),
                Span::styled("Enter", theme.accent_bold),
                Span::styled(" to generate a result.", theme.dim),
            ]);
            Text::from(line)
        }
    };

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));
    frame.render_widget(paragraph, inner);
}

fn draw_provenance(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(title(" Provenance ", theme))
        .style(theme.surface);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = match &app.last_provenance {
        Some(Value::Object(map)) => render_provenance_card(map, theme),
        Some(value) => render_value(value, 0, theme),
        None => {
            let line = Line::from(vec![
                Span::styled("⊛ ", theme.accent),
                Span::styled("Provenance will appear here.", theme.dim),
            ]);
            Text::from(line)
        }
    };

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let seed_hint: Span<'static> = if app.seed.is_empty() {
        Span::styled("none", theme.dim)
    } else {
        Span::styled(app.seed.clone(), theme.accent)
    };

    let full = Line::from(vec![
        Span::styled(" source: ", theme.dim),
        Span::styled(app.current_source_name().to_string(), theme.accent),
        Span::styled(" │ seed: ", theme.dim),
        seed_hint,
        Span::styled(" │ ", theme.dim),
        Span::styled("↑/↓", theme.accent_bold),
        Span::styled(" select  ", theme.dim),
        Span::styled("Enter", theme.accent_bold),
        Span::styled(" run  ", theme.dim),
        Span::styled("s", theme.accent_bold),
        Span::styled(" source  ", theme.dim),
        Span::styled("S", theme.accent_bold),
        Span::styled(" seed  ", theme.dim),
        Span::styled("q", theme.accent_bold),
        Span::styled(" quit", theme.dim),
    ]);

    let paragraph = Paragraph::new(Text::from(full))
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);

    // If there's a status message, overlay it briefly by replacing the footer text.
    if let Some(msg) = &app.status_message {
        let is_error = msg.starts_with("error");
        let line = if is_error {
            Line::from(vec![
                Span::styled(" ✦ error: ", theme.error),
                Span::styled(msg.clone(), theme.error),
            ])
        } else {
            Line::from(vec![
                Span::styled(" ✦ ", theme.accent),
                Span::styled(msg.clone(), theme.dim),
            ])
        };
        let p = Paragraph::new(Text::from(line)).style(if is_error {
            Style::default().bg(ACCENT_DARK).fg(RED)
        } else {
            Style::default().bg(SURFACE).fg(TEXT)
        });
        frame.render_widget(p, area);
    }
}

fn draw_source_popup(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(42, 62, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.accent)
        .title(title(" Select Source ", theme))
        .style(theme.surface);

    let items: Vec<ListItem> = app
        .sources
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_selected = i == app.popup_selection;
            let bullet = if is_selected { "▸" } else { "·" };
            let style = if is_selected {
                theme.selected
            } else {
                theme.text
            };
            let line = Line::from(vec![
                Span::styled(format!(" {} ", bullet), style),
                Span::styled(s.clone(), style),
            ]);
            ListItem::new(line).style(if is_selected {
                theme.selected
            } else {
                Style::default().bg(SURFACE)
            })
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_seed_popup(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(52, 22, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.accent)
        .title(title(" Seed ", theme))
        .style(theme.surface);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if app.seed.is_empty() {
        Text::from(Span::styled("type a seed…", theme.dim))
    } else {
        Text::from(Span::styled(app.seed.clone(), theme.accent))
    };

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(theme.surface);
    let centered = centered_rect_inner(inner, 80, 50);
    frame.render_widget(paragraph, centered);

    let hint = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", theme.accent_bold),
            Span::styled(" confirm  ·  ", theme.dim),
            Span::styled("Esc", theme.accent_bold),
            Span::styled(" cancel", theme.dim),
        ]),
    ]))
    .alignment(Alignment::Center)
    .style(theme.surface);
    let hint_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(2),
        width: inner.width,
        height: 2,
    };
    frame.render_widget(hint, hint_area);
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn title(label: &str, theme: &Theme) -> ratatui::widgets::block::Title<'static> {
    ratatui::widgets::block::Title::from(Span::styled(label.to_string(), theme.accent))
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn centered_rect_inner(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}

fn indent(n: usize) -> String {
    " ".repeat(n)
}

fn is_non_empty_container(value: &Value) -> bool {
    match value {
        Value::Object(map) => !map.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        _ => false,
    }
}

fn render_value(value: &Value, depth: usize, theme: &Theme) -> Text<'static> {
    Text::from(render_value_lines(value, depth, theme))
}

fn render_value_lines(value: &Value, depth: usize, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let prefix = indent(depth);
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled("{}", theme.punct),
                ]));
                return lines;
            }
            lines.push(Line::from(vec![
                Span::raw(prefix.clone()),
                Span::styled("{", theme.punct),
            ]));
            let count = map.len();
            for (i, (k, v)) in map.iter().enumerate() {
                let comma = if i + 1 == count { "" } else { "," };
                if is_non_empty_container(v) {
                    lines.push(Line::from(vec![
                        Span::raw(format!("{}  ", prefix)),
                        Span::styled(format!("\"{}\"", k), theme.key),
                        Span::styled(": ", theme.punct),
                    ]));
                    let mut inner = render_value_lines(v, depth + 2, theme);
                    if let Some(last) = inner.last_mut() {
                        last.spans
                            .push(Span::styled(comma.to_string(), theme.punct));
                    }
                    lines.extend(inner);
                } else {
                    lines.push(scalar_member_line(k, v, &prefix, theme, comma));
                }
            }
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled("}", theme.punct),
            ]));
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled("[]", theme.punct),
                ]));
                return lines;
            }
            lines.push(Line::from(vec![
                Span::raw(prefix.clone()),
                Span::styled("[", theme.punct),
            ]));
            let count = arr.len();
            for (i, v) in arr.iter().enumerate() {
                let comma = if i + 1 == count { "" } else { "," };
                if is_non_empty_container(v) {
                    let mut inner = render_value_lines(v, depth + 2, theme);
                    if let Some(last) = inner.last_mut() {
                        last.spans
                            .push(Span::styled(comma.to_string(), theme.punct));
                    }
                    lines.extend(inner);
                } else {
                    lines.push(scalar_element_line(v, &prefix, theme, comma));
                }
            }
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled("]", theme.punct),
            ]));
        }
        scalar => {
            lines.push(Line::from(vec![
                Span::raw(prefix),
                scalar_span(scalar, theme),
            ]));
        }
    }
    lines
}

fn scalar_member_line(
    key: &str,
    value: &Value,
    prefix: &str,
    theme: &Theme,
    comma: &str,
) -> Line<'static> {
    Line::from(vec![
        Span::raw(format!("{}  ", prefix)),
        Span::styled(format!("\"{}\"", key), theme.key),
        Span::styled(": ", theme.punct),
        scalar_span(value, theme),
        Span::styled(comma.to_string(), theme.punct),
    ])
}

fn scalar_element_line(value: &Value, prefix: &str, theme: &Theme, comma: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw(format!("{}  ", prefix)),
        scalar_span(value, theme),
        Span::styled(comma.to_string(), theme.punct),
    ])
}

fn scalar_span(value: &Value, theme: &Theme) -> Span<'static> {
    match value {
        Value::String(s) => Span::styled(format!("\"{}\"", s), theme.string),
        Value::Number(n) => Span::styled(n.to_string(), theme.number),
        Value::Bool(b) => Span::styled(b.to_string(), theme.bool_null),
        Value::Null => Span::styled("null", theme.bool_null),
        Value::Array(arr) if arr.is_empty() => Span::styled("[]", theme.punct),
        Value::Object(map) if map.is_empty() => Span::styled("{}", theme.punct),
        _ => Span::raw(""),
    }
}

fn render_provenance_card(map: &serde_json::Map<String, Value>, theme: &Theme) -> Text<'static> {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let keys = [
        ("source", "source"),
        ("source_kind", "kind"),
        ("entropy_bits", "entropy"),
        ("latency_ms", "latency"),
        ("request_id", "request"),
        ("timestamp", "time"),
        ("seed", "seed"),
    ];
    for (json_key, label) in keys {
        if let Some(v) = map.get(json_key) {
            let value_span = match (json_key, v) {
                ("source_kind", Value::String(s)) => Span::styled(s.clone(), theme.accent),
                ("entropy_bits", Value::Number(n)) => {
                    Span::styled(format!("{} bits", n), theme.number)
                }
                ("latency_ms", Value::Number(n)) => Span::styled(format!("{} ms", n), theme.number),
                (_, Value::String(s)) if s.is_empty() => Span::styled("—", theme.dim),
                (_, Value::Null) => Span::styled("—", theme.dim),
                (_, Value::String(s)) => Span::styled(s.clone(), theme.text),
                (_, other) => Span::styled(other.to_string(), theme.text),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<10} ", label), theme.dim),
                value_span,
            ]));
        }
    }
    lines.push(Line::from(""));
    Text::from(lines)
}
