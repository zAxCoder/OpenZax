use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use openzax_core::{
    agent::{Agent, AgentConfig},
    event::{Event as OzEvent, EventBus},
    storage::Storage,
};
use uuid::Uuid;
use chrono::Utc;

// ─── Palette (monochrome: black / grays / white) ─────────────────────────────

const BG:       Color = Color::Rgb(10, 10, 10);
const BG_PANEL: Color = Color::Rgb(16, 16, 16);
const BG_INPUT: Color = Color::Rgb(22, 22, 22);
const BG_POPUP: Color = Color::Rgb(18, 18, 18);
const BG_SEL:   Color = Color::Rgb(220, 220, 220);

const BD:       Color = Color::Rgb(55, 55, 55);

const W:        Color = Color::Rgb(245, 245, 245);
const G1:       Color = Color::Rgb(180, 180, 180);
const G2:       Color = Color::Rgb(120, 120, 120);
const G3:       Color = Color::Rgb(70, 70, 70);
const G4:       Color = Color::Rgb(45, 45, 45);
const BLK:      Color = Color::Rgb(10, 10, 10);

// ─── Brand (clean block letters) ─────────────────────────────────────────────

const BRAND: &[&str] = &[
    " @@@@  @@@@@  @@@@@ @@  @@     @@@@@@  @@@  @@  @@",
    "@@  @@ @@  @@ @@    @@@@ @@       @@  @@  @@  @@@@",
    "@@  @@ @@@@@  @@@@  @@ @@@@ @@   @@   @@@@@@   @@",
    "@@  @@ @@     @@    @@  @@@ @@  @@    @@  @@  @@@@",
    " @@@@  @@     @@@@@ @@   @@ @@ @@@@@@ @@  @@ @@  @@",
];

// ─── Intelligence tiers ──────────────────────────────────────────────────────

const TIERS: &[&str] = &["high", "max", "auto"];

// ─── Free models ─────────────────────────────────────────────────────────────

pub struct FreeModel {
    pub id: &'static str,
    pub display: &'static str,
    pub ctx: &'static str,
    pub provider: &'static str,
    pub api_url: &'static str,
}

const FREE_MODELS: &[FreeModel] = &[
    // ── OpenRouter ───────────────────────────────────────────────────────────
    FreeModel { id: "arcee-ai/trinity-large-preview:free",           display: "Trinity Large",     ctx: "128K", provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "stepfun/step-3.5-flash:free",                   display: "Step 3.5 Flash",    ctx: "256K", provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "deepseek/deepseek-r1-0528:free",                display: "DeepSeek R1",       ctx: "128K", provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "qwen/qwen3-235b-a22b:free",                     display: "Qwen3 235B",        ctx: "40K",  provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "meta-llama/llama-3.3-70b-instruct:free",        display: "Llama 3.3 70B",     ctx: "128K", provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "mistralai/mistral-small-3.1-24b-instruct:free", display: "Mistral Small 3.1", ctx: "96K",  provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "google/gemma-3-27b-it:free",                    display: "Gemma 3 27B",       ctx: "96K",  provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    FreeModel { id: "google/gemma-3-4b-it:free",                     display: "Gemma 3 4B",        ctx: "32K",  provider: "OpenRouter", api_url: "https://openrouter.ai/api/v1/chat/completions" },
    // ── Groq ────────────────────────────────────────────────────────────────
    FreeModel { id: "llama-3.3-70b-versatile",                       display: "Llama 3.3 70B",     ctx: "128K", provider: "Groq",       api_url: "https://api.groq.com/openai/v1/chat/completions" },
    FreeModel { id: "llama-3.1-8b-instant",                          display: "Llama 3.1 8B",      ctx: "128K", provider: "Groq",       api_url: "https://api.groq.com/openai/v1/chat/completions" },
    FreeModel { id: "gemma2-9b-it",                                  display: "Gemma 2 9B",        ctx: "8K",   provider: "Groq",       api_url: "https://api.groq.com/openai/v1/chat/completions" },
    FreeModel { id: "mixtral-8x7b-32768",                            display: "Mixtral 8x7B",      ctx: "32K",  provider: "Groq",       api_url: "https://api.groq.com/openai/v1/chat/completions" },
    // ── Cerebras ────────────────────────────────────────────────────────────
    FreeModel { id: "llama-3.3-70b",                                 display: "Llama 3.3 70B",     ctx: "128K", provider: "Cerebras",   api_url: "https://api.cerebras.ai/v1/chat/completions" },
    FreeModel { id: "qwen-3-32b",                                    display: "Qwen3 32B",         ctx: "32K",  provider: "Cerebras",   api_url: "https://api.cerebras.ai/v1/chat/completions" },
];

// ─── Command palette ─────────────────────────────────────────────────────────

struct CmdEntry { label: &'static str, shortcut: &'static str, cat: &'static str }

const CMD_PALETTE: &[CmdEntry] = &[
    CmdEntry { label: "Switch model",     shortcut: "Ctrl+M", cat: "Model" },
    CmdEntry { label: "Intelligence tier", shortcut: "Ctrl+T", cat: "Model" },
    CmdEntry { label: "Switch mode",      shortcut: "Tab",    cat: "Session" },
    CmdEntry { label: "New session",      shortcut: "Ctrl+N", cat: "Session" },
    CmdEntry { label: "Skills",           shortcut: "Ctrl+K", cat: "Tools" },
    CmdEntry { label: "Help",             shortcut: "/help",  cat: "System" },
    CmdEntry { label: "Exit",             shortcut: "Ctrl+C", cat: "System" },
];

// ─── Skills ──────────────────────────────────────────────────────────────────

struct SkillEntry { name: &'static str, desc: &'static str }

const SKILLS: &[SkillEntry] = &[
    SkillEntry { name: "webapp-testing",           desc: "Test and interact with web applications" },
    SkillEntry { name: "frontend-design",          desc: "Production-grade frontend interfaces" },
    SkillEntry { name: "docker-expert",            desc: "Docker containerization & orchestration" },
    SkillEntry { name: "e2e-testing-patterns",     desc: "E2E testing with Playwright & Cypress" },
    SkillEntry { name: "python-testing-patterns",  desc: "Comprehensive testing with pytest" },
    SkillEntry { name: "python-design-patterns",   desc: "KISS, SoC, SRP design patterns" },
    SkillEntry { name: "async-python-patterns",    desc: "Asyncio & concurrent programming" },
    SkillEntry { name: "javascript-testing",       desc: "JS/TS testing with Jest & Vitest" },
    SkillEntry { name: "docker-best-practices",    desc: "Production Docker deployments" },
    SkillEntry { name: "database-migration",       desc: "DB migrations across ORMs" },
    SkillEntry { name: "prisma-database-setup",    desc: "Configure Prisma with any DB" },
    SkillEntry { name: "database-schema-designer", desc: "Scalable database schema design" },
    SkillEntry { name: "rust-systems",             desc: "Advanced Rust system patterns" },
    SkillEntry { name: "security-audit",           desc: "Security auditing for code & deps" },
    SkillEntry { name: "api-design-patterns",      desc: "REST & GraphQL API design" },
    SkillEntry { name: "ci-cd-pipelines",          desc: "CI/CD pipeline optimization" },
    SkillEntry { name: "kubernetes-expert",         desc: "Kubernetes orchestration" },
    SkillEntry { name: "vercel-react",             desc: "React/Next.js performance" },
    SkillEntry { name: "python-performance",       desc: "Profile & optimize Python" },
    SkillEntry { name: "find-skills",              desc: "Discover & install new skills" },
];

// ─── System prompts ──────────────────────────────────────────────────────────

const BUILD_PROMPT: &str = "You are OpenZax, an elite AI coding assistant. Write production-ready code with no shortcuts. Handle all edge cases. Follow SOLID, clean architecture, DRY. Use best practices for the language/framework. Be concise. If complex, break into steps and execute each fully.";

const PLAN_PROMPT: &str = "You are OpenZax in Planning Mode. PLAN before code. For every request: 1) Requirements analysis 2) Architecture design 3) Implementation plan 4) Risk matrix. Never write implementation code.";

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum Msg {
    User(String),
    Assistant(String),
    System(String),
    Status { model: String, secs: f32 },
}

#[derive(PartialEq, Copy, Clone)]
enum Overlay { None, Commands, Skills, Models }

#[derive(PartialEq, Copy, Clone)]
enum Mode { Build, Plan }

#[derive(PartialEq, Copy, Clone)]
enum Phase { Empty, Chat, Stream }

// ─── App ─────────────────────────────────────────────────────────────────────

pub struct App {
    phase: Phase,
    msgs: Vec<Msg>,
    pending_sys: Vec<String>,
    input: String,
    cursor: usize,
    scroll: usize,
    cursor_visible: bool,
    cursor_blink: Instant,
    model_name: String,
    model_short: String,
    model_provider: String,
    model_api: String,
    session_tokens: u32,
    session_start: Instant,
    mode: Mode,
    tier_idx: usize,
    overlay: Overlay,
    ov_idx: usize,
    ov_search: String,
    ov_rect: Rect,
    ov_item_y: u16,
    stream_buf: Arc<Mutex<String>>,
    done_flag: Arc<Mutex<bool>>,
}

impl App {
    pub fn new(model: &str) -> Self {
        let short = model.split('/').last().unwrap_or(model).trim_end_matches(":free").to_string();
        Self {
            phase: Phase::Empty,
            msgs: Vec::new(),
            pending_sys: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll: 0,
            cursor_visible: true,
            cursor_blink: Instant::now(),
            model_name: model.to_string(),
            model_short: short,
            model_provider: "OpenRouter".into(),
            model_api: "https://openrouter.ai/api/v1/chat/completions".into(),
            session_tokens: 0,
            session_start: Instant::now(),
            mode: Mode::Build,
            tier_idx: 0,
            overlay: Overlay::None,
            ov_idx: 0,
            ov_search: String::new(),
            ov_rect: Rect::default(),
            ov_item_y: 0,
            stream_buf: Arc::new(Mutex::new(String::new())),
            done_flag: Arc::new(Mutex::new(false)),
        }
    }

    fn tick_cursor(&mut self) {
        if self.cursor_blink.elapsed() > Duration::from_millis(530) {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink = Instant::now();
        }
    }

    fn reset_cursor(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink = Instant::now();
    }

    fn ins(&mut self, c: char) { self.input.insert(self.cursor, c); self.cursor += c.len_utf8(); self.reset_cursor(); }
    fn bksp(&mut self) {
        if self.cursor > 0 {
            let p = self.input[..self.cursor].char_indices().last().map(|(i,_)|i).unwrap_or(0);
            self.input.remove(p);
            self.cursor = p;
            self.reset_cursor();
        }
    }
    fn left(&mut self) { if self.cursor > 0 { self.cursor = self.input[..self.cursor].char_indices().last().map(|(i,_)|i).unwrap_or(0); self.reset_cursor(); } }
    fn right(&mut self) { if self.cursor < self.input.len() { self.cursor = self.input[self.cursor..].char_indices().nth(1).map(|(i,_)|self.cursor+i).unwrap_or(self.input.len()); self.reset_cursor(); } }
    fn take(&mut self) -> String { let s = std::mem::take(&mut self.input); self.cursor = 0; self.reset_cursor(); s }
    fn sup(&mut self) { self.scroll = self.scroll.saturating_sub(3); }
    fn sdn(&mut self) { self.scroll += 3; }
    fn bot(&mut self) { self.scroll = usize::MAX; }
    fn push(&mut self, m: Msg) { self.msgs.push(m); self.bot(); }
    fn flush(&mut self) {
        let c = { let mut b = self.stream_buf.lock().unwrap(); let s = b.clone(); b.clear(); s };
        if c.is_empty() { return; }
        self.session_tokens += (c.len() / 4) as u32;
        if let Some(Msg::Assistant(ref mut b)) = self.msgs.last_mut() { b.push_str(&c); } else { self.msgs.push(Msg::Assistant(c)); }
        self.bot();
    }
    fn done(&mut self) -> bool { let d = *self.done_flag.lock().unwrap(); if d { *self.done_flag.lock().unwrap() = false; } d }
    fn secs(&self) -> f32 { self.session_start.elapsed().as_secs_f32() }
    fn tier(&self) -> &'static str { TIERS[self.tier_idx] }
    fn mode_label(&self) -> &str { match self.mode { Mode::Build => "Build", Mode::Plan => "Plan" } }
    fn ov_count(&self) -> usize {
        match self.overlay {
            Overlay::Commands => CMD_PALETTE.iter().filter(|e| self.ov_search.is_empty() || e.label.to_lowercase().contains(&self.ov_search.to_lowercase())).count(),
            Overlay::Skills => SKILLS.iter().filter(|s| self.ov_search.is_empty() || s.name.contains(&self.ov_search) || s.desc.to_lowercase().contains(&self.ov_search.to_lowercase())).count(),
            Overlay::Models => FREE_MODELS.len(),
            Overlay::None => 0,
        }
    }
}

// Helper to detect Ctrl+letter (works on Windows cmd.exe too)
fn is_ctrl(key: &crossterm::event::KeyEvent, ch: char) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(ch) {
        return true;
    }
    // Windows cmd.exe sometimes sends raw control codes
    let ctrl_code = (ch as u8).wrapping_sub(b'a').wrapping_add(1);
    if key.code == KeyCode::Char(ctrl_code as char) && ctrl_code < 27 {
        return true;
    }
    false
}

// ─── Render ──────────────────────────────────────────────────────────────────

fn render(f: &mut Frame, app: &mut App) {
    app.tick_cursor();
    f.render_widget(Block::default().style(Style::default().bg(BG)), f.area());
    match app.phase {
        Phase::Empty => draw_empty(f, app),
        _ => draw_chat(f, app),
    }
    match app.overlay {
        Overlay::Commands => draw_commands(f, app),
        Overlay::Skills => draw_skills(f, app),
        Overlay::Models => draw_models(f, app),
        Overlay::None => {}
    }
}

fn draw_empty(f: &mut Frame, app: &App) {
    let a = f.area();
    let brand_h = BRAND.len() as u16;
    let content_h = brand_h + 12;
    let top = a.height.saturating_sub(content_h) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top),
            Constraint::Length(brand_h),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(a);

    // Brand
    let mut bl: Vec<Line> = Vec::new();
    for line in BRAND {
        let styled: String = line.replace('@', "\u{2588}");
        bl.push(Line::from(Span::styled(styled, Style::default().fg(W).add_modifier(Modifier::BOLD))));
    }
    f.render_widget(
        Paragraph::new(bl).alignment(Alignment::Center).style(Style::default().bg(BG)),
        chunks[1],
    );

    // Input
    let ic = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(18), Constraint::Percentage(64), Constraint::Percentage(18)])
        .split(chunks[3]);
    draw_input(f, app, ic[1]);

    // Mode + tier
    let ml = Line::from(vec![
        Span::styled(app.mode_label(), Style::default().fg(W).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::default().fg(G4)),
        Span::styled(app.tier(), Style::default().fg(G2)),
    ]);
    f.render_widget(Paragraph::new(ml).alignment(Alignment::Center).style(Style::default().bg(BG)), chunks[5]);

    // Shortcuts
    let sc = Line::from(vec![
        Span::styled("Ctrl+T ", Style::default().fg(G2)),
        Span::styled("tier   ", Style::default().fg(G4)),
        Span::styled("Tab ", Style::default().fg(G2)),
        Span::styled("mode   ", Style::default().fg(G4)),
        Span::styled("Ctrl+P ", Style::default().fg(G2)),
        Span::styled("commands   ", Style::default().fg(G4)),
        Span::styled("Ctrl+M ", Style::default().fg(G2)),
        Span::styled("model", Style::default().fg(G4)),
    ]);
    f.render_widget(Paragraph::new(sc).alignment(Alignment::Center).style(Style::default().bg(BG)), chunks[7]);

    // Bottom tip
    let tip = Line::from(vec![
        Span::styled("Ctrl+N ", Style::default().fg(G3)),
        Span::styled("new   ", Style::default().fg(G4)),
        Span::styled("Ctrl+K ", Style::default().fg(G3)),
        Span::styled("skills   ", Style::default().fg(G4)),
        Span::styled("/help ", Style::default().fg(G3)),
        Span::styled("commands", Style::default().fg(G4)),
    ]);
    f.render_widget(Paragraph::new(tip).alignment(Alignment::Center).style(Style::default().bg(BG)), chunks[9]);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.phase == Phase::Stream { G3 } else { BD }))
        .style(Style::default().bg(BG_INPUT));
    let inner = blk.inner(area);
    f.render_widget(blk, area);

    let cursor_char = if app.cursor_visible { "\u{2588}" } else { " " };

    let line = if app.input.is_empty() {
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(G1).add_modifier(Modifier::BOLD)),
            Span::styled(cursor_char, Style::default().fg(W)),
            Span::styled(" Ask anything...", Style::default().fg(G4)),
        ])
    } else {
        let before = &app.input[..app.cursor];
        let after = if app.cursor < app.input.len() { &app.input[app.cursor..] } else { "" };
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(G1).add_modifier(Modifier::BOLD)),
            Span::styled(before, Style::default().fg(W)),
            Span::styled(cursor_char, Style::default().fg(W)),
            Span::styled(after, Style::default().fg(W)),
        ])
    };
    f.render_widget(Paragraph::new(line).style(Style::default().bg(BG_INPUT)), inner);
}

fn draw_chat(f: &mut Frame, app: &mut App) {
    let a = f.area();
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(32)])
        .split(a);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(cols[0]);

    draw_messages(f, app, rows[0]);
    draw_bottom(f, app, rows[1]);
    draw_sidebar(f, app, cols[1]);
}

fn draw_messages(f: &mut Frame, app: &mut App, area: Rect) {
    let w = area.width.saturating_sub(4) as usize;
    let ml = app.mode_label().to_string();
    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.msgs {
        match msg {
            Msg::User(t) => {
                lines.push(Line::default());
                lines.push(Line::from(vec![
                    Span::styled(" > ", Style::default().fg(W).add_modifier(Modifier::BOLD)),
                    Span::styled(t.as_str(), Style::default().fg(W).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::default());
            }
            Msg::Assistant(t) => {
                for wr in wrap(t, w) {
                    lines.push(Line::from(vec![Span::raw("  "), Span::styled(wr, Style::default().fg(G1))]));
                }
                lines.push(Line::default());
            }
            Msg::Status { model, secs } => {
                lines.push(Line::from(vec![
                    Span::styled("  . ", Style::default().fg(G3)),
                    Span::styled(ml.as_str(), Style::default().fg(G2)),
                    Span::styled(" . ", Style::default().fg(G4)),
                    Span::styled(model.as_str(), Style::default().fg(G3)),
                    Span::styled(format!(" . {:.1}s", secs), Style::default().fg(G4)),
                ]));
                lines.push(Line::default());
            }
            Msg::System(t) => {
                if t != "__EXIT__" {
                    lines.push(Line::from(vec![
                        Span::styled("  . ", Style::default().fg(G4)),
                        Span::styled(t.as_str(), Style::default().fg(G3)),
                    ]));
                }
            }
        }
    }

    let total = lines.len();
    let vis = area.height as usize;
    let mx = total.saturating_sub(vis);
    if app.scroll >= usize::MAX / 2 { app.scroll = mx; } else { app.scroll = app.scroll.min(mx); }
    f.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(BG)).scroll((app.scroll as u16, 0)),
        area,
    );
}

fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let info = Line::from(vec![
        Span::styled(format!(" {} ", app.mode_label()), Style::default().fg(W).add_modifier(Modifier::BOLD)),
        Span::styled(". ", Style::default().fg(G4)),
        Span::styled(app.tier(), Style::default().fg(G2)),
        Span::styled(format!("  {}  ", app.model_short), Style::default().fg(G3)),
    ]);
    f.render_widget(Paragraph::new(info).style(Style::default().bg(BG)), rows[0]);
    draw_input(f, app, rows[1]);

    let sc = Line::from(vec![
        Span::styled(" Ctrl+T ", Style::default().fg(G2)),
        Span::styled("tier  ", Style::default().fg(G4)),
        Span::styled("Tab ", Style::default().fg(G2)),
        Span::styled("mode  ", Style::default().fg(G4)),
        Span::styled("Ctrl+P ", Style::default().fg(G2)),
        Span::styled("cmds  ", Style::default().fg(G4)),
        Span::styled("Ctrl+M ", Style::default().fg(G2)),
        Span::styled("model  ", Style::default().fg(G4)),
        Span::styled("Esc ", Style::default().fg(G2)),
        Span::styled("cancel", Style::default().fg(G4)),
    ]);
    f.render_widget(Paragraph::new(sc).style(Style::default().bg(BG)), rows[2]);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(
        Block::default().borders(Borders::LEFT).border_style(Style::default().fg(G4)).style(Style::default().bg(BG_PANEL)),
        area,
    );
    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
    let title = app.msgs.iter().find_map(|m| if let Msg::User(t) = m { Some(t.chars().take(20).collect::<String>()) } else { None }).unwrap_or_else(|| "New session".into());

    let l = vec![
        Line::from(Span::styled(&title, Style::default().fg(W).add_modifier(Modifier::BOLD))),
        Line::default(),
        Line::from(Span::styled("Context", Style::default().fg(G2).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} tokens", app.session_tokens), Style::default().fg(G3))),
        Line::from(Span::styled(format!("  {:.0}s elapsed", app.secs()), Style::default().fg(G3))),
        Line::from(Span::styled("  $0.00 (free)", Style::default().fg(G2))),
        Line::default(),
        Line::from(Span::styled("Model", Style::default().fg(G2).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {}", app.model_short), Style::default().fg(W))),
        Line::from(Span::styled(format!("  {}", app.model_provider), Style::default().fg(G3))),
        Line::default(),
        Line::from(Span::styled("Free API keys", Style::default().fg(G2).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  openrouter.ai/keys", Style::default().fg(G1))),
        Line::from(Span::styled("  console.groq.com", Style::default().fg(G1))),
        Line::from(Span::styled("  cloud.cerebras.ai", Style::default().fg(G1))),
    ];
    f.render_widget(Paragraph::new(Text::from(l)).style(Style::default().bg(BG_PANEL)).wrap(Wrap { trim: false }), inner);
}

// ─── Overlays ────────────────────────────────────────────────────────────────

fn popup_rect(area: Rect, w: u16, h: u16) -> Rect {
    let pw = w.min(area.width.saturating_sub(4));
    let ph = h.min(area.height.saturating_sub(2));
    Rect::new((area.width.saturating_sub(pw)) / 2, (area.height.saturating_sub(ph)) / 2, pw, ph)
}

fn draw_commands(f: &mut Frame, app: &mut App) {
    let popup = popup_rect(f.area(), 48, 18);
    app.ov_rect = popup;
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default().borders(Borders::ALL).border_style(Style::default().fg(G4))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(" Commands ", Style::default().fg(W).add_modifier(Modifier::BOLD)));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let sl = if app.ov_search.is_empty() {
        Line::from(Span::styled(" Search...", Style::default().fg(G3)))
    } else {
        Line::from(Span::styled(format!(" {}", app.ov_search), Style::default().fg(W)))
    };
    f.render_widget(Paragraph::new(sl).style(Style::default().bg(BG_INPUT)), rows[0]);
    f.render_widget(Paragraph::new(Line::from(Span::styled("\u{2500}".repeat(rows[1].width as usize), Style::default().fg(G4)))), rows[1]);

    app.ov_item_y = rows[2].y;
    let filtered: Vec<&CmdEntry> = CMD_PALETTE.iter().filter(|e| app.ov_search.is_empty() || e.label.to_lowercase().contains(&app.ov_search.to_lowercase())).collect();
    let mut lines: Vec<Line> = Vec::new();
    let mut last = "";
    let mut gi = 0usize;
    for entry in &filtered {
        if entry.cat != last {
            lines.push(Line::from(Span::styled(format!(" {}", entry.cat), Style::default().fg(G2).add_modifier(Modifier::BOLD))));
            last = entry.cat;
        }
        let sel = gi == app.ov_idx;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G1, BG_POPUP) };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<26}", entry.label), Style::default().fg(fg).bg(bg)),
            Span::styled(format!("{:>12}", entry.shortcut), Style::default().fg(if sel { G3 } else { G4 }).bg(bg)),
        ]));
        gi += 1;
    }
    f.render_widget(Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)), rows[2]);
}

fn draw_skills(f: &mut Frame, app: &mut App) {
    let popup = popup_rect(f.area(), 60, 24);
    app.ov_rect = popup;
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default().borders(Borders::ALL).border_style(Style::default().fg(G4))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(" Skills ", Style::default().fg(W).add_modifier(Modifier::BOLD)));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let sl = if app.ov_search.is_empty() {
        Line::from(Span::styled(" Search skills...", Style::default().fg(G3)))
    } else {
        Line::from(Span::styled(format!(" {}", app.ov_search), Style::default().fg(W)))
    };
    f.render_widget(Paragraph::new(sl).style(Style::default().bg(BG_INPUT)), rows[0]);
    f.render_widget(Paragraph::new(Line::from(Span::styled("\u{2500}".repeat(rows[1].width as usize), Style::default().fg(G4)))), rows[1]);

    app.ov_item_y = rows[2].y;
    let nc = 24usize;
    let dc = (inner.width as usize).saturating_sub(nc + 4);
    let filtered: Vec<&SkillEntry> = SKILLS.iter().filter(|s| app.ov_search.is_empty() || s.name.contains(&app.ov_search) || s.desc.to_lowercase().contains(&app.ov_search.to_lowercase())).collect();
    let mut lines: Vec<Line> = Vec::new();
    for (i, e) in filtered.iter().enumerate() {
        let sel = i == app.ov_idx;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G2, BG_POPUP) };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<w$}", e.name.chars().take(nc).collect::<String>(), w = nc), Style::default().fg(if sel { BLK } else { W }).bg(bg)),
            Span::styled(e.desc.chars().take(dc).collect::<String>(), Style::default().fg(fg).bg(bg)),
        ]));
    }
    f.render_widget(Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)), rows[2]);
}

fn draw_models(f: &mut Frame, app: &mut App) {
    let h = (FREE_MODELS.len() as u16) + 5;
    let popup = popup_rect(f.area(), 54, h);
    app.ov_rect = popup;
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default().borders(Borders::ALL).border_style(Style::default().fg(G4))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(" Switch Model ", Style::default().fg(W).add_modifier(Modifier::BOLD)));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(Paragraph::new(Line::from(Span::styled(" Free models (no credit card)", Style::default().fg(G2).add_modifier(Modifier::BOLD)))).style(Style::default().bg(BG_POPUP)), rows[0]);
    f.render_widget(Paragraph::new(Line::from(Span::styled("\u{2500}".repeat(rows[1].width as usize), Style::default().fg(G4)))), rows[1]);

    app.ov_item_y = rows[2].y;
    let mut lines: Vec<Line> = Vec::new();
    for (i, m) in FREE_MODELS.iter().enumerate() {
        let sel = i == app.ov_idx;
        let cur = app.model_name == m.id;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G1, BG_POPUP) };
        let mark = if cur { "> " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(mark, Style::default().fg(if cur { W } else { G4 }).bg(bg)),
            Span::styled(format!("{:<20}", m.display), Style::default().fg(fg).bg(bg)),
            Span::styled(format!("{:>6}", m.ctx), Style::default().fg(if sel { G3 } else { G4 }).bg(bg)),
            Span::styled(format!("  {:<10}", m.provider), Style::default().fg(if sel { G3 } else { G3 }).bg(bg)),
        ]));
    }
    f.render_widget(Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)), rows[2]);
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 { return vec![text.to_string()]; }
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.is_empty() { out.push(String::new()); continue; }
        let mut cur = String::new();
        let mut cl = 0;
        for w in raw.split_whitespace() {
            let wl = w.chars().count();
            if cl == 0 { cur.push_str(w); cl = wl; }
            else if cl + 1 + wl <= width { cur.push(' '); cur.push_str(w); cl += 1 + wl; }
            else { out.push(std::mem::take(&mut cur)); cur.push_str(w); cl = wl; }
        }
        if !cur.is_empty() { out.push(cur); }
    }
    out
}

fn handle_slash(app: &mut App, cmd: &str) -> bool {
    match cmd.trim() {
        "/help" | "/h" => { app.push(Msg::System("Tab mode . Ctrl+T tier . Ctrl+P cmds . Ctrl+M model . Ctrl+K skills . Ctrl+N new . /exit quit".into())); true }
        "/clear" | "/new" => { app.msgs.clear(); app.phase = Phase::Empty; app.session_tokens = 0; app.session_start = Instant::now(); true }
        "/model" => { let info = format!("{} ({})", app.model_name, app.model_provider); app.push(Msg::System(info)); true }
        "/exit" | "/quit" | "/q" => { app.push(Msg::System("__EXIT__".into())); true }
        _ => false,
    }
}

// ─── Entry ───────────────────────────────────────────────────────────────────

pub async fn run_tui(model_name: String, api_key: Option<String>, db_path: std::path::PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = main_loop(&mut terminal, model_name, api_key, db_path).await;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    res
}

async fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model_name: String,
    api_key: Option<String>,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut app = App::new(&model_name);

    let key = api_key
        .or_else(|| std::env::var("OPENZAX_API_KEY").ok())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok());

    if key.is_none() {
        app.pending_sys.push("No API key. Get free keys (no credit card):".into());
        app.pending_sys.push("  openrouter.ai/keys  .  console.groq.com  .  cloud.cerebras.ai".into());
        app.pending_sys.push("Then: set OPENROUTER_API_KEY=sk-or-... && openzax".into());
    }

    if let Some(p) = db_path.parent() { std::fs::create_dir_all(p).ok(); }
    let eb = EventBus::default();
    let cfg = AgentConfig {
        api_url: "https://openrouter.ai/api/v1/chat/completions".into(),
        api_key: key.clone(),
        model: model_name.clone(),
        system_prompt: Some(BUILD_PROMPT.to_string()),
        ..Default::default()
    };
    let agent = Arc::new(Agent::new(cfg, eb.clone()));
    let storage = Storage::new(&db_path)?;
    let conv_id = Uuid::new_v4();
    storage.create_conversation(conv_id)?;

    {
        let mut rx = eb.subscribe();
        let buf = Arc::clone(&app.stream_buf);
        let df = Arc::clone(&app.done_flag);
        tokio::spawn(async move {
            while let Ok(ev) = rx.recv().await {
                match ev {
                    OzEvent::AgentTokenStream { token, .. } => { buf.lock().unwrap().push_str(&token); }
                    OzEvent::AgentOutput { .. } => { *df.lock().unwrap() = true; }
                    _ => {}
                }
            }
        });
    }

    loop {
        if app.phase == Phase::Stream {
            app.flush();
            if app.done() {
                let s = app.secs();
                let m = app.model_short.clone();
                app.push(Msg::Status { model: m, secs: s });
                app.phase = Phase::Chat;
            }
        }
        terminal.draw(|f| render(f, &mut app))?;
        if !event::poll(Duration::from_millis(40))? { continue; }

        match event::read()? {
            Event::Mouse(me) => {
                if app.overlay != Overlay::None {
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let (mx, my) = (me.column, me.row);
                            if mx < app.ov_rect.x || mx >= app.ov_rect.x + app.ov_rect.width
                                || my < app.ov_rect.y || my >= app.ov_rect.y + app.ov_rect.height {
                                app.overlay = Overlay::None; app.ov_search.clear(); continue;
                            }
                            if my >= app.ov_item_y {
                                let clicked = (my - app.ov_item_y) as usize;
                                let actual = if app.overlay == Overlay::Commands {
                                    resolve_cmd_click(&app, clicked)
                                } else {
                                    let max = app.ov_count();
                                    if clicked < max { Some(clicked) } else { None }
                                };
                                if let Some(idx) = actual {
                                    app.ov_idx = idx;
                                    execute_overlay(&mut app, &agent);
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => { if app.ov_idx > 0 { app.ov_idx -= 1; } }
                        MouseEventKind::ScrollDown => { let mx = app.ov_count(); if app.ov_idx + 1 < mx { app.ov_idx += 1; } }
                        _ => {}
                    }
                } else {
                    match me.kind {
                        MouseEventKind::ScrollUp => app.sup(),
                        MouseEventKind::ScrollDown => app.sdn(),
                        _ => {}
                    }
                }
            }

            Event::Key(key) => {
                if key.kind != KeyEventKind::Press { continue; }

                // Ctrl+C always exits
                if is_ctrl(&key, 'c') { break; }

                // Overlay input
                if app.overlay != Overlay::None {
                    match key.code {
                        KeyCode::Esc => { app.overlay = Overlay::None; app.ov_search.clear(); }
                        KeyCode::Up => { if app.ov_idx > 0 { app.ov_idx -= 1; } }
                        KeyCode::Down => { let mx = app.ov_count(); if app.ov_idx + 1 < mx { app.ov_idx += 1; } }
                        KeyCode::Enter => execute_overlay(&mut app, &agent),
                        KeyCode::Backspace => { app.ov_search.pop(); app.ov_idx = 0; }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => { app.ov_search.push(c); app.ov_idx = 0; }
                        _ => {}
                    }
                    continue;
                }

                // Ctrl shortcuts (must be checked BEFORE general char input)
                if is_ctrl(&key, 'p') { app.overlay = Overlay::Commands; app.ov_idx = 0; app.ov_search.clear(); continue; }
                if is_ctrl(&key, 'm') { app.overlay = Overlay::Models; app.ov_idx = 0; app.ov_search.clear(); continue; }
                if is_ctrl(&key, 'k') { app.overlay = Overlay::Skills; app.ov_idx = 0; app.ov_search.clear(); continue; }
                if is_ctrl(&key, 'n') { app.msgs.clear(); app.phase = Phase::Empty; app.session_tokens = 0; app.session_start = Instant::now(); continue; }
                if is_ctrl(&key, 't') {
                    app.tier_idx = (app.tier_idx + 1) % TIERS.len();
                    if app.phase != Phase::Empty { app.push(Msg::System(format!("Tier: {}", TIERS[app.tier_idx]))); }
                    continue;
                }

                // Skip any other Ctrl combos so they don't get typed as chars
                if key.modifiers.contains(KeyModifiers::CONTROL) { continue; }

                if key.code == KeyCode::Tab && app.phase != Phase::Stream {
                    app.mode = match app.mode { Mode::Build => Mode::Plan, Mode::Plan => Mode::Build };
                    agent.set_system_prompt(match app.mode { Mode::Build => BUILD_PROMPT, Mode::Plan => PLAN_PROMPT }.to_string());
                    if app.phase != Phase::Empty { app.push(Msg::System(format!("Mode: {}", app.mode_label()))); }
                    continue;
                }

                if app.phase == Phase::Stream {
                    match key.code { KeyCode::Up => app.sup(), KeyCode::Down => app.sdn(), _ => {} }
                    continue;
                }

                match key.code {
                    KeyCode::Enter => {
                        let text = app.take();
                        if text.trim().is_empty() { continue; }
                        if text.trim().starts_with('/') {
                            handle_slash(&mut app, text.trim());
                            if app.msgs.last().map(|m| matches!(m, Msg::System(s) if s == "__EXIT__")).unwrap_or(false) { break; }
                            continue;
                        }
                        if app.phase == Phase::Empty {
                            app.phase = Phase::Chat;
                            app.session_start = Instant::now();
                            for s in std::mem::take(&mut app.pending_sys) { app.push(Msg::System(s)); }
                        }
                        app.push(Msg::User(text.clone()));
                        storage.save_message(Uuid::new_v4(), conv_id, "user", &text).ok();
                        eb.publish(OzEvent::UserInput { session_id: conv_id, content: text.clone(), attachments: vec![], timestamp: Utc::now() }).ok();
                        app.msgs.push(Msg::Assistant(String::new()));
                        app.bot();
                        app.phase = Phase::Stream;
                        let ag = Arc::clone(&agent);
                        let sb = Arc::clone(&app.stream_buf);
                        let df = Arc::clone(&app.done_flag);
                        tokio::spawn(async move {
                            if let Err(e) = ag.process_streaming(&text).await {
                                sb.lock().unwrap().push_str(&format!("\n[Error] {}", e));
                                *df.lock().unwrap() = true;
                            }
                        });
                    }
                    KeyCode::Char(c) => app.ins(c),
                    KeyCode::Backspace => app.bksp(),
                    KeyCode::Left => app.left(),
                    KeyCode::Right => app.right(),
                    KeyCode::Up => app.sup(),
                    KeyCode::Down => app.sdn(),
                    KeyCode::Home => { app.cursor = 0; app.reset_cursor(); }
                    KeyCode::End => { app.cursor = app.input.len(); app.reset_cursor(); }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_cmd_click(app: &App, clicked: usize) -> Option<usize> {
    let filtered: Vec<&CmdEntry> = CMD_PALETTE.iter().filter(|e| app.ov_search.is_empty() || e.label.to_lowercase().contains(&app.ov_search.to_lowercase())).collect();
    let mut line = 0usize;
    let mut last = "";
    for (gi, e) in filtered.iter().enumerate() {
        if e.cat != last { if line == clicked { return None; } line += 1; last = e.cat; }
        if line == clicked { return Some(gi); }
        line += 1;
    }
    None
}

fn execute_overlay(app: &mut App, agent: &Arc<Agent>) {
    match app.overlay {
        Overlay::Commands => {
            let filtered: Vec<&CmdEntry> = CMD_PALETTE.iter().filter(|e| app.ov_search.is_empty() || e.label.to_lowercase().contains(&app.ov_search.to_lowercase())).collect();
            if let Some(e) = filtered.get(app.ov_idx) {
                match e.label {
                    "Exit" => { app.push(Msg::System("__EXIT__".into())); }
                    "New session" => { app.msgs.clear(); app.phase = Phase::Empty; app.session_tokens = 0; app.session_start = Instant::now(); }
                    "Switch model" => { app.overlay = Overlay::Models; app.ov_idx = 0; app.ov_search.clear(); return; }
                    "Skills" => { app.overlay = Overlay::Skills; app.ov_idx = 0; app.ov_search.clear(); return; }
                    "Switch mode" => {
                        app.mode = match app.mode { Mode::Build => Mode::Plan, Mode::Plan => Mode::Build };
                        agent.set_system_prompt(match app.mode { Mode::Build => BUILD_PROMPT, Mode::Plan => PLAN_PROMPT }.to_string());
                        app.push(Msg::System(format!("Mode: {}", app.mode_label())));
                    }
                    "Intelligence tier" => { app.tier_idx = (app.tier_idx + 1) % TIERS.len(); app.push(Msg::System(format!("Tier: {}", TIERS[app.tier_idx]))); }
                    "Help" => { app.push(Msg::System("Tab mode . Ctrl+T tier . Ctrl+P cmds . Ctrl+M model".into())); }
                    _ => {}
                }
            }
        }
        Overlay::Models => {
            if let Some(m) = FREE_MODELS.get(app.ov_idx) {
                app.model_name = m.id.to_string();
                app.model_short = m.display.to_string();
                app.model_provider = m.provider.to_string();
                app.model_api = m.api_url.to_string();
                agent.set_model(m.id.to_string());
                app.push(Msg::System(format!("Model: {} ({})", m.display, m.provider)));
            }
        }
        Overlay::Skills => {
            let filtered: Vec<&SkillEntry> = SKILLS.iter().filter(|s| app.ov_search.is_empty() || s.name.contains(&app.ov_search) || s.desc.to_lowercase().contains(&app.ov_search.to_lowercase())).collect();
            if let Some(s) = filtered.get(app.ov_idx) { app.push(Msg::System(format!("Skill: {}", s.name))); }
        }
        Overlay::None => {}
    }
    app.overlay = Overlay::None;
    app.ov_search.clear();
}
