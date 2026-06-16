//! Per-source "how it works" visualizations.
//!
//! Press `v` in the TUI to open a full-screen visualization for the selected
//! randomness source. Each source gets a colored title + one-line explanation,
//! an animated pipeline of its stages (entropy → transform → output), and a
//! live flow of "randomness" particles in the source's accent color. ChaCha20
//! additionally renders its 4×4 state matrix with the quarter-round sweep.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line as TextLine, Span},
    widgets::{
        canvas::{self, Canvas},
        Block, BorderType, Borders, Paragraph, Wrap,
    },
    Frame,
};

use super::app::App;
use super::ui::{ACCENT, ACCENT_DARK, BG, BORDER, CYAN, GREEN, MAGENTA, RED, SURFACE, TEXT, TEXT_DIM};

/// A soft electric blue used for splitmix64.
const BLUE: Color = Color::Rgb(120, 180, 255);

#[derive(Clone, Copy)]
struct Stage {
    glyph: &'static str,
    label: &'static str,
}

#[derive(Clone, Copy)]
struct Spec {
    family: &'static str,
    blurb: &'static str,
    accent: Color,
    stages: &'static [Stage],
    show_matrix: bool,
}

// -----------------------------------------------------------------------------
// Per-source specifications
// -----------------------------------------------------------------------------

const OS: Spec = Spec {
    family: "OS kernel CSPRNG",
    blurb: "Kernel CSPRNG. The OS pools hardware & timing noise into an entropy \
            reservoir, vets it, and serves unpredictable bytes via getrandom().",
    accent: CYAN,
    stages: &[
        Stage { glyph: "⏳", label: "entropy pool" },
        Stage { glyph: "⚙", label: "getrandom()" },
        Stage { glyph: "✓", label: "CSPRNG bytes" },
    ],
    show_matrix: false,
};

const CHACHA: Spec = Spec {
    family: "ChaCha20 stream-cipher PRNG",
    blurb: "Stream cipher as PRNG. A 256-bit key + counter expand through 20 \
            add–rotate–xor rounds over a 4×4 matrix, yielding a fast keystream.",
    accent: ACCENT,
    stages: &[
        Stage { glyph: "🔑", label: "256-bit key" },
        Stage { glyph: "▦", label: "4×4 state" },
        Stage { glyph: "⟳", label: "20 ARX rounds" },
        Stage { glyph: "≈", label: "keystream" },
    ],
    show_matrix: true,
};

const XOSHIRO: Spec = Spec {
    family: "xoshiro / xoroshiro",
    blurb: "xor·shift·rotate over a handful of 64-bit words, then tempered \
            (multiply ** or add-rotate ++) into a 64-bit output.",
    accent: MAGENTA,
    stages: &[
        Stage { glyph: "▣", label: "state words" },
        Stage { glyph: "↻", label: "xor·shift·rotate" },
        Stage { glyph: "✦", label: "tempered out" },
    ],
    show_matrix: false,
};

const PCG: Spec = Spec {
    family: "Permuted Congruential Generator",
    blurb: "A 128-bit LCG (state×mult+incr) feeds an XSL-RR permutation that \
            scrambles the high bits into fast, statistically-excellent output.",
    accent: GREEN,
    stages: &[
        Stage { glyph: "▢", label: "u128 state" },
        Stage { glyph: "×", label: "LCG step" },
        Stage { glyph: "⚢", label: "permute" },
        Stage { glyph: "→", label: "output" },
    ],
    show_matrix: false,
};

const SPLITMIX: Spec = Spec {
    family: "SplitMix64",
    blurb: "A single 64-bit state advances by the golden-ratio γ, then a \
            splitmix xor·shift·multiply scatters the bits into output.",
    accent: BLUE,
    stages: &[
        Stage { glyph: "▢", label: "u64 state" },
        Stage { glyph: "+γ", label: "golden add" },
        Stage { glyph: "ǂ", label: "xor·mul mix" },
        Stage { glyph: "→", label: "output" },
    ],
    show_matrix: false,
};

const DRAND: Spec = Spec {
    family: "drand · distributed beacon",
    blurb: "A threshold of independent nodes co-signs each round with BLS; the \
            chained signatures are unbiased, publicly-verifiable randomness.",
    accent: GREEN,
    stages: &[
        Stage { glyph: "📡", label: "beacon nodes" },
        Stage { glyph: "✶", label: "BLS threshold" },
        Stage { glyph: "⛓", label: "round chain" },
        Stage { glyph: "☉", label: "public rand" },
    ],
    show_matrix: false,
};

const HARDWARE: Spec = Spec {
    family: "on-die hardware RNG",
    blurb: "Thermal/quantum noise → AES-CBC-MAC conditioner → DRBG → the \
            RDRAND / RDSEED instruction.",
    accent: RED,
    stages: &[
        Stage { glyph: "⚡", label: "noise source" },
        Stage { glyph: "🔒", label: "conditioner" },
        Stage { glyph: "🧬", label: "DRBG" },
        Stage { glyph: "→", label: "instruction" },
    ],
    show_matrix: false,
};

const MIX: Spec = Spec {
    family: "source mixer (HKDF)",
    blurb: "Multiple sources are funneled through SHA-256 and stretched by HKDF — \
            one trustworthy source taints the blend into a CSPRNG.",
    accent: ACCENT,
    stages: &[
        Stage { glyph: "A·B", label: "input sources" },
        Stage { glyph: "▽", label: "SHA-256 funnel" },
        Stage { glyph: "↯", label: "HKDF extract" },
        Stage { glyph: "☉", label: "blended" },
    ],
    show_matrix: false,
};

fn spec_for(name: &str) -> Spec {
    match name {
        "os-csprng" => OS,
        "chacha20" => CHACHA,
        "xoshiro256**" | "xoshiro256++" | "xoroshiro128**" => XOSHIRO,
        "pcg64" | "pcg64mcg" => PCG,
        "splitmix64" => SPLITMIX,
        "drand" => DRAND,
        "rdrand" | "rdseed" => HARDWARE,
        n if n.starts_with("mix") => MIX,
        _ => OS,
    }
}

// -----------------------------------------------------------------------------
// Top-level draw
// -----------------------------------------------------------------------------

pub fn draw_viz(frame: &mut Frame, app: &App) {
    let name = app.sources[app.popup_selection].as_str();
    let spec = spec_for(name);
    let accent = spec.accent;

    frame.render_widget(Paragraph::new("").style(Style::default().bg(BG)), frame.area());

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(3), // blurb
            Constraint::Length(1), // spacer
            Constraint::Min(0),    // pipeline + flow (+ matrix)
            Constraint::Length(1), // hint
        ])
        .split(area);

    let title = TextLine::from(vec![
        Span::styled(
            " ⬢ ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            name.to_string(),
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(spec.family, Style::default().fg(accent)),
    ]);
    frame.render_widget(
        Paragraph::new(title).style(Style::default().bg(BG)),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(Span::styled(spec.blurb, Style::default().fg(TEXT_DIM)))
            .style(Style::default().bg(BG))
            .wrap(Wrap { trim: true }),
        chunks[1],
    );

    // middle: stage cards, optional matrix, flow lane
    let mut middle_cons = vec![Constraint::Length(7)]; // cards
    if spec.show_matrix {
        middle_cons.push(Constraint::Length(11)); // matrix
    }
    middle_cons.push(Constraint::Min(3)); // flow
    let middle = Layout::default()
        .direction(Direction::Vertical)
        .constraints(middle_cons)
        .split(chunks[3]);

    draw_cards(frame, middle[0], &spec, accent, app.tick);
    if spec.show_matrix {
        draw_matrix(frame, middle[1], app.tick, accent);
        draw_flow(frame, middle[2], app.tick, accent);
    } else {
        draw_flow(frame, middle[1], app.tick, accent);
    }

    let hint = TextLine::from(vec![
        Span::styled(" ←/→ j/k ", Style::default().fg(CYAN)),
        Span::styled("switch source   ", Style::default().fg(TEXT_DIM)),
        Span::styled("v / Enter", Style::default().fg(CYAN)),
        Span::styled(" use source   ", Style::default().fg(TEXT_DIM)),
        Span::styled("Esc", Style::default().fg(CYAN)),
        Span::styled(" back", Style::default().fg(TEXT_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(hint)
            .alignment(Alignment::Center)
            .style(Style::default().bg(BG)),
        chunks[4],
    );
}

fn draw_cards(frame: &mut Frame, area: Rect, spec: &Spec, accent: Color, tick: u64) {
    let n = spec.stages.len();
    let cons: Vec<Constraint> = (0..n).map(|_| Constraint::Min(10)).collect();
    let cells = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(cons)
        .split(area);
    for (i, st) in spec.stages.iter().enumerate() {
        let lit = (tick as usize + i) % 2 == 0;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(accent))
            .title(TextLine::from(Span::styled(
                format!(" {} ", st.glyph),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )));
        let body = if lit {
            Style::default()
                .fg(TEXT)
                .add_modifier(Modifier::BOLD)
                .bg(SURFACE)
        } else {
            Style::default().fg(TEXT_DIM).bg(SURFACE)
        };
        let pos = TextLine::from(Span::styled(
            format!("{} / {}", i + 1, n),
            Style::default().fg(BORDER),
        ));
        let label = TextLine::from(Span::styled(st.label, body));
        let para = Paragraph::new(vec![pos, TextLine::raw(""), label])
            .alignment(Alignment::Center)
            .style(Style::default().bg(SURFACE))
            .block(block);
        frame.render_widget(para, cells[i]);
    }
}

fn draw_flow(frame: &mut Frame, area: Rect, tick: u64, accent: Color) {
    if area.width < 4 || area.height < 2 {
        return;
    }
    let w = area.width as f64;
    let h = area.height as f64;
    let mid = h / 2.0;
    let canvas = Canvas::default()
        .background_color(BG)
        .block(Block::default().style(Style::default().bg(BG)))
        .x_bounds([0.0, w])
        .y_bounds([0.0, h])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            ctx.draw(&canvas::Line {
                x1: 0.0,
                y1: mid,
                x2: w,
                y2: mid,
                color: BORDER,
            });
            let count = 18usize;
            let speed = 2.4;
            let span = w + 8.0;
            for i in 0..count {
                let base = (tick as f64 * speed) + (i as f64) * (w / count as f64);
                let x = base.rem_euclid(span) - 4.0;
                let wob = ((tick as f64 + i as f64 * 0.7) * 0.18).sin() * (mid * 0.55);
                let y = mid + wob;
                let big = i % 5 == 0;
                ctx.draw(&canvas::Rectangle {
                    x,
                    y,
                    width: if big { 4.0 } else { 3.0 },
                    height: if big { 2.0 } else { 1.2 },
                    color: if big { accent } else { dim(accent) },
                });
            }
        });
    frame.render_widget(canvas, area);
}

fn draw_matrix(frame: &mut Frame, area: Rect, tick: u64, accent: Color) {
    let words: [[&str; 4]; 4] = [
        ["61707865", "3320646e", "79622d32", "6b206574"],
        ["key0", "key1", "key2", "key3"],
        ["key4", "key5", "key6", "key7"],
        ["ctr", "ctr+1", "nonce0", "nonce1"],
    ];
    let row_lbl = ["const", "key A", "key B", "ctr/n"];
    // 8 phases: quarter-round sweeps columns 0..4, then diagonals 0..4
    let phase = (tick / 3) % 8;

    let mut lines: Vec<TextLine> = Vec::new();
    lines.push(TextLine::from(Span::styled(
        "  16 words (512-bit state) — the bright cell is the active quarter-round",
        Style::default().fg(BORDER),
    )));
    lines.push(TextLine::raw(""));

    for r in 0..4 {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            format!("{:>6}  ", row_lbl[r]),
            Style::default().fg(TEXT_DIM),
        ));
        for c in 0..4 {
            let active = if phase < 4 {
                c == phase as usize
            } else {
                ((c + r) % 4) == ((phase - 4) as usize)
            };
            let style = if active {
                Style::default()
                    .bg(accent)
                    .fg(ACCENT_DARK)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT)
            };
            spans.push(Span::styled(format!(" {:^10} ", words[r][c]), style));
            if c < 3 {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(TextLine::from(spans));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .title(TextLine::from(Span::styled(
            " ChaCha20 state · quarter-round sweep ",
            Style::default().fg(accent),
        )));
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(BG))
            .block(block),
        area,
    );
}

/// Dim a color toward the background for the "lesser" particles.
fn dim(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 3 + 8, g / 3 + 8, b / 3 + 8),
        other => other,
    }
}
