#![allow(dead_code)]
use colored::*;
use console::Term;
use std::io::Write;

#[allow(dead_code)]
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn term_width() -> usize {
    Term::stdout().size().1 as usize
}

fn center(text: &str, width: usize) -> String {
    let visible_len = console::measure_text_width(text);
    if visible_len >= width {
        return text.to_string();
    }
    let pad = (width - visible_len) / 2;
    format!("{}{}", " ".repeat(pad), text)
}

pub fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();
    let w = term_width();

    println!();
    println!();

    let logo_lines = [
        " ██████  ██████  ███████ ███    ██",
        "██    ██ ██   ██ ██      ████   ██",
        "██    ██ ██████  █████   ██ ██  ██",
        "██    ██ ██      ██      ██  ██ ██",
        " ██████  ██      ███████ ██   ████",
    ];

    let zax_lines = [
        "███████  █████  ██   ██",
        "   ███  ██   ██  ██ ██ ",
        "  ███   ███████   ███  ",
        " ███    ██   ██  ██ ██ ",
        "███████ ██   ██ ██   ██",
    ];

    for i in 0..5 {
        let combined = format!(
            "{}  {}",
            logo_lines[i].truecolor(100, 180, 255).bold(),
            zax_lines[i].truecolor(255, 180, 60).bold()
        );
        println!("{}", center(&combined, w + 20));
    }

    println!();

    let tagline = format!(
        "{}  {}  {}",
        "━".repeat(8).truecolor(60, 60, 80),
        "Secure AI Assistant".truecolor(140, 140, 170),
        "━".repeat(8).truecolor(60, 60, 80),
    );
    println!("{}", center(&tagline, w + 10));

    println!();
}

pub fn print_welcome() {
    let w = term_width().min(72);
    let inner = w.saturating_sub(4);

    let top = format!(
        "  {}{}{}",
        "╭".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╮".truecolor(60, 60, 80)
    );
    let bot = format!(
        "  {}{}{}",
        "╰".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╯".truecolor(60, 60, 80)
    );

    let pad_line = |content: &str| -> String {
        let vis = console::measure_text_width(content);
        let right_pad = inner.saturating_sub(vis + 1);
        format!(
            "  {} {} {}{}",
            "│".truecolor(60, 60, 80),
            content,
            " ".repeat(right_pad),
            "│".truecolor(60, 60, 80)
        )
    };

    let empty = pad_line("");
    let ask = format!(
        "{}  \"{}\"",
        "Ask anything...".truecolor(100, 100, 130),
        "What can you do?".truecolor(140, 140, 170).italic()
    );

    println!("{}", top);
    println!("{}", empty);
    println!("{}", pad_line(&ask));
    println!("{}", empty);

    let modes = format!(
        "  {}  {}  {}  · {}",
        "Build".truecolor(100, 180, 255).bold(),
        "Agent".truecolor(255, 180, 60).bold(),
        "OpenZax Zen".truecolor(140, 140, 170),
        "max".truecolor(255, 100, 100),
    );
    println!("{}", pad_line(&modes));
    println!("{}", empty);
    println!("{}", bot);

    println!();

    let shortcuts = format!(
        "    {}  {}    {}  {}    {}  {}",
        "/help".truecolor(255, 180, 60),
        "commands".truecolor(100, 100, 130),
        "/models".truecolor(255, 180, 60),
        "switch model".truecolor(100, 100, 130),
        "/clear".truecolor(255, 180, 60),
        "reset".truecolor(100, 100, 130),
    );
    println!("{}", shortcuts);

    println!();

    let tip = format!(
        "    {} {}",
        "●".truecolor(255, 180, 60),
        "Type your message below, or use /help to see all commands".truecolor(100, 100, 130),
    );
    println!("{}", tip);

    println!();
    print_separator();
    println!();
}

pub fn print_prompt(session_id: &str) {
    let id_short = if session_id.len() > 8 {
        &session_id[..8]
    } else {
        session_id
    };
    print!(
        " {} {} ",
        "❯".truecolor(100, 180, 255).bold(),
        id_short.truecolor(70, 70, 90),
    );
    std::io::stdout().flush().unwrap();
}

pub fn print_thinking() {
    println!(
        "  {} {}",
        "◆".truecolor(255, 180, 60),
        "Thinking...".truecolor(140, 140, 170).italic()
    );
}

pub fn print_error(msg: &str) {
    println!(
        "  {} {}",
        "✗".truecolor(255, 80, 80).bold(),
        msg.truecolor(255, 120, 120)
    );
}

pub fn print_success(msg: &str) {
    println!(
        "  {} {}",
        "✓".truecolor(80, 220, 120).bold(),
        msg.truecolor(180, 255, 200)
    );
}

pub fn print_info(msg: &str) {
    println!(
        "  {} {}",
        "●".truecolor(100, 180, 255),
        msg.truecolor(200, 200, 220)
    );
}

pub fn print_model_info(name: &str, provider: &str) {
    let w = term_width().min(72);
    let inner = w.saturating_sub(4);

    let line = format!(
        "  {}{}{}",
        "├".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "┤".truecolor(60, 60, 80),
    );
    println!("{}", line);

    let model_text = format!(
        " {}  {}  {}  {}",
        "◈".truecolor(100, 180, 255),
        name.truecolor(100, 180, 255).bold(),
        "·".truecolor(70, 70, 90),
        provider.truecolor(140, 140, 170),
    );
    println!("{}", model_text);

    println!("{}", line);
    println!();
}

pub fn print_separator() {
    let w = term_width().min(72);
    let inner = w.saturating_sub(4);
    println!("  {}", "─".repeat(inner).truecolor(45, 45, 60));
}

pub fn print_streaming_start() {
    print!("  {} ", "▍".truecolor(100, 180, 255));
    std::io::stdout().flush().unwrap();
}

pub fn print_token(token: &str) {
    print!("{}", token.truecolor(220, 220, 235));
    std::io::stdout().flush().unwrap();
}

pub fn print_streaming_end() {
    println!();
    println!();
}

pub fn print_help() {
    let w = term_width().min(72);
    let inner = w.saturating_sub(4);

    println!();
    let top = format!(
        "  {}{}{}",
        "╭".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╮".truecolor(60, 60, 80),
    );
    let bot = format!(
        "  {}{}{}",
        "╰".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╯".truecolor(60, 60, 80),
    );

    let pad_line = |content: &str| -> String {
        let vis = console::measure_text_width(content);
        let right_pad = inner.saturating_sub(vis + 1);
        format!(
            "  {} {}{}{}",
            "│".truecolor(60, 60, 80),
            content,
            " ".repeat(right_pad),
            "│".truecolor(60, 60, 80)
        )
    };

    println!("{}", top);
    println!(
        "{}",
        pad_line(&format!("{}", " Commands".truecolor(100, 180, 255).bold()))
    );
    println!(
        "{}",
        pad_line(&format!(
            " {}",
            "─".repeat(inner.saturating_sub(3)).truecolor(45, 45, 60)
        ))
    );

    let cmds: Vec<(&str, &str)> = vec![
        ("/help", "Show this help message"),
        ("/clear", "Clear the screen"),
        ("/info", "Show current session info"),
        ("/models", "List available models"),
        ("/model <name>", "Switch to a different model"),
        ("/exit", "Exit the shell"),
    ];

    for (cmd, desc) in &cmds {
        let line = format!(
            " {}  {}",
            format!("{:<16}", cmd).truecolor(255, 180, 60),
            desc.truecolor(140, 140, 170),
        );
        println!("{}", pad_line(&line));
    }

    println!("{}", pad_line(""));
    println!("{}", bot);
    println!();
}

pub fn print_session_info(session_id: &str, model: &str, messages: usize) {
    let w = term_width().min(72);
    let inner = w.saturating_sub(4);

    println!();
    let top = format!(
        "  {}{}{}",
        "╭".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╮".truecolor(60, 60, 80),
    );
    let bot = format!(
        "  {}{}{}",
        "╰".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╯".truecolor(60, 60, 80),
    );

    let pad_line = |content: &str| -> String {
        let vis = console::measure_text_width(content);
        let right_pad = inner.saturating_sub(vis + 1);
        format!(
            "  {} {}{}{}",
            "│".truecolor(60, 60, 80),
            content,
            " ".repeat(right_pad),
            "│".truecolor(60, 60, 80)
        )
    };

    println!("{}", top);
    println!(
        "{}",
        pad_line(&format!(
            "{}",
            " Session Info".truecolor(100, 180, 255).bold()
        ))
    );
    println!(
        "{}",
        pad_line(&format!(
            " {}",
            "─".repeat(inner.saturating_sub(3)).truecolor(45, 45, 60)
        ))
    );

    let rows: Vec<(&str, String)> = vec![
        ("Session", session_id.to_string()),
        ("Model", model.to_string()),
        ("Messages", messages.to_string()),
    ];

    for (label, val) in &rows {
        let line = format!(
            " {}  {}",
            format!("{:<12}", label).truecolor(140, 140, 170),
            val.truecolor(100, 180, 255),
        );
        println!("{}", pad_line(&line));
    }

    println!("{}", pad_line(""));
    println!("{}", bot);
    println!();
}

pub fn print_status_bar(model: &str, provider: &str) {
    let w = term_width().min(72);

    let left = format!(
        " {}  {} · {}",
        "◈".truecolor(100, 180, 255),
        model.truecolor(100, 180, 255).bold(),
        provider.truecolor(140, 140, 170),
    );

    let right = format!("v{}  ", VERSION.truecolor(70, 70, 90));

    let left_vis = console::measure_text_width(&left);
    let right_vis = console::measure_text_width(&right);
    let gap = w.saturating_sub(left_vis + right_vis);

    let bar_bg = "─".repeat(w).truecolor(35, 35, 50);
    println!("{}", bar_bg);
    println!("{}{}{}", left, " ".repeat(gap), right);
    println!("{}", bar_bg);
}

pub fn print_user_message(msg: &str) {
    println!();
    println!(
        "  {} {}",
        "▸".truecolor(100, 180, 255).bold(),
        msg.truecolor(220, 220, 235)
    );
    println!();
}

pub fn print_agent_label() {
    println!(
        "  {} {}",
        "▪".truecolor(255, 180, 60),
        "OpenZax".truecolor(255, 180, 60).bold()
    );
}

pub fn print_cost_info(tokens: usize, duration_secs: f64) {
    println!(
        "  {}",
        format!("  {} tokens · {:.1}s", tokens, duration_secs).truecolor(70, 70, 90)
    );
}

pub fn print_getting_started() {
    let w = term_width().min(60);
    let inner = w.saturating_sub(4);

    let top = format!(
        "  {}{}{}",
        "╭".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╮".truecolor(60, 60, 80),
    );
    let bot = format!(
        "  {}{}{}",
        "╰".truecolor(60, 60, 80),
        "─".repeat(inner).truecolor(60, 60, 80),
        "╯".truecolor(60, 60, 80),
    );

    let pad_line = |content: &str| -> String {
        let vis = console::measure_text_width(content);
        let right_pad = inner.saturating_sub(vis + 1);
        format!(
            "  {} {}{}{}",
            "│".truecolor(60, 60, 80),
            content,
            " ".repeat(right_pad),
            "│".truecolor(60, 60, 80)
        )
    };

    println!("{}", top);
    println!(
        "{}",
        pad_line(&format!(
            " {} {}",
            "◈".truecolor(255, 180, 60),
            "Getting started".truecolor(220, 220, 235).bold()
        ))
    );
    println!("{}", pad_line(""));
    println!(
        "{}",
        pad_line(
            &" OpenZax includes free models"
                .truecolor(140, 140, 170)
                .to_string()
        )
    );
    println!(
        "{}",
        pad_line(
            &" so you can start immediately."
                .truecolor(140, 140, 170)
                .to_string()
        )
    );
    println!("{}", pad_line(""));
    println!(
        "{}",
        pad_line(
            &" Connect from 7+ providers to"
                .truecolor(140, 140, 170)
                .to_string()
        )
    );
    println!(
        "{}",
        pad_line(
            &" use other models, including"
                .truecolor(140, 140, 170)
                .to_string()
        )
    );
    println!(
        "{}",
        pad_line(
            &" Claude, GPT, Gemini etc"
                .truecolor(140, 140, 170)
                .to_string()
        )
    );
    println!("{}", pad_line(""));

    let connect_line = format!(
        " {}{}{}",
        "Connect provider".truecolor(220, 220, 235),
        "         ",
        "/connect".truecolor(255, 180, 60),
    );
    println!("{}", pad_line(&connect_line));

    println!("{}", pad_line(""));
    println!("{}", bot);
}
