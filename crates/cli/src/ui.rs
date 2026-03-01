use colored::*;
use console::Term;
use std::io::Write;

pub fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();
    
    println!();
    println!("{}", "   ██████  ██████  ███████ ███    ██ ███████  █████  ██   ██".bright_cyan().bold());
    println!("{}", "  ██    ██ ██   ██ ██      ████   ██    ███  ██   ██  ██ ██ ".bright_cyan().bold());
    println!("{}", "  ██    ██ ██████  █████   ██ ██  ██   ███   ███████   ███  ".bright_cyan().bold());
    println!("{}", "  ██    ██ ██      ██      ██  ██ ██  ███    ██   ██  ██ ██ ".bright_cyan().bold());
    println!("{}", "   ██████  ██      ███████ ██   ████ ███████ ██   ██ ██   ██".bright_cyan().bold());
    println!();
    println!("{}", "  Secure AI Development Assistant".bright_white().dimmed());
    println!("{}", "  Built with Rust • WASM Sandbox • Zero-Trust Security".bright_white().dimmed());
    println!();
}

pub fn print_welcome() {
    println!("{}", "  Welcome to OpenZax!".bright_green().bold());
    println!();
    println!("  {} Type your message and press Enter", "•".bright_cyan());
    println!("  {} Use {} to see available commands", "•".bright_cyan(), "/help".bright_yellow());
    println!("  {} Press {} to exit", "•".bright_cyan(), "Ctrl+C".bright_yellow());
    println!();
    println!("{}", "  ─────────────────────────────────────────────────────────".dimmed());
    println!();
}

pub fn print_prompt(session_id: &str) {
    print!("{} {} ", "❯".bright_cyan().bold(), session_id.bright_black());
    std::io::stdout().flush().unwrap();
}

pub fn print_thinking() {
    println!("{} {}", "⚡".bright_yellow(), "Thinking...".bright_white().dimmed());
}

pub fn print_error(msg: &str) {
    println!("{} {}", "✗".bright_red().bold(), msg.bright_red());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".bright_green().bold(), msg.bright_green());
}

pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".bright_blue().bold(), msg.bright_white());
}

pub fn print_model_info(name: &str, provider: &str) {
    println!();
    println!("{} {} {}", 
        "Model:".bright_white().dimmed(), 
        name.bright_cyan().bold(),
        format!("({})", provider).bright_black()
    );
}

pub fn print_separator() {
    println!("{}", "  ─────────────────────────────────────────────────────────".dimmed());
}

pub fn print_streaming_start() {
    print!("{} ", "🤖".to_string());
    std::io::stdout().flush().unwrap();
}

pub fn print_token(token: &str) {
    print!("{}", token.bright_white());
    std::io::stdout().flush().unwrap();
}

pub fn print_streaming_end() {
    println!();
    println!();
}

pub fn print_help() {
    println!();
    println!("{}", "  Available Commands:".bright_cyan().bold());
    println!();
    println!("  {} - Show this help message", "/help".bright_yellow());
    println!("  {} - Clear the screen", "/clear".bright_yellow());
    println!("  {} - Show current session info", "/info".bright_yellow());
    println!("  {} - List available models", "/models".bright_yellow());
    println!("  {} - Switch model", "/model <name>".bright_yellow());
    println!("  {} - Exit the shell", "/exit".bright_yellow());
    println!();
}

pub fn print_session_info(session_id: &str, model: &str, messages: usize) {
    println!();
    println!("{}", "  Session Information:".bright_cyan().bold());
    println!();
    println!("  {} {}", "Session ID:".bright_white().dimmed(), session_id.bright_cyan());
    println!("  {} {}", "Model:".bright_white().dimmed(), model.bright_cyan());
    println!("  {} {}", "Messages:".bright_white().dimmed(), messages.to_string().bright_cyan());
    println!();
}
