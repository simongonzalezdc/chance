use crate::core::range::uniform_entropy_bits;
use crate::core::source::Source;
use crate::core::SourceHealth;
use crate::methods::*;
use crate::sources::create_source;
use clap::{Parser, Subcommand};
use serde_json;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "chance")]
#[command(about = "A fancy, fun, beautiful multi-source random number generator.")]
#[command(version)]
pub struct Cli {
    /// Randomness source to use.
    #[arg(short, long, global = true, default_value = "os-csprng")]
    pub source: String,

    /// Seed for deterministic sources (hex, decimal, or string).
    #[arg(long, global = true)]
    pub seed: Option<String>,

    /// Output as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose output with roll details.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Roll dice using standard RPG notation.
    Roll {
        /// Dice notation, e.g. d20, 4d6kh3, 2d20adv.
        notation: String,
    },

    /// Flip a coin.
    Flip {
        /// Number of flips.
        #[arg(short, long, default_value = "1")]
        times: u64,

        /// Show heads/tails counts.
        #[arg(long)]
        histogram: bool,
    },

    /// Draw cards from a shuffled deck.
    Draw {
        /// Number of cards to draw.
        #[arg(short, long, default_value = "5")]
        count: usize,
    },

    /// Pick one or more winners from a list.
    Pick {
        /// Items to choose from.
        items: Vec<String>,

        /// Number of distinct winners.
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Shuffle a list of items.
    Shuffle {
        /// Items to shuffle.
        items: Vec<String>,
    },

    /// Generate a random integer.
    Int {
        /// Minimum value (inclusive).
        #[arg(short, long, default_value = "1")]
        min: i64,

        /// Maximum value (inclusive).
        #[arg(short = 'M', long, default_value = "100")]
        max: i64,
    },

    /// Generate random bytes.
    Bytes {
        /// Number of bytes.
        #[arg(short, long, default_value = "16")]
        count: usize,

        /// Output as hex.
        #[arg(long)]
        hex: bool,

        /// Output as base64.
        #[arg(long)]
        base64: bool,
    },

    /// Generate a UUID.
    Uuid {
        /// UUID version (4 or 7).
        #[arg(short, long, default_value = "4")]
        version: u8,
    },

    /// Generate a password.
    Password {
        /// Password length.
        #[arg(short, long, default_value = "16")]
        length: usize,

        /// Exclude symbols.
        #[arg(long)]
        no_symbols: bool,
    },

    /// Draw Elder Futhark runes.
    Runes {
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Cast an I Ching hexagram.
    Iching {
        #[arg(short, long, default_value = "coin")]
        method: String,
    },

    /// Draw Tarot cards.
    Tarot {
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Draw dominoes from a double-n set.
    Dominoes {
        #[arg(short, long, default_value = "6")]
        set: u8,
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Spin a roulette wheel.
    Roulette {
        #[arg(short, long, default_value = "european")]
        variant: String,
    },

    /// Draw lottery numbers.
    Lottery {
        #[arg(short, long, default_value = "49")]
        pool: u8,
        #[arg(short, long, default_value = "6")]
        pick: usize,
        #[arg(long)]
        bonus_pool: Option<u8>,
    },

    /// Cast knucklebones / astragali.
    Knucklebones {
        #[arg(short, long, default_value = "4")]
        count: usize,
    },

    /// Spin a teetotum / dreidel.
    Teetotum {
        #[arg(long)]
        dreidel: bool,
    },

    /// Cast cowrie shells.
    Cowrie {
        #[arg(short, long, default_value = "4")]
        shells: usize,
    },

    /// Draw lots (sortition).
    Lots {
        #[arg(short, long, default_value = "1")]
        count: usize,
        items: Vec<String>,
    },

    /// Serve the HTTP API.
    ///
    /// Binds to loopback by default; pass `--host 0.0.0.0` (or another
    /// interface) to expose the server beyond the local machine.
    #[cfg(feature = "api")]
    Serve {
        /// Network interface to bind (default loopback for safety).
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// TCP port to listen on.
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Start the MCP server (stdio transport).
    #[cfg(feature = "mcp")]
    Mcp,

    /// Start the interactive TUI.
    #[cfg(feature = "tui")]
    Tui,

    /// List available randomness sources.
    Sources,
}

pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Roll { notation } => cmd_roll(&cli, notation),
        Commands::Flip { times, histogram } => cmd_flip(&cli, *times, *histogram),
        Commands::Draw { count } => cmd_draw(&cli, *count),
        Commands::Pick { items, count } => cmd_pick(&cli, items, *count),
        Commands::Shuffle { items } => cmd_shuffle(&cli, items),
        Commands::Int { min, max } => cmd_int(&cli, *min, *max),
        Commands::Bytes { count, hex, base64 } => cmd_bytes(&cli, *count, *hex, *base64),
        Commands::Uuid { version } => cmd_uuid(&cli, *version),
        Commands::Password { length, no_symbols } => cmd_password(&cli, *length, *no_symbols),
        Commands::Runes { count } => cmd_runes(&cli, *count),
        Commands::Iching { method } => cmd_iching(&cli, method),
        Commands::Tarot { count } => cmd_tarot(&cli, *count),
        Commands::Dominoes { set, count } => cmd_dominoes(&cli, *set, *count),
        Commands::Roulette { variant } => cmd_roulette(&cli, variant),
        Commands::Lottery {
            pool,
            pick,
            bonus_pool,
        } => cmd_lottery(&cli, *pool, *pick, *bonus_pool),
        Commands::Knucklebones { count } => cmd_knucklebones(&cli, *count),
        Commands::Teetotum { dreidel } => cmd_teetotum(&cli, *dreidel),
        Commands::Cowrie { shells } => cmd_cowrie(&cli, *shells),
        Commands::Lots { count, items } => cmd_lots(&cli, items, *count),
        #[cfg(feature = "api")]
        Commands::Serve { host, port } => cmd_serve(host, *port),
        #[cfg(feature = "mcp")]
        Commands::Mcp => cmd_mcp(),
        #[cfg(feature = "tui")]
        Commands::Tui => cmd_tui(),
        Commands::Sources => cmd_sources(),
    }
}

fn make_source(cli: &Cli) -> Result<Box<dyn Source>, Box<dyn std::error::Error>> {
    let src = create_source(&cli.source, cli.seed.as_deref())?;
    if src.health() == SourceHealth::Unavailable {
        return Err(format!("source '{}' is unavailable", cli.source).into());
    }
    Ok(src)
}

fn cmd_roll(cli: &Cli, notation: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = roll_dice(source.as_mut(), notation)?;
    let entropy = estimate_dice_entropy(notation).unwrap_or(0.0);

    if cli.json {
        let json = serde_json::json!({
            "result": result.total,
            "notation": notation,
            "source": source.name(),
            "source_kind": source.kind().to_string(),
            "rolls": result.rolls.iter().map(|r| serde_json::json!({
                "value": r.value,
                "size": r.size,
                "exploded": r.exploded,
                "rerolled": r.rerolled,
            })).collect::<Vec<_>>(),
            "dropped": result.dropped.iter().map(|r| r.value).collect::<Vec<_>>(),
            "modifier_total": result.modifier_total,
            "success_count": result.success_count,
            "entropy_bits": entropy,
            "seed": source.seed(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{}", result.total);
        if cli.verbose {
            eprintln!("source: {} ({})", source.name(), source.kind());
            eprintln!(
                "rolls:  {:?}",
                result.rolls.iter().map(|r| r.value).collect::<Vec<_>>()
            );
            if !result.dropped.is_empty() {
                eprintln!(
                    "dropped: {:?}",
                    result.dropped.iter().map(|r| r.value).collect::<Vec<_>>()
                );
            }
            if result.modifier_total != 0 {
                eprintln!("modifier: {}", result.modifier_total);
            }
            if let Some(sc) = result.success_count {
                eprintln!("successes: {}", sc);
            }
            eprintln!("entropy: {:.2} bits", entropy);
        }
    }
    Ok(())
}

fn cmd_flip(cli: &Cli, times: u64, histogram: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let flips = flip_n(source.as_mut(), times)?;

    if cli.json {
        let out: Vec<_> = flips.iter().map(|f| f.to_string()).collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        let heads = flips.iter().filter(|f| **f == CoinSide::Heads).count();
        let tails = flips.len() - heads;
        if times == 1 {
            println!("{}", flips[0]);
        } else if histogram {
            println!("heads: {}\ntails: {}", heads, tails);
        } else {
            let line: Vec<_> = flips.iter().map(|f| f.to_string()).collect();
            println!("{}", line.join(", "));
        }
    }
    Ok(())
}

fn cmd_draw(cli: &Cli, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let cards = draw_cards(source.as_mut(), count)?;

    if cli.json {
        let out: Vec<_> = cards.iter().map(|c| c.to_string()).collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for card in &cards {
            println!("{}", card);
        }
    }
    Ok(())
}

fn cmd_pick(cli: &Cli, items: &[String], count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;

    if count == 1 {
        let winner = pick_one(source.as_mut(), items)?;
        println!("{}", winner);
    } else {
        let winners = pick_distinct(source.as_mut(), items, count)?;
        for w in &winners {
            println!("{}", w);
        }
    }
    Ok(())
}

fn cmd_shuffle(cli: &Cli, items: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let mut items = items.to_vec();
    crate::methods::shuffle::shuffle(source.as_mut(), &mut items)?;
    for item in &items {
        println!("{}", item);
    }
    Ok(())
}

fn cmd_int(cli: &Cli, min: i64, max: i64) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let value = random_i64(source.as_mut(), min, max)?;
    println!("{}", value);
    Ok(())
}

fn cmd_bytes(
    cli: &Cli,
    count: usize,
    hex: bool,
    base64: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let bytes = random_bytes(source.as_mut(), count)?;

    if hex {
        println!("{}", bytes_to_hex(&bytes));
    } else if base64 {
        println!("{}", bytes_to_base64(&bytes));
    } else {
        // Raw bytes to stdout.
        io::stdout().write_all(&bytes)?;
        io::stdout().flush()?;
    }
    Ok(())
}

fn cmd_uuid(cli: &Cli, version: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let uuid = generate_uuid(source.as_mut(), version)?;
    println!("{}", uuid);
    Ok(())
}

fn cmd_password(
    cli: &Cli,
    length: usize,
    no_symbols: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let options = PasswordOptions {
        length,
        symbols: !no_symbols,
        ..Default::default()
    };
    let pw = generate_password(source.as_mut(), &options)?;
    println!("{}", pw);
    Ok(())
}

fn cmd_sources() -> Result<(), Box<dyn std::error::Error>> {
    for name in crate::sources::source_names() {
        println!("{}", name);
    }
    Ok(())
}

/// Crude entropy estimator for dice notation.
fn estimate_dice_entropy(notation: &str) -> Option<f64> {
    use crate::methods::dice::ast::*;
    use crate::methods::dice::parser::parse;

    let expr = parse(notation).ok()?;
    let Expr::Sum(terms) = expr;
    let mut bits = 0.0;
    for (_, term) in terms {
        if let Term::Dice(dice) = term {
            let sides = match dice.size {
                DieSize::Sides(n) => n as f64,
                DieSize::Percentile => 100.0,
                DieSize::Fudge => 3.0,
            };
            bits += dice.count as f64 * uniform_entropy_bits(sides as u64);
        }
    }
    Some(bits)
}

fn cmd_runes(cli: &Cli, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let runes = draw_runes(source.as_mut(), count)?;
    for rune in &runes {
        println!("{}", rune);
    }
    Ok(())
}

fn cmd_iching(cli: &Cli, method: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let reading = cast_iching(source.as_mut(), method)?;
    println!(
        "Primary hexagram: {} - {}",
        reading.primary,
        reading.hexagram_name()
    );
    if let Some(t) = reading.transformed {
        println!("Transformed hexagram: {}", t);
    }
    if cli.verbose {
        for (i, line) in reading.lines.iter().enumerate() {
            println!(
                "Line {}: {} ({})",
                i + 1,
                line.value,
                if line.changing { "changing" } else { "stable" }
            );
        }
    }
    Ok(())
}

fn cmd_tarot(cli: &Cli, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let cards = draw_tarot(source.as_mut(), count)?;
    for card in &cards {
        println!("{}", card);
    }
    Ok(())
}

fn cmd_dominoes(cli: &Cli, set: u8, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let dominoes = draw_dominoes(source.as_mut(), set, count)?;
    for domino in &dominoes {
        println!("{}", domino);
    }
    Ok(())
}

fn cmd_roulette(cli: &Cli, variant: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = spin_roulette(source.as_mut(), variant)?;
    println!("{}", result);
    Ok(())
}

fn cmd_lottery(
    cli: &Cli,
    pool: u8,
    pick: usize,
    bonus_pool: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = draw_lottery(source.as_mut(), pool, pick, bonus_pool)?;
    println!("{}", result);
    Ok(())
}

fn cmd_knucklebones(cli: &Cli, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = cast_knucklebones(source.as_mut(), count)?;
    println!("{}", result);
    Ok(())
}

fn cmd_teetotum(cli: &Cli, dreidel: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = if dreidel {
        spin_dreidel(source.as_mut())?
    } else {
        spin_teetotum(source.as_mut())?
    };
    println!("{}", result);
    Ok(())
}

fn cmd_cowrie(cli: &Cli, shells: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let result = cast_cowrie(source.as_mut(), shells)?;
    println!("{}", result);
    Ok(())
}

fn cmd_lots(cli: &Cli, items: &[String], count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut source = make_source(cli)?;
    let winners = draw_lots(source.as_mut(), items, count)?;
    for winner in &winners {
        println!("{}", winner);
    }
    Ok(())
}

#[cfg(feature = "api")]
fn cmd_serve(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::api::serve(host, port))?;
    Ok(())
}

#[cfg(all(test, feature = "api"))]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn serve_defaults_to_loopback() {
        let cli = Cli::parse_from(["chance", "serve"]);
        match cli.command {
            Commands::Serve { host, port } => {
                assert_eq!(host, "127.0.0.1", "default host must be loopback");
                assert_eq!(port, 8080);
            }
            _ => panic!("expected Serve command"),
        }
    }

    #[test]
    fn serve_accepts_explicit_host() {
        let cli = Cli::parse_from(["chance", "serve", "--host", "0.0.0.0", "-p", "9090"]);
        match cli.command {
            Commands::Serve { host, port } => {
                assert_eq!(host, "0.0.0.0");
                assert_eq!(port, 9090);
            }
            _ => panic!("expected Serve command"),
        }
    }
}

#[cfg(feature = "mcp")]
fn cmd_mcp() -> Result<(), Box<dyn std::error::Error>> {
    crate::mcp::run()?;
    Ok(())
}

#[cfg(feature = "tui")]
fn cmd_tui() -> Result<(), Box<dyn std::error::Error>> {
    crate::tui::run()?;
    Ok(())
}
