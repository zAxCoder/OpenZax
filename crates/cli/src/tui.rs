use chrono::Utc;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use openzax_core::{
    agent::{Agent, AgentConfig},
    event::{Event as OzEvent, EventBus},
    storage::Storage,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use uuid::Uuid;

fn clipboard_paste() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

// ─── Ctrl+C global exit flag ─────────────────────────────────────────────────

static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

// ─── Palette ─────────────────────────────────────────────────────────────────

const BG: Color = Color::Rgb(10, 10, 10);
const BG_PANEL: Color = Color::Rgb(16, 16, 16);
const BG_INPUT: Color = Color::Rgb(22, 22, 22);
const BG_POPUP: Color = Color::Rgb(20, 20, 20);
const BG_SEL: Color = Color::Rgb(220, 220, 220);

const W: Color = Color::Rgb(245, 245, 245);
const G1: Color = Color::Rgb(180, 180, 180);
const G2: Color = Color::Rgb(120, 120, 120);
const G3: Color = Color::Rgb(70, 70, 70);
const G4: Color = Color::Rgb(45, 45, 45);
const BLK: Color = Color::Rgb(10, 10, 10);

// ─── Brand logo (two-tone block font) ────────────────────────────────────────

const BRAND_OPEN: &[&str] = &[
    " ██████  ██████  ███████ ███    ██",
    "██    ██ ██   ██ ██      ████   ██",
    "██    ██ ██████  █████   ██ ██  ██",
    "██    ██ ██      ██      ██  ██ ██",
    " ██████  ██      ███████ ██   ████",
];

const BRAND_ZAX: &[&str] = &[
    "███████  █████  ██   ██",
    "   ███  ██   ██  ██ ██ ",
    "  ███   ███████   ███  ",
    " ███    ██   ██  ██ ██ ",
    "███████ ██   ██ ██   ██",
];

const ACCENT_BLUE: Color = Color::Rgb(100, 180, 255);
const ACCENT_GOLD: Color = Color::Rgb(255, 180, 60);
const VERSION: &str = env!("CARGO_PKG_VERSION");

// ─── Intelligence tiers ──────────────────────────────────────────────────────

const TIERS: &[&str] = &["high", "max", "auto"];

// ─── Free models ─────────────────────────────────────────────────────────────

pub struct FreeModel {
    pub id: &'static str,
    pub display: &'static str,
    pub ctx: &'static str,
    pub provider: &'static str,
    pub api_url: &'static str,
    pub key_env: &'static str,
}

const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id: "arcee-ai/trinity-large-preview:free",
        display: "Trinity Large",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "stepfun/step-3.5-flash:free",
        display: "Step 3.5 Flash",
        ctx: "256K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "deepseek/deepseek-r1-0528:free",
        display: "DeepSeek R1",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "qwen/qwen3-235b-a22b:free",
        display: "Qwen3 235B",
        ctx: "40K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "meta-llama/llama-3.3-70b-instruct:free",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "mistralai/mistral-small-3.1-24b-instruct:free",
        display: "Mistral Small 3.1",
        ctx: "96K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "google/gemma-3-27b-it:free",
        display: "Gemma 3 27B",
        ctx: "96K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "google/gemma-3-4b-it:free",
        display: "Gemma 3 4B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
    },
    FreeModel {
        id: "llama-3.3-70b-versatile",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
    },
    FreeModel {
        id: "llama-3.1-8b-instant",
        display: "Llama 3.1 8B",
        ctx: "128K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
    },
    FreeModel {
        id: "gemma2-9b-it",
        display: "Gemma 2 9B",
        ctx: "8K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
    },
    FreeModel {
        id: "mixtral-8x7b-32768",
        display: "Mixtral 8x7B",
        ctx: "32K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
    },
    FreeModel {
        id: "llama-3.3-70b",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "Cerebras",
        api_url: "https://api.cerebras.ai/v1/chat/completions",
        key_env: "CEREBRAS_API_KEY",
    },
    FreeModel {
        id: "qwen-3-32b",
        display: "Qwen3 32B",
        ctx: "32K",
        provider: "Cerebras",
        api_url: "https://api.cerebras.ai/v1/chat/completions",
        key_env: "CEREBRAS_API_KEY",
    },
];

// ─── API Providers ───────────────────────────────────────────────────────────

struct ApiProvider {
    name: &'static str,
    env_var: &'static str,
    config_key: &'static str,
    hint: &'static str,
}

const API_PROVIDERS: &[ApiProvider] = &[
    ApiProvider {
        name: "OpenRouter",
        env_var: "OPENROUTER_API_KEY",
        config_key: "openrouter_key",
        hint: "openrouter.ai/keys",
    },
    ApiProvider {
        name: "Groq",
        env_var: "GROQ_API_KEY",
        config_key: "groq_key",
        hint: "console.groq.com",
    },
    ApiProvider {
        name: "Cerebras",
        env_var: "CEREBRAS_API_KEY",
        config_key: "cerebras_key",
        hint: "cloud.cerebras.ai",
    },
];

fn resolve_provider_key(key_env: &str) -> Option<String> {
    std::env::var(key_env)
        .ok()
        .or_else(|| std::env::var("OPENZAX_API_KEY").ok())
        .or_else(|| {
            let cfg = load_openzax_config();
            let config_key = match key_env {
                "OPENROUTER_API_KEY" => "openrouter_key",
                "GROQ_API_KEY" => "groq_key",
                "CEREBRAS_API_KEY" => "cerebras_key",
                _ => return None,
            };
            cfg[config_key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| {
                    cfg["api_key"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
        })
}

fn mask_key(key: &str) -> String {
    if key.len() <= 10 {
        return "****".to_string();
    }
    format!("{}...{}", &key[..5], &key[key.len() - 4..])
}

// ─── Command palette ─────────────────────────────────────────────────────────

struct CmdEntry {
    label: &'static str,
    shortcut: &'static str,
    cat: &'static str,
}

const CMD_PALETTE: &[CmdEntry] = &[
    CmdEntry {
        label: "Switch model",
        shortcut: "Ctrl+M",
        cat: "Model",
    },
    CmdEntry {
        label: "Intelligence tier",
        shortcut: "Ctrl+T",
        cat: "Model",
    },
    CmdEntry {
        label: "API keys",
        shortcut: "/connect",
        cat: "Model",
    },
    CmdEntry {
        label: "Switch mode",
        shortcut: "Tab",
        cat: "Session",
    },
    CmdEntry {
        label: "New session",
        shortcut: "Ctrl+N",
        cat: "Session",
    },
    CmdEntry {
        label: "Skills",
        shortcut: "Ctrl+K",
        cat: "Tools",
    },
    CmdEntry {
        label: "Help",
        shortcut: "/help",
        cat: "System",
    },
    CmdEntry {
        label: "Exit",
        shortcut: "Ctrl+C",
        cat: "System",
    },
];

// ─── Skills ──────────────────────────────────────────────────────────────────

struct SkillEntry {
    name: &'static str,
    desc: &'static str,
}

const SKILLS: &[SkillEntry] = &[
    SkillEntry {
        name: "webapp-testing",
        desc: "Test and interact with web applications",
    },
    SkillEntry {
        name: "frontend-design",
        desc: "Production-grade frontend interfaces",
    },
    SkillEntry {
        name: "docker-expert",
        desc: "Docker containerization & orchestration",
    },
    SkillEntry {
        name: "e2e-testing-patterns",
        desc: "E2E testing with Playwright & Cypress",
    },
    SkillEntry {
        name: "python-testing-patterns",
        desc: "Comprehensive testing with pytest",
    },
    SkillEntry {
        name: "python-design-patterns",
        desc: "KISS, SoC, SRP design patterns",
    },
    SkillEntry {
        name: "async-python-patterns",
        desc: "Asyncio & concurrent programming",
    },
    SkillEntry {
        name: "javascript-testing",
        desc: "JS/TS testing with Jest & Vitest",
    },
    SkillEntry {
        name: "docker-best-practices",
        desc: "Production Docker deployments",
    },
    SkillEntry {
        name: "database-migration",
        desc: "DB migrations across ORMs",
    },
    SkillEntry {
        name: "prisma-database-setup",
        desc: "Configure Prisma with any DB",
    },
    SkillEntry {
        name: "database-schema-designer",
        desc: "Scalable database schema design",
    },
    SkillEntry {
        name: "rust-systems",
        desc: "Advanced Rust system patterns",
    },
    SkillEntry {
        name: "security-audit",
        desc: "Security auditing for code & deps",
    },
    SkillEntry {
        name: "api-design-patterns",
        desc: "REST & GraphQL API design",
    },
    SkillEntry {
        name: "ci-cd-pipelines",
        desc: "CI/CD pipeline optimization",
    },
    SkillEntry {
        name: "kubernetes-expert",
        desc: "Kubernetes orchestration",
    },
    SkillEntry {
        name: "vercel-react",
        desc: "React/Next.js performance",
    },
    SkillEntry {
        name: "python-performance",
        desc: "Profile & optimize Python",
    },
    SkillEntry {
        name: "find-skills",
        desc: "Discover & install new skills",
    },
];

// ─── System prompts ──────────────────────────────────────────────────────────

const BUILD_PROMPT: &str = "You are OpenZax, an elite AI coding assistant with full access to the user's filesystem and shell. You can read/write files, create directories, delete files, move files, and execute shell commands using your tools. When the user asks to work on files or run commands, USE YOUR TOOLS to do it directly — do not just describe how to do it. Write production-ready code with no shortcuts. Handle all edge cases. Follow SOLID, clean architecture, DRY. Use best practices for the language/framework. Be concise. If complex, break into steps and execute each fully.";

const PLAN_PROMPT: &str = "You are OpenZax in Planning Mode. PLAN before code. For every request: 1) Requirements analysis 2) Architecture design 3) Implementation plan 4) Risk matrix. Never write implementation code.";

// ─── Config persistence ───────────────────────────────────────────────────────

fn config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".openzax").join("config.json"))
}

fn load_openzax_config() -> serde_json::Value {
    if let Some(path) = config_path() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                return v;
            }
        }
    }
    serde_json::json!({})
}

fn save_openzax_config(config: &serde_json::Value) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            &path,
            serde_json::to_string_pretty(config).unwrap_or_default(),
        );
    }
}

// ─── Auto-update ─────────────────────────────────────────────────────────────

async fn check_and_auto_update() {
    let current = env!("CARGO_PKG_VERSION");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .user_agent(format!("openzax-cli/{}", current))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let resp = match client
        .get("https://api.github.com/repos/zAxCoder/OpenZax/releases/latest")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return,
    };

    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(_) => return,
    };

    let latest = match data["tag_name"].as_str() {
        Some(t) => t.trim_start_matches('v').to_string(),
        None => return,
    };

    if latest == current {
        return;
    }

    println!();
    println!("  Updating OpenZax v{} → v{}...", current, latest);

    match run_installer(&client).await {
        Ok(true) => {
            println!("  Update complete! Restarting...");
            println!();
            if let Ok(exe) = std::env::current_exe() {
                let args: Vec<String> = std::env::args().skip(1).collect();
                let _ = std::process::Command::new(&exe).args(&args).status();
            }
            std::process::exit(0);
        }
        Ok(false) => {
            println!("  Update failed. Continuing with current version.");
        }
        Err(_) => {}
    }
}

async fn run_installer(client: &reqwest::Client) -> anyhow::Result<bool> {
    #[cfg(windows)]
    {
        let url = "https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1";
        let content = client.get(url).send().await?.text().await?;
        let temp = std::env::temp_dir().join("openzax-update.ps1");
        std::fs::write(&temp, &content)?;
        let status = std::process::Command::new("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                temp.to_str().unwrap_or(""),
            ])
            .status()?;
        let _ = std::fs::remove_file(&temp);
        Ok(status.success())
    }

    #[cfg(not(windows))]
    {
        let url = "https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.sh";
        let content = client.get(url).send().await?.text().await?;
        let temp = std::env::temp_dir().join("openzax-update.sh");
        std::fs::write(&temp, &content)?;
        let status = std::process::Command::new("bash")
            .arg(temp.to_str().unwrap_or(""))
            .status()?;
        let _ = std::fs::remove_file(&temp);
        Ok(status.success())
    }
}

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum Msg {
    User(String),
    Assistant(String),
    System(String),
    Status { model: String, secs: f32 },
}

#[derive(PartialEq, Copy, Clone)]
enum Overlay {
    None,
    Commands,
    Skills,
    Models,
    Connect,
}

#[derive(PartialEq, Copy, Clone)]
enum Mode {
    Build,
    Plan,
}

#[derive(PartialEq, Copy, Clone)]
enum Phase {
    Empty,
    Chat,
    Stream,
}

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
    connect_input: String,
    connect_editing: bool,
    stream_buf: Arc<Mutex<String>>,
    done_flag: Arc<Mutex<bool>>,
}

impl App {
    pub fn new(model: &str) -> Self {
        let short = model
            .split('/')
            .next_back()
            .unwrap_or(model)
            .trim_end_matches(":free")
            .to_string();
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
            connect_input: String::new(),
            connect_editing: false,
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

    fn ins(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.reset_cursor();
    }
    fn bksp(&mut self) {
        if self.cursor > 0 {
            let p = self.input[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.remove(p);
            self.cursor = p;
            self.reset_cursor();
        }
    }
    fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.input[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.reset_cursor();
        }
    }
    fn right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor = self.input[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.input.len());
            self.reset_cursor();
        }
    }
    fn take(&mut self) -> String {
        let s = std::mem::take(&mut self.input);
        self.cursor = 0;
        self.reset_cursor();
        s
    }
    fn sup(&mut self) {
        self.scroll = self.scroll.saturating_sub(3);
    }
    fn sdn(&mut self) {
        self.scroll += 3;
    }
    fn bot(&mut self) {
        self.scroll = usize::MAX;
    }
    fn push(&mut self, m: Msg) {
        self.msgs.push(m);
        self.bot();
    }
    fn flush(&mut self) {
        let c = {
            let mut b = self.stream_buf.lock().unwrap();
            let s = b.clone();
            b.clear();
            s
        };
        if c.is_empty() {
            return;
        }
        self.session_tokens += (c.len() / 4) as u32;
        if let Some(Msg::Assistant(ref mut b)) = self.msgs.last_mut() {
            b.push_str(&c);
        } else {
            self.msgs.push(Msg::Assistant(c));
        }
        self.bot();
    }
    fn done(&mut self) -> bool {
        let d = *self.done_flag.lock().unwrap();
        if d {
            *self.done_flag.lock().unwrap() = false;
        }
        d
    }
    fn secs(&self) -> f32 {
        self.session_start.elapsed().as_secs_f32()
    }
    fn tier(&self) -> &'static str {
        TIERS[self.tier_idx]
    }
    fn mode_label(&self) -> &str {
        match self.mode {
            Mode::Build => "Build",
            Mode::Plan => "Plan",
        }
    }
    fn ov_count(&self) -> usize {
        match self.overlay {
            Overlay::Commands => CMD_PALETTE
                .iter()
                .filter(|e| {
                    self.ov_search.is_empty()
                        || e.label
                            .to_lowercase()
                            .contains(&self.ov_search.to_lowercase())
                })
                .count(),
            Overlay::Skills => SKILLS
                .iter()
                .filter(|s| {
                    self.ov_search.is_empty()
                        || s.name.contains(&self.ov_search)
                        || s.desc
                            .to_lowercase()
                            .contains(&self.ov_search.to_lowercase())
                })
                .count(),
            Overlay::Models => FREE_MODELS.len(),
            Overlay::Connect => API_PROVIDERS.len(),
            Overlay::None => 0,
        }
    }
}

// ─── Input height helper ──────────────────────────────────────────────────────

fn input_height(app: &App) -> u16 {
    let lines = app.input.split('\n').count().max(3);
    ((lines + 2) as u16).min(10)
}

// Helper to detect Ctrl+letter
fn is_ctrl(key: &crossterm::event::KeyEvent, ch: char) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(ch) {
        return true;
    }
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
        Overlay::Connect => draw_connect(f, app),
        Overlay::None => {}
    }
}

fn draw_empty(f: &mut Frame, app: &App) {
    let a = f.area();
    let brand_h = BRAND_OPEN.len() as u16;
    let ih = input_height(app);
    let content_h = brand_h + 2 + ih + 2 + 1 + 2 + 1;
    let top = a.height.saturating_sub(content_h) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top),
            Constraint::Length(brand_h),
            Constraint::Length(2),
            Constraint::Length(ih),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(a);

    // Two-tone brand logo
    let mut bl: Vec<Line> = Vec::new();
    for i in 0..BRAND_OPEN.len() {
        bl.push(Line::from(vec![
            Span::styled(
                BRAND_OPEN[i],
                Style::default()
                    .fg(ACCENT_BLUE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                BRAND_ZAX[i],
                Style::default()
                    .fg(ACCENT_GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(bl)
            .alignment(Alignment::Center)
            .style(Style::default().bg(BG)),
        chunks[1],
    );

    // Input box
    let ic = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[3]);
    draw_input(f, app, ic[1]);

    // Mode + model + tier
    let ml = Line::from(vec![
        Span::styled(
            app.mode_label(),
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(G4)),
        Span::styled(app.model_short.as_str(), Style::default().fg(G2)),
        Span::styled("  ·  ", Style::default().fg(G4)),
        Span::styled(app.tier(), Style::default().fg(ACCENT_GOLD)),
    ]);
    f.render_widget(
        Paragraph::new(ml)
            .alignment(Alignment::Center)
            .style(Style::default().bg(BG)),
        chunks[4],
    );

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
    f.render_widget(
        Paragraph::new(sc)
            .alignment(Alignment::Center)
            .style(Style::default().bg(BG)),
        chunks[6],
    );

    // Bottom: CWD (left) + version (right)
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let ver_str = format!("OpenZax {}", VERSION);
    let gap = (a.width as usize).saturating_sub(cwd.len() + ver_str.len() + 4);
    let bottom = Line::from(vec![
        Span::styled(format!(" {}", cwd), Style::default().fg(G3)),
        Span::styled(" ".repeat(gap), Style::default()),
        Span::styled(format!("{} ", ver_str), Style::default().fg(G3)),
    ]);
    f.render_widget(
        Paragraph::new(bottom).style(Style::default().bg(BG)),
        chunks[9],
    );
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if app.phase == Phase::Stream { G3 } else { G2 }))
        .style(Style::default().bg(BG_INPUT));
    let inner = blk.inner(area);
    f.render_widget(blk, area);

    let cursor_char = if app.cursor_visible { "\u{2588}" } else { " " };

    let mut all_lines: Vec<Line> = Vec::new();

    if app.input.is_empty() {
        all_lines.push(Line::from(vec![
            Span::styled(" > ", Style::default().fg(G1).add_modifier(Modifier::BOLD)),
            Span::styled(cursor_char, Style::default().fg(W)),
            Span::styled("Ask anything...  ", Style::default().fg(G4)),
            Span::styled("\"Fix broken tests\"", Style::default().fg(G3)),
        ]));
    } else {
        let before = &app.input[..app.cursor];
        let after = if app.cursor < app.input.len() {
            &app.input[app.cursor..]
        } else {
            ""
        };

        let before_parts: Vec<&str> = before.split('\n').collect();
        let after_parts: Vec<&str> = after.split('\n').collect();
        let cursor_line_idx = before_parts.len() - 1;
        let total_lines = before_parts.len() + after_parts.len() - 1;

        for i in 0..total_lines {
            let prefix = if i == 0 { " > " } else { "   " };

            if i < cursor_line_idx {
                // Lines before the cursor line
                all_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(G1).add_modifier(Modifier::BOLD)),
                    Span::styled(before_parts[i].to_string(), Style::default().fg(W)),
                ]));
            } else if i == cursor_line_idx {
                // The cursor line
                let before_on_line = before_parts[cursor_line_idx];
                let after_on_line = after_parts.first().copied().unwrap_or("");
                all_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(G1).add_modifier(Modifier::BOLD)),
                    Span::styled(before_on_line.to_string(), Style::default().fg(W)),
                    Span::styled(cursor_char.to_string(), Style::default().fg(W)),
                    Span::styled(after_on_line.to_string(), Style::default().fg(W)),
                ]));
            } else {
                // Lines after the cursor
                let after_idx = i - cursor_line_idx;
                if after_idx < after_parts.len() {
                    all_lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(G1).add_modifier(Modifier::BOLD)),
                        Span::styled(after_parts[after_idx].to_string(), Style::default().fg(W)),
                    ]));
                }
            }
        }
    }

    f.render_widget(
        Paragraph::new(all_lines).style(Style::default().bg(BG_INPUT)),
        inner,
    );
}

fn draw_chat(f: &mut Frame, app: &mut App) {
    let a = f.area();
    let ih = input_height(app);
    let bottom_h = 1 + ih + 1; // info(1) + input + shortcuts(1)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(32)])
        .split(a);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(bottom_h)])
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
                // Handle multi-line user messages
                for (i, part) in t.split('\n').enumerate() {
                    let prefix = if i == 0 { " > " } else { "   " };
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(W).add_modifier(Modifier::BOLD)),
                        Span::styled(
                            part.to_string(),
                            Style::default().fg(W).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
                lines.push(Line::default());
            }
            Msg::Assistant(t) => {
                for wr in wrap(t, w) {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(wr, Style::default().fg(G1)),
                    ]));
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
    if app.scroll >= usize::MAX / 2 {
        app.scroll = mx;
    } else {
        app.scroll = app.scroll.min(mx);
    }
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .style(Style::default().bg(BG))
            .scroll((app.scroll as u16, 0)),
        area,
    );
}

fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let ih = input_height(app);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(ih),
            Constraint::Length(1),
        ])
        .split(area);

    let info = Line::from(vec![
        Span::styled(
            format!(" {} ", app.mode_label()),
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ),
        Span::styled(". ", Style::default().fg(G4)),
        Span::styled(app.tier(), Style::default().fg(G2)),
        Span::styled(format!("  {}  ", app.model_short), Style::default().fg(G3)),
        Span::styled(format!("  . OpenZax {}", VERSION), Style::default().fg(G4)),
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
        Span::styled("S+Enter ", Style::default().fg(G2)),
        Span::styled("newline  ", Style::default().fg(G4)),
        Span::styled("Esc ", Style::default().fg(G2)),
        Span::styled("cancel", Style::default().fg(G4)),
    ]);
    f.render_widget(Paragraph::new(sc).style(Style::default().bg(BG)), rows[2]);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(
        Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(G4))
            .style(Style::default().bg(BG_PANEL)),
        area,
    );
    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let title = app
        .msgs
        .iter()
        .find_map(|m| {
            if let Msg::User(t) = m {
                Some(t.chars().take(20).collect::<String>())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "New session".into());

    let cwd = std::env::current_dir()
        .map(|p| {
            let s = p.display().to_string();
            if s.len() > 26 {
                format!("...{}", &s[s.len() - 23..])
            } else {
                s
            }
        })
        .unwrap_or_default();

    let l = vec![
        Line::from(Span::styled(
            &title,
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Context",
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} tokens", app.session_tokens),
            Style::default().fg(G3),
        )),
        Line::from(Span::styled(
            format!("  {:.0}s elapsed", app.secs()),
            Style::default().fg(G3),
        )),
        Line::from(Span::styled("  $0.00 (free)", Style::default().fg(G2))),
        Line::default(),
        Line::from(Span::styled(
            "Model",
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", app.model_short),
            Style::default().fg(W),
        )),
        Line::from(Span::styled(
            format!("  {}", app.model_provider),
            Style::default().fg(G3),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Session",
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} . {}", app.mode_label(), app.tier()),
            Style::default().fg(G3),
        )),
        Line::from(Span::styled(format!("  {}", cwd), Style::default().fg(G3))),
        Line::default(),
        Line::from(Span::styled(
            "Free API keys",
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  openrouter.ai/keys",
            Style::default().fg(G1),
        )),
        Line::from(Span::styled("  console.groq.com", Style::default().fg(G1))),
        Line::from(Span::styled("  cloud.cerebras.ai", Style::default().fg(G1))),
    ];
    f.render_widget(
        Paragraph::new(Text::from(l))
            .style(Style::default().bg(BG_PANEL))
            .wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Overlays (with Clear widget for solid background) ───────────────────────

fn popup_rect(area: Rect, w: u16, h: u16) -> Rect {
    let pw = w.min(area.width.saturating_sub(4));
    let ph = h.min(area.height.saturating_sub(2));
    Rect::new(
        (area.width.saturating_sub(pw)) / 2,
        (area.height.saturating_sub(ph)) / 2,
        pw,
        ph,
    )
}

fn draw_connect(f: &mut Frame, app: &mut App) {
    let edit_extra = if app.connect_editing { 4 } else { 0 };
    let h = (API_PROVIDERS.len() as u16) + 7 + edit_extra;
    let popup = popup_rect(f.area(), 56, h);
    app.ov_rect = popup;
    f.render_widget(Clear, popup);
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(G3))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(
            " API Keys ",
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let cfg = load_openzax_config();
    let iw = inner.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " Manage your provider API keys",
        Style::default().fg(G2),
    )));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(iw),
        Style::default().fg(G4),
    )));

    for (i, p) in API_PROVIDERS.iter().enumerate() {
        let sel = i == app.ov_idx;
        let (fg, bg_c) = if sel { (BLK, BG_SEL) } else { (G1, BG_POPUP) };
        let key_val = std::env::var(p.env_var).ok().or_else(|| {
            cfg[p.config_key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });
        let (status, sfg) = match &key_val {
            Some(k) => (
                format!("{}  [set]", mask_key(k)),
                if sel { BLK } else { ACCENT_BLUE },
            ),
            None => (format!("not set  {}", p.hint), if sel { G3 } else { G4 }),
        };
        lines.push(Line::from(vec![
            Span::styled(
                if sel { " > " } else { "   " },
                Style::default().fg(fg).bg(bg_c),
            ),
            Span::styled(format!("{:<14}", p.name), Style::default().fg(fg).bg(bg_c)),
            Span::styled(status, Style::default().fg(sfg).bg(bg_c)),
        ]));
    }

    if app.connect_editing {
        lines.push(Line::from(Span::styled(
            "\u{2500}".repeat(iw),
            Style::default().fg(G4),
        )));
        let pname = API_PROVIDERS.get(app.ov_idx).map(|p| p.name).unwrap_or("?");
        lines.push(Line::from(Span::styled(
            format!(" Key for {}:", pname),
            Style::default().fg(ACCENT_BLUE),
        )));
        let cursor_ch = if app.cursor_visible { "\u{2588}" } else { " " };
        let placeholder = if app.connect_input.is_empty() {
            Span::styled(
                "Paste key here (Ctrl+V)...",
                Style::default().fg(G3).bg(BG_INPUT),
            )
        } else {
            let masked: String = if app.connect_input.len() > 12 {
                format!(
                    "{}...{}",
                    &app.connect_input[..5],
                    &app.connect_input[app.connect_input.len() - 4..]
                )
            } else {
                app.connect_input.clone()
            };
            Span::styled(masked, Style::default().fg(ACCENT_BLUE).bg(BG_INPUT))
        };
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default().bg(BG_INPUT)),
            placeholder,
            Span::styled(cursor_ch, Style::default().fg(ACCENT_GOLD).bg(BG_INPUT)),
            Span::styled(
                " ".repeat(iw.saturating_sub(14)),
                Style::default().bg(BG_INPUT),
            ),
        ]));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(iw),
        Style::default().fg(G4),
    )));
    if app.connect_editing {
        lines.push(Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(G2)),
            Span::styled("save  ", Style::default().fg(G4)),
            Span::styled("Ctrl+V ", Style::default().fg(ACCENT_GOLD)),
            Span::styled("paste  ", Style::default().fg(G4)),
            Span::styled("Esc ", Style::default().fg(G2)),
            Span::styled("cancel", Style::default().fg(G4)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(G2)),
            Span::styled("edit  ", Style::default().fg(G4)),
            Span::styled("Del ", Style::default().fg(G2)),
            Span::styled("remove  ", Style::default().fg(G4)),
            Span::styled("Esc ", Style::default().fg(G2)),
            Span::styled("close", Style::default().fg(G4)),
        ]));
    }
    f.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)),
        inner,
    );
}

fn draw_commands(f: &mut Frame, app: &mut App) {
    let popup = popup_rect(f.area(), 52, 20);
    app.ov_rect = popup;
    // Clear the area first so background content doesn't show through
    f.render_widget(Clear, popup);
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(G3))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(
            " Commands ",
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let sl = if app.ov_search.is_empty() {
        Line::from(Span::styled(" Search...", Style::default().fg(G3)))
    } else {
        Line::from(Span::styled(
            format!(" {}", app.ov_search),
            Style::default().fg(W),
        ))
    };
    f.render_widget(
        Paragraph::new(sl).style(Style::default().bg(BG_INPUT)),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(rows[1].width as usize),
            Style::default().fg(G4),
        )))
        .style(Style::default().bg(BG_POPUP)),
        rows[1],
    );

    app.ov_item_y = rows[2].y;
    let filtered: Vec<&CmdEntry> = CMD_PALETTE
        .iter()
        .filter(|e| {
            app.ov_search.is_empty()
                || e.label
                    .to_lowercase()
                    .contains(&app.ov_search.to_lowercase())
        })
        .collect();
    let mut lines: Vec<Line> = Vec::new();
    let mut last = "";
    for (gi, entry) in filtered.iter().enumerate() {
        if entry.cat != last {
            lines.push(Line::from(
                Span::styled(
                    format!(" {}", entry.cat),
                    Style::default().fg(G2).add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(BG_POPUP)),
            ));
            last = entry.cat;
        }
        let sel = gi == app.ov_idx;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G1, BG_POPUP) };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<26}", entry.label),
                Style::default().fg(fg).bg(bg),
            ),
            Span::styled(
                format!("{:>14}", entry.shortcut),
                Style::default().fg(if sel { G3 } else { G4 }).bg(bg),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)),
        rows[2],
    );
}

fn draw_skills(f: &mut Frame, app: &mut App) {
    let popup = popup_rect(f.area(), 64, 26);
    app.ov_rect = popup;
    f.render_widget(Clear, popup);
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(G3))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(
            " Skills ",
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let sl = if app.ov_search.is_empty() {
        Line::from(Span::styled(" Search skills...", Style::default().fg(G3)))
    } else {
        Line::from(Span::styled(
            format!(" {}", app.ov_search),
            Style::default().fg(W),
        ))
    };
    f.render_widget(
        Paragraph::new(sl).style(Style::default().bg(BG_INPUT)),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(rows[1].width as usize),
            Style::default().fg(G4),
        )))
        .style(Style::default().bg(BG_POPUP)),
        rows[1],
    );

    app.ov_item_y = rows[2].y;
    let nc = 24usize;
    let dc = (inner.width as usize).saturating_sub(nc + 4);
    let filtered: Vec<&SkillEntry> = SKILLS
        .iter()
        .filter(|s| {
            app.ov_search.is_empty()
                || s.name.contains(&app.ov_search)
                || s.desc
                    .to_lowercase()
                    .contains(&app.ov_search.to_lowercase())
        })
        .collect();
    let mut lines: Vec<Line> = Vec::new();
    for (i, e) in filtered.iter().enumerate() {
        let sel = i == app.ov_idx;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G2, BG_POPUP) };
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "  {:<w$}",
                    e.name.chars().take(nc).collect::<String>(),
                    w = nc
                ),
                Style::default().fg(if sel { BLK } else { W }).bg(bg),
            ),
            Span::styled(
                e.desc.chars().take(dc).collect::<String>(),
                Style::default().fg(fg).bg(bg),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)),
        rows[2],
    );
}

fn draw_models(f: &mut Frame, app: &mut App) {
    let h = (FREE_MODELS.len() as u16) + 6;
    let popup = popup_rect(f.area(), 58, h);
    app.ov_rect = popup;
    f.render_widget(Clear, popup);
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(G3))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(
            " Switch Model ",
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ));
    let inner = blk.inner(popup);
    f.render_widget(blk, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " Free models (no credit card)",
            Style::default().fg(G2).add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(BG_POPUP)),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(rows[1].width as usize),
            Style::default().fg(G4),
        )))
        .style(Style::default().bg(BG_POPUP)),
        rows[1],
    );

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
            Span::styled(
                format!("{:>6}", m.ctx),
                Style::default().fg(if sel { G3 } else { G4 }).bg(bg),
            ),
            Span::styled(
                format!("  {:<10}", m.provider),
                Style::default().fg(if sel { G2 } else { G3 }).bg(bg),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(BG_POPUP)),
        rows[2],
    );
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut cur = String::new();
        let mut cl = 0;
        for w in raw.split_whitespace() {
            let wl = w.chars().count();
            if cl == 0 {
                cur.push_str(w);
                cl = wl;
            } else if cl + 1 + wl <= width {
                cur.push(' ');
                cur.push_str(w);
                cl += 1 + wl;
            } else {
                out.push(std::mem::take(&mut cur));
                cur.push_str(w);
                cl = wl;
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

fn handle_slash(app: &mut App, cmd: &str) -> bool {
    match cmd.trim() {
        "/help" | "/h" => {
            app.push(Msg::System("Tab mode · Ctrl+T tier · Ctrl+P cmds · Ctrl+M model · Ctrl+K skills · Ctrl+N new · Shift+Enter newline · /exit quit".into()));
            true
        }
        "/clear" | "/new" => {
            app.msgs.clear();
            app.phase = Phase::Empty;
            app.session_tokens = 0;
            app.session_start = Instant::now();
            true
        }
        "/model" => {
            let info = format!("{} ({})", app.model_name, app.model_provider);
            app.push(Msg::System(info));
            true
        }
        "/exit" | "/quit" | "/q" => {
            app.push(Msg::System("__EXIT__".into()));
            true
        }
        "/connect" | "/keys" | "/api" => {
            app.overlay = Overlay::Connect;
            app.ov_idx = 0;
            app.connect_editing = false;
            app.connect_input.clear();
            true
        }
        _ => false,
    }
}

// ─── Entry ───────────────────────────────────────────────────────────────────

pub async fn run_tui(
    model_name: String,
    api_key: Option<String>,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    // Auto-update check (before entering TUI alternate screen)
    tokio::select! {
        _ = check_and_auto_update() => {}
        _ = tokio::time::sleep(std::time::Duration::from_secs(4)) => {}
    }

    // Set up Ctrl+C signal handler
    let _ = ctrlc::set_handler(move || {
        EXIT_FLAG.store(true, Ordering::SeqCst);
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = main_loop(&mut terminal, model_name, api_key, db_path).await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res
}

async fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model_name: String,
    api_key: Option<String>,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    // Load persisted config
    let saved_config = load_openzax_config();

    // Resolve API key: arg > env > config file
    let key = api_key
        .clone()
        .or_else(|| std::env::var("OPENZAX_API_KEY").ok())
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .or_else(|| {
            saved_config["api_key"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });

    // If API key was provided via --api-key flag, save it
    if api_key.is_some() {
        if let Some(ref k) = key {
            let mut cfg = load_openzax_config();
            cfg["api_key"] = serde_json::json!(k);
            save_openzax_config(&cfg);
        }
    }

    // Resolve model: use saved model if the passed model is the default
    let default_model = "deepseek/deepseek-r1-0528:free";
    let resolved_model = if model_name == default_model {
        saved_config["model"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or(model_name.clone())
    } else {
        model_name.clone()
    };

    // Find matching free model entry for correct API URL + provider + key env
    let initial_fm = FREE_MODELS.iter().find(|m| m.id == resolved_model);
    let (initial_api_url, initial_provider, initial_key_env) = initial_fm
        .map(|m| {
            (
                m.api_url.to_string(),
                m.provider.to_string(),
                m.key_env.to_string(),
            )
        })
        .unwrap_or_else(|| {
            (
                "https://openrouter.ai/api/v1/chat/completions".to_string(),
                "OpenRouter".to_string(),
                "OPENROUTER_API_KEY".to_string(),
            )
        });

    // Re-resolve key using per-provider config if initial resolution failed
    let key = key.or_else(|| resolve_provider_key(&initial_key_env));

    let mut app = App::new(&resolved_model);
    app.model_api = initial_api_url.clone();
    app.model_provider = initial_provider;

    if key.is_none() {
        app.pending_sys
            .push("No API key. Get free keys (no credit card):".into());
        app.pending_sys
            .push("  openrouter.ai/keys  ·  console.groq.com  ·  cloud.cerebras.ai".into());
        app.pending_sys
            .push("Use /connect to add your key, or set OPENROUTER_API_KEY env var".into());
    }

    if let Some(p) = db_path.parent() {
        std::fs::create_dir_all(p).ok();
    }
    let eb = EventBus::default();
    let cfg = AgentConfig {
        api_url: initial_api_url,
        api_key: key.clone(),
        model: resolved_model.clone(),
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
                    OzEvent::AgentTokenStream { token, .. } => {
                        buf.lock().unwrap().push_str(&token);
                    }
                    OzEvent::AgentOutput { .. } => {
                        *df.lock().unwrap() = true;
                    }
                    _ => {}
                }
            }
        });
    }

    loop {
        // Check Ctrl+C signal flag
        if EXIT_FLAG.load(Ordering::SeqCst) {
            break;
        }

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
        if !event::poll(Duration::from_millis(40))? {
            continue;
        }

        match event::read()? {
            Event::Mouse(me) => {
                if app.overlay != Overlay::None {
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let (mx, my) = (me.column, me.row);
                            if mx < app.ov_rect.x
                                || mx >= app.ov_rect.x + app.ov_rect.width
                                || my < app.ov_rect.y
                                || my >= app.ov_rect.y + app.ov_rect.height
                            {
                                app.overlay = Overlay::None;
                                app.ov_search.clear();
                                continue;
                            }
                            if my >= app.ov_item_y {
                                let clicked = (my - app.ov_item_y) as usize;
                                let actual = if app.overlay == Overlay::Commands {
                                    resolve_cmd_click(&app, clicked)
                                } else {
                                    let max = app.ov_count();
                                    if clicked < max {
                                        Some(clicked)
                                    } else {
                                        None
                                    }
                                };
                                if let Some(idx) = actual {
                                    app.ov_idx = idx;
                                    execute_overlay(&mut app, &agent);
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if app.ov_idx > 0 {
                                app.ov_idx -= 1;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            let mx = app.ov_count();
                            if app.ov_idx + 1 < mx {
                                app.ov_idx += 1;
                            }
                        }
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
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Ctrl+C always exits
                if is_ctrl(&key, 'c') {
                    break;
                }

                // Overlay input
                if app.overlay != Overlay::None {
                    if app.overlay == Overlay::Connect {
                        if app.connect_editing {
                            match key.code {
                                KeyCode::Esc => {
                                    app.connect_editing = false;
                                    app.connect_input.clear();
                                }
                                KeyCode::Enter => {
                                    let trimmed = app.connect_input.trim().to_string();
                                    if !trimmed.is_empty() {
                                        if let Some(p) = API_PROVIDERS.get(app.ov_idx) {
                                            let mut cfg = load_openzax_config();
                                            cfg[p.config_key] = serde_json::json!(&trimmed);
                                            // Also save as generic fallback key if OpenRouter
                                            if p.config_key == "openrouter_key" {
                                                cfg["api_key"] = serde_json::json!(&trimmed);
                                            }
                                            save_openzax_config(&cfg);
                                            // Update active agent key if provider matches
                                            if app.model_provider == p.name {
                                                agent.set_api_key(trimmed.clone());
                                            }
                                            app.push(Msg::System(format!(
                                                "API key saved for {} - ready to use",
                                                p.name
                                            )));
                                        }
                                    }
                                    app.connect_editing = false;
                                    app.connect_input.clear();
                                }
                                KeyCode::Backspace => {
                                    app.connect_input.pop();
                                }
                                KeyCode::Char('v')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    if let Some(text) = clipboard_paste() {
                                        app.connect_input.push_str(text.trim());
                                    }
                                }
                                KeyCode::Char(c)
                                    if !key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.connect_input.push(c);
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Esc => {
                                    app.overlay = Overlay::None;
                                }
                                KeyCode::Up => {
                                    if app.ov_idx > 0 {
                                        app.ov_idx -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if app.ov_idx + 1 < API_PROVIDERS.len() {
                                        app.ov_idx += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    app.connect_editing = true;
                                    app.connect_input.clear();
                                }
                                KeyCode::Delete => {
                                    if let Some(p) = API_PROVIDERS.get(app.ov_idx) {
                                        let mut cfg = load_openzax_config();
                                        if let Some(obj) = cfg.as_object_mut() {
                                            obj.remove(p.config_key);
                                        }
                                        save_openzax_config(&cfg);
                                        app.push(Msg::System(format!(
                                            "API key removed for {}",
                                            p.name
                                        )));
                                    }
                                }
                                _ => {}
                            }
                        }
                        continue;
                    }
                    match key.code {
                        KeyCode::Esc => {
                            app.overlay = Overlay::None;
                            app.ov_search.clear();
                        }
                        KeyCode::Up => {
                            if app.ov_idx > 0 {
                                app.ov_idx -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let mx = app.ov_count();
                            if app.ov_idx + 1 < mx {
                                app.ov_idx += 1;
                            }
                        }
                        KeyCode::Enter => execute_overlay(&mut app, &agent),
                        KeyCode::Backspace => {
                            app.ov_search.pop();
                            app.ov_idx = 0;
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.ov_search.push(c);
                            app.ov_idx = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Ctrl shortcuts
                if is_ctrl(&key, 'p') {
                    app.overlay = Overlay::Commands;
                    app.ov_idx = 0;
                    app.ov_search.clear();
                    continue;
                }
                if is_ctrl(&key, 'm') {
                    app.overlay = Overlay::Models;
                    app.ov_idx = 0;
                    app.ov_search.clear();
                    continue;
                }
                if is_ctrl(&key, 'k') {
                    app.overlay = Overlay::Skills;
                    app.ov_idx = 0;
                    app.ov_search.clear();
                    continue;
                }
                if is_ctrl(&key, 'n') {
                    app.msgs.clear();
                    app.phase = Phase::Empty;
                    app.session_tokens = 0;
                    app.session_start = Instant::now();
                    continue;
                }
                if is_ctrl(&key, 't') {
                    app.tier_idx = (app.tier_idx + 1) % TIERS.len();
                    if app.phase != Phase::Empty {
                        app.push(Msg::System(format!("Tier: {}", TIERS[app.tier_idx])));
                    }
                    continue;
                }

                // Ctrl+V: paste from clipboard into input
                if is_ctrl(&key, 'v') {
                    if let Some(text) = clipboard_paste() {
                        for c in text.chars() {
                            if c != '\r' {
                                app.ins(c);
                            }
                        }
                    }
                    continue;
                }

                // Skip other Ctrl combos
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    continue;
                }

                if key.code == KeyCode::Tab && app.phase != Phase::Stream {
                    app.mode = match app.mode {
                        Mode::Build => Mode::Plan,
                        Mode::Plan => Mode::Build,
                    };
                    agent.set_system_prompt(
                        match app.mode {
                            Mode::Build => BUILD_PROMPT,
                            Mode::Plan => PLAN_PROMPT,
                        }
                        .to_string(),
                    );
                    if app.phase != Phase::Empty {
                        app.push(Msg::System(format!("Mode: {}", app.mode_label())));
                    }
                    continue;
                }

                if app.phase == Phase::Stream {
                    match key.code {
                        KeyCode::Up => app.sup(),
                        KeyCode::Down => app.sdn(),
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Enter => {
                        // Shift+Enter inserts a newline
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.ins('\n');
                            continue;
                        }
                        // Regular Enter submits
                        let text = app.take();
                        if text.trim().is_empty() {
                            continue;
                        }
                        if text.trim().starts_with('/') {
                            handle_slash(&mut app, text.trim());
                            if app
                                .msgs
                                .last()
                                .map(|m| matches!(m, Msg::System(s) if s == "__EXIT__"))
                                .unwrap_or(false)
                            {
                                break;
                            }
                            continue;
                        }
                        if app.phase == Phase::Empty {
                            app.phase = Phase::Chat;
                            app.session_start = Instant::now();
                            for s in std::mem::take(&mut app.pending_sys) {
                                app.push(Msg::System(s));
                            }
                        }
                        app.push(Msg::User(text.clone()));
                        storage
                            .save_message(Uuid::new_v4(), conv_id, "user", &text)
                            .ok();
                        eb.publish(OzEvent::UserInput {
                            session_id: conv_id,
                            content: text.clone(),
                            attachments: vec![],
                            timestamp: Utc::now(),
                        })
                        .ok();
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
                    KeyCode::Down => app.ins('\n'),
                    KeyCode::Home => {
                        app.cursor = 0;
                        app.reset_cursor();
                    }
                    KeyCode::End => {
                        app.cursor = app.input.len();
                        app.reset_cursor();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_cmd_click(app: &App, clicked: usize) -> Option<usize> {
    let filtered: Vec<&CmdEntry> = CMD_PALETTE
        .iter()
        .filter(|e| {
            app.ov_search.is_empty()
                || e.label
                    .to_lowercase()
                    .contains(&app.ov_search.to_lowercase())
        })
        .collect();
    let mut line = 0usize;
    let mut last = "";
    for (gi, e) in filtered.iter().enumerate() {
        if e.cat != last {
            if line == clicked {
                return None;
            }
            line += 1;
            last = e.cat;
        }
        if line == clicked {
            return Some(gi);
        }
        line += 1;
    }
    None
}

fn execute_overlay(app: &mut App, agent: &Arc<Agent>) {
    match app.overlay {
        Overlay::Commands => {
            let filtered: Vec<&CmdEntry> = CMD_PALETTE
                .iter()
                .filter(|e| {
                    app.ov_search.is_empty()
                        || e.label
                            .to_lowercase()
                            .contains(&app.ov_search.to_lowercase())
                })
                .collect();
            if let Some(e) = filtered.get(app.ov_idx) {
                match e.label {
                    "Exit" => {
                        app.push(Msg::System("__EXIT__".into()));
                    }
                    "New session" => {
                        app.msgs.clear();
                        app.phase = Phase::Empty;
                        app.session_tokens = 0;
                        app.session_start = Instant::now();
                    }
                    "Switch model" => {
                        app.overlay = Overlay::Models;
                        app.ov_idx = 0;
                        app.ov_search.clear();
                        return;
                    }
                    "Skills" => {
                        app.overlay = Overlay::Skills;
                        app.ov_idx = 0;
                        app.ov_search.clear();
                        return;
                    }
                    "API keys" => {
                        app.overlay = Overlay::Connect;
                        app.ov_idx = 0;
                        app.connect_editing = false;
                        app.connect_input.clear();
                        return;
                    }
                    "Switch mode" => {
                        app.mode = match app.mode {
                            Mode::Build => Mode::Plan,
                            Mode::Plan => Mode::Build,
                        };
                        agent.set_system_prompt(
                            match app.mode {
                                Mode::Build => BUILD_PROMPT,
                                Mode::Plan => PLAN_PROMPT,
                            }
                            .to_string(),
                        );
                        app.push(Msg::System(format!("Mode: {}", app.mode_label())));
                    }
                    "Intelligence tier" => {
                        app.tier_idx = (app.tier_idx + 1) % TIERS.len();
                        app.push(Msg::System(format!("Tier: {}", TIERS[app.tier_idx])));
                    }
                    "Help" => {
                        app.push(Msg::System("Tab mode · Ctrl+T tier · Ctrl+P cmds · Ctrl+M model · Shift+Enter newline".into()));
                    }
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

                // Update agent with correct model + API URL
                agent.set_model(m.id.to_string());
                agent.set_api_url(m.api_url.to_string());

                let provider_key = resolve_provider_key(m.key_env);
                if let Some(k) = provider_key {
                    agent.set_api_key(k);
                    app.push(Msg::System(format!(
                        "Model: {} ({})",
                        m.display, m.provider
                    )));
                } else {
                    app.push(Msg::System(format!(
                        "Model: {} ({}) -- no API key, use /connect",
                        m.display, m.provider
                    )));
                }

                // Persist selected model
                let mut config = load_openzax_config();
                config["model"] = serde_json::json!(m.id);
                save_openzax_config(&config);
            }
        }
        Overlay::Skills => {
            let filtered: Vec<&SkillEntry> = SKILLS
                .iter()
                .filter(|s| {
                    app.ov_search.is_empty()
                        || s.name.contains(&app.ov_search)
                        || s.desc
                            .to_lowercase()
                            .contains(&app.ov_search.to_lowercase())
                })
                .collect();
            if let Some(s) = filtered.get(app.ov_idx) {
                app.push(Msg::System(format!("Skill activated: {}", s.name)));
            }
        }
        Overlay::Connect => {}
        Overlay::None => {}
    }
    app.overlay = Overlay::None;
    app.ov_search.clear();
}
