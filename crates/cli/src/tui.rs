use chrono::Utc;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
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

const ACCENT: Color = Color::Rgb(200, 60, 60);
const ACCENT_DIM: Color = Color::Rgb(140, 35, 35);
const ACCENT_BRIGHT: Color = Color::Rgb(255, 70, 70);
const ACCENT_BLUE: Color = Color::Rgb(100, 180, 255);
const ACCENT_GOLD: Color = Color::Rgb(255, 180, 60);
const VERSION: &str = env!("CARGO_PKG_VERSION");

const BRAND_OPEN_GRAD: [Color; 5] = [
    Color::Rgb(245, 240, 240),
    Color::Rgb(210, 170, 170),
    Color::Rgb(180, 110, 110),
    Color::Rgb(155, 65, 65),
    Color::Rgb(130, 30, 30),
];
const BRAND_ZAX_COLOR: Color = Color::Rgb(180, 15, 15);

// ─── Intelligence tiers ──────────────────────────────────────────────────────

const TIERS: &[&str] = &["auto", "high", "max"];

// ─── Free models ─────────────────────────────────────────────────────────────

pub struct FreeModel {
    pub id: &'static str,
    pub display: &'static str,
    pub ctx: &'static str,
    pub provider: &'static str,
    pub api_url: &'static str,
    pub key_env: &'static str,
    pub category: &'static str,
    pub strength: &'static str,
}

const FREE_MODELS: &[FreeModel] = &[
    // ── Tier 1: Elite text models ────────────────────────────────────────
    FreeModel {
        id: "nousresearch/hermes-3-llama-3.1-405b:free",
        display: "Hermes 3 405B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Largest free model. Elite general knowledge, creative writing, long-form content, complex reasoning, and nuanced conversation",
    },
    FreeModel {
        id: "openai/gpt-oss-120b:free",
        display: "GPT-OSS 120B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "OpenAI open-source 120B. Excellent full-stack coding, strong reasoning, great instruction following and structured output",
    },
    FreeModel {
        id: "qwen/qwen3-235b-a22b-thinking-2507",
        display: "Qwen3 235B Think",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Massive 235B thinking model. Best for complex math, deep analysis, multi-step reasoning, algorithm design, and architecture planning",
    },
    FreeModel {
        id: "qwen/qwen3-vl-235b-a22b-thinking",
        display: "Qwen3 VL 235B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "vision",
        strength: "235B vision+thinking. Best for image understanding combined with deep reasoning, visual code review, UI/UX analysis, and diagram interpretation",
    },
    FreeModel {
        id: "qwen/qwen3-next-80b-a3b-instruct:free",
        display: "Qwen3 Next 80B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "80B MoE free model. Strong balanced coding and reasoning, excellent instruction following, good for backend development and system design",
    },
    FreeModel {
        id: "qwen/qwen3-coder:free",
        display: "Qwen3 Coder",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Dedicated coding model. Best for code generation, debugging, refactoring, full-stack development (frontend + backend), code review, and writing tests",
    },
    FreeModel {
        id: "deepseek/deepseek-r1-0528:free",
        display: "DeepSeek R1",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Deep reasoning specialist. Excellent at math, logic puzzles, step-by-step problem solving, and complex code debugging",
    },
    // ── Tier 2: Strong text models ───────────────────────────────────────
    FreeModel {
        id: "meta-llama/llama-3.3-70b-instruct:free",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Proven 70B model. Strong general coding, API development, backend systems, documentation writing, and reliable structured output",
    },
    FreeModel {
        id: "qwen/qwen3-235b-a22b:free",
        display: "Qwen3 235B",
        ctx: "40K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "235B general model. Strong reasoning, coding, and multilingual capabilities. Good for complex analysis and technical writing",
    },
    FreeModel {
        id: "sourceful/riverflow-v2-max-preview",
        display: "Riverflow V2 Max",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Maximum quality writing model. Best for marketing copy, creative content, storytelling, blog posts, and persuasive natural language",
    },
    FreeModel {
        id: "sourceful/riverflow-v2-pro",
        display: "Riverflow V2 Pro",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Professional writing model. Great for content creation, email drafting, social media posts, product descriptions, and conversational AI",
    },
    FreeModel {
        id: "arcee-ai/trinity-large-preview:free",
        display: "Trinity Large",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Strong reasoning and analysis. Good for research tasks, data analysis, technical writing, and balanced coding",
    },
    FreeModel {
        id: "qwen/qwen3-vl-30b-a3b-thinking",
        display: "Qwen3 VL 30B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "vision",
        strength: "30B vision+thinking. Good for image analysis, screenshot debugging, chart interpretation, OCR tasks, and visual QA",
    },
    // ── Tier 3: Good text models ─────────────────────────────────────────
    FreeModel {
        id: "nvidia/nemotron-3-nano-30b-a3b:free",
        display: "Nemotron 30B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Efficient 30B model. Good for general coding, quick reasoning tasks, and balanced instruction following",
    },
    FreeModel {
        id: "openai/gpt-oss-20b:free",
        display: "GPT-OSS 20B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "OpenAI 20B. Fast coding and quick tasks, good for simple scripts, utility functions, and rapid prototyping",
    },
    FreeModel {
        id: "stepfun/step-3.5-flash:free",
        display: "Step 3.5 Flash",
        ctx: "256K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Fast with huge 256K context. Best for processing very long documents, large codebases, and extensive chat histories",
    },
    FreeModel {
        id: "z-ai/glm-4.5-air:free",
        display: "GLM 4.5 Air",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Multilingual model. Strong Chinese/English bilingual, good for translation, multilingual content, and general conversation",
    },
    FreeModel {
        id: "sourceful/riverflow-v2-standard-preview",
        display: "Riverflow Standard",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Balanced writing model. Good for everyday content creation, summarization, and natural conversational responses",
    },
    // ── Tier 4: Mid-range text models ────────────────────────────────────
    FreeModel {
        id: "google/gemma-3-27b-it:free",
        display: "Gemma 3 27B",
        ctx: "96K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Google 27B. Good for frontend development, HTML/CSS/JS, React components, and general web development tasks",
    },
    FreeModel {
        id: "mistralai/mistral-small-3.1-24b-instruct:free",
        display: "Mistral Small 3.1",
        ctx: "96K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Efficient 24B instruct. Good for structured output, JSON generation, API responses, and concise technical answers",
    },
    FreeModel {
        id: "cognitivecomputations/dolphin-mistral-24b-venice-edition:free",
        display: "Dolphin 24B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Uncensored 24B model. Best for unrestricted creative writing, roleplay, brainstorming, and open-ended exploration",
    },
    FreeModel {
        id: "nvidia/nemotron-nano-12b-v2-vl:free",
        display: "Nemotron 12B VL",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "vision",
        strength: "12B vision model. Efficient image understanding, screenshot analysis, document parsing, and visual question answering",
    },
    FreeModel {
        id: "google/gemma-3-12b-it:free",
        display: "Gemma 3 12B",
        ctx: "96K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Google 12B. Solid for quick coding tasks, simple scripts, CSS styling, and lightweight web development",
    },
    // ── Tier 5: Light / fast text models ─────────────────────────────────
    FreeModel {
        id: "nvidia/nemotron-nano-9b-v2:free",
        display: "Nemotron 9B",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Fast 9B model. Quick utility scripts, simple Q&A, basic coding, and rapid iteration tasks",
    },
    FreeModel {
        id: "sourceful/riverflow-v2-fast",
        display: "Riverflow V2 Fast",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Fast writing model. Quick drafts, short-form content, social media captions, and rapid content iteration",
    },
    FreeModel {
        id: "sourceful/riverflow-v2-fast-preview",
        display: "Riverflow Fast Pre",
        ctx: "128K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Fast writing preview. Quick content generation, short answers, and rapid prototyping of written content",
    },
    FreeModel {
        id: "google/gemma-3n-e4b-it:free",
        display: "Gemma 3N E4B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Efficient 4B model. Quick simple tasks, basic code snippets, short answers, and lightweight assistance",
    },
    FreeModel {
        id: "qwen/qwen3-4b:free",
        display: "Qwen3 4B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Small fast model. Simple coding, quick translations, basic Q&A, and rapid completions",
    },
    FreeModel {
        id: "google/gemma-3-4b-it:free",
        display: "Gemma 3 4B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Google 4B. Quick simple tasks, basic HTML/CSS, short code snippets, and lightweight chat",
    },
    FreeModel {
        id: "google/gemma-3n-e2b-it:free",
        display: "Gemma 3N E2B",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Tiny 2B model. Ultra-fast simple completions, basic formatting, and lightweight assistance",
    },
    FreeModel {
        id: "arcee-ai/trinity-mini:free",
        display: "Trinity Mini",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Small reasoning model. Quick analysis, simple logic, and fast concise answers",
    },
    FreeModel {
        id: "liquid/lfm-2.5-1.2b-thinking:free",
        display: "LFM 1.2B Think",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Tiny thinking model. Basic step-by-step reasoning at minimal cost, simple math, and quick logic puzzles",
    },
    FreeModel {
        id: "liquid/lfm-2.5-1.2b-instruct:free",
        display: "LFM 1.2B Instruct",
        ctx: "32K",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        key_env: "OPENROUTER_API_KEY",
        category: "text",
        strength: "Tiny instruct model. Ultra-fast simple instructions, basic formatting, and minimal latency tasks",
    },
    // ── Image generation models ──────────────────────────────────────────
    FreeModel {
        id: "black-forest-labs/flux.2-max",
        display: "FLUX.2 Max",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/images/generations",
        key_env: "OPENROUTER_API_KEY",
        category: "image",
        strength: "Best quality image generation. Photorealistic images, detailed art, complex scenes, and professional-grade visuals",
    },
    FreeModel {
        id: "black-forest-labs/flux.2-pro",
        display: "FLUX.2 Pro",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/images/generations",
        key_env: "OPENROUTER_API_KEY",
        category: "image",
        strength: "Professional image generation. High-quality photos, product images, marketing visuals, and detailed illustrations",
    },
    FreeModel {
        id: "bytedance-seed/seedream-4.5",
        display: "SeeDream 4.5",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/images/generations",
        key_env: "OPENROUTER_API_KEY",
        category: "image",
        strength: "Creative image generation. Artistic styles, imaginative scenes, anime/illustration, and diverse visual aesthetics",
    },
    FreeModel {
        id: "black-forest-labs/flux.2-flex",
        display: "FLUX.2 Flex",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/images/generations",
        key_env: "OPENROUTER_API_KEY",
        category: "image",
        strength: "Flexible style image generation. Style mixing, artistic control, custom aesthetics, and versatile visual output",
    },
    FreeModel {
        id: "black-forest-labs/flux.2-klein-4b",
        display: "FLUX.2 Klein",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/images/generations",
        key_env: "OPENROUTER_API_KEY",
        category: "image",
        strength: "Fast lightweight image generation. Quick thumbnails, icons, simple graphics, and rapid visual prototyping",
    },
    // ── Embedding model ──────────────────────────────────────────────────
    FreeModel {
        id: "nvidia/llama-nemotron-embed-vl-1b-v2:free",
        display: "Nemotron Embed VL",
        ctx: "-",
        provider: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/embeddings",
        key_env: "OPENROUTER_API_KEY",
        category: "embedding",
        strength: "Vision-language embedding model. Text and image embeddings for semantic search, similarity matching, and retrieval",
    },
    // ── Groq models (ultra-fast inference) ───────────────────────────────
    FreeModel {
        id: "llama-3.3-70b-versatile",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
        category: "text",
        strength: "Ultra-fast Groq inference. Same Llama 3.3 70B but with blazing speed, great for rapid coding and quick iteration",
    },
    FreeModel {
        id: "llama-3.1-8b-instant",
        display: "Llama 3.1 8B",
        ctx: "128K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
        category: "text",
        strength: "Ultra-fast small model on Groq. Instant responses for simple tasks, basic coding, and rapid prototyping",
    },
    FreeModel {
        id: "gemma2-9b-it",
        display: "Gemma 2 9B",
        ctx: "8K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
        category: "text",
        strength: "Fast 9B on Groq. Quick code snippets, simple tasks, and instant lightweight assistance",
    },
    FreeModel {
        id: "mixtral-8x7b-32768",
        display: "Mixtral 8x7B",
        ctx: "32K",
        provider: "Groq",
        api_url: "https://api.groq.com/openai/v1/chat/completions",
        key_env: "GROQ_API_KEY",
        category: "text",
        strength: "MoE model on Groq. Good balance of speed and quality for coding, writing, and general tasks",
    },
    // ── Cerebras models (fastest inference) ──────────────────────────────
    FreeModel {
        id: "llama-3.3-70b",
        display: "Llama 3.3 70B",
        ctx: "128K",
        provider: "Cerebras",
        api_url: "https://api.cerebras.ai/v1/chat/completions",
        key_env: "CEREBRAS_API_KEY",
        category: "text",
        strength: "Fastest inference provider. Llama 3.3 70B at record speed, ideal for rapid development and real-time coding",
    },
    FreeModel {
        id: "qwen-3-32b",
        display: "Qwen3 32B",
        ctx: "32K",
        provider: "Cerebras",
        api_url: "https://api.cerebras.ai/v1/chat/completions",
        key_env: "CEREBRAS_API_KEY",
        category: "text",
        strength: "Fast 32B on Cerebras. Quick reasoning, decent coding, and rapid general assistance",
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

const BUILD_PROMPT: &str = r#"You are OpenZax — an autonomous AI software engineer. You BUILD working software using your tools. You do NOT just describe or explain — you CREATE files, WRITE code, and RUN commands.

## HOW TO BUILD A PROJECT (Follow this EVERY time):

### Step 1: Create project structure
Use create_directory to make all folders FIRST:
```
create_directory("my-project/src")
create_directory("my-project/public")
```

### Step 2: Write EVERY file with COMPLETE code
Use write_file for EACH file. CRITICAL: write the ENTIRE file content — every import, every function, every line. NEVER write empty files. NEVER use placeholder comments like "// add code here" or "TODO".

Example of CORRECT file writing:
```
write_file("my-project/index.html", "<!DOCTYPE html>\n<html>... [FULL 100+ lines of real HTML] ...</html>")
```

Example of WRONG (never do this):
```
write_file("my-project/index.html", "<!-- Add your HTML here -->")  // WRONG - empty placeholder
write_file("my-project/app.js", "// TODO: implement")  // WRONG - no real code
```

### Step 3: Write dependency/config files
Always create: package.json, requirements.txt, Cargo.toml, or whatever the project needs. Include ALL dependencies with versions.

### Step 4: Verify
Use execute_command to build/run/test:
```
execute_command("cd my-project && npm install && npm start")
```

### Step 5: Fix any errors
If verification fails, read the error, fix the code, and try again.

## ABSOLUTE RULES:
1. EVERY file you create MUST have COMPLETE, REAL, WORKING code inside
2. When asked to "create a website" — write the FULL HTML, CSS, and JS. All of it. Hundreds of lines if needed
3. When asked to "build an API" — write the FULL server, routes, middleware, models, error handling
4. NEVER say "I'll create..." and then write empty files. Write the ACTUAL code
5. NEVER truncate code with "..." or "// rest of code". Write ALL of it
6. If a file should be 300 lines, write all 300 lines
7. Use your tools in EVERY response — read_file, write_file, execute_command
8. After creating files, ALWAYS run execute_command to verify they work
9. If the user asks you to do something, DO IT with tools — don't just explain how
10. Remember the user's preferences with remember_user when they share personal details

## Your Tools:
- read_file(path) — read any file
- write_file(path, content) — create/write files with COMPLETE content
- list_directory(path) — list files
- execute_command(command) — run ANY shell command
- create_directory(path) — make directories
- delete_file(path) — remove files/dirs
- move_file(source, destination) — move/rename
- search_files(pattern) — find files by name pattern
- search_text(pattern) — search file contents
- spawn_agent(task, model) — delegate subtask to a sub-agent
- remember_user(key, value) — save user's personal details permanently

You are the engineer. Take the request. Use your tools. Build it. Verify it. Ship it."#;

const PLAN_PROMPT: &str = r#"You are OpenZax in Strategic Planning Mode — a world-class software architect and technical strategist with decades of engineering wisdom condensed into pure analytical power.

## Your Mission
You PLAN before anyone codes. You are the architect, the strategist, the risk assessor. You see the entire system from 10,000 feet while understanding every implementation detail at ground level.

## Planning Protocol — For EVERY request:

### 1. Requirements Analysis
- Break the request to atomic, testable requirements
- Identify implicit requirements not explicitly stated
- Define constraints: performance, compatibility, security, scalability
- Establish clear acceptance criteria

### 2. Architecture Design
- System components and their precise responsibilities
- Data flow, state management, and component communication
- API contracts, interfaces, and type definitions
- Technology choices with clear justification
- Trade-offs analyzed — present the optimal path with alternatives

### 3. Implementation Roadmap
- Step-by-step plan with clear dependencies and ordering
- Complexity estimate for each step (trivial / moderate / complex / critical)
- Critical path identification — what blocks what
- Parallel work streams where tasks are independent

### 4. Risk Matrix
- Technical risks: what could fail, what's fragile, what's unknown
- Mitigation strategy for every identified risk
- Rollback plan if implementation goes wrong
- Edge cases that MUST be handled — enumerate them

## Rules
- NEVER write implementation code — only pseudocode, specs, and diagrams
- Be opinionated — recommend the BEST approach, not all approaches
- Think about the team: clarity, maintainability, and knowledge transfer
- Think about the future: will this design accommodate likely changes?
- Every plan should be so clear that any competent engineer can execute it perfectly

You are the architect. Design systems that outlast their creators."#;

const AGENT_PROMPT_TEMPLATE: &str = r#"You are OpenZax in Multi-Agent Command Mode. You coordinate multiple AI agents using spawn_agent to build complex projects.

## CRITICAL: SMART MODEL ROUTING
You CANNOT run the same model twice simultaneously. You MUST assign a DIFFERENT model to each sub-agent.
Choose the BEST model for each task based on its strengths. ALWAYS specify the "model" parameter in spawn_agent.

## AVAILABLE MODELS CATALOG:
{MODEL_CATALOG}

## HOW TO CHOOSE THE RIGHT MODEL:
1. **Coding tasks (backend/API/systems)**: Use Qwen3 Coder, GPT-OSS 120B, Qwen3 Next 80B, or DeepSeek R1
2. **Coding tasks (frontend/UI/CSS)**: Use Gemma 3 27B, GPT-OSS 120B, or Qwen3 Coder
3. **Creative writing / marketing / content**: Use Riverflow V2 Max, Riverflow V2 Pro, or Hermes 3 405B
4. **Deep reasoning / math / analysis**: Use Qwen3 235B Think, DeepSeek R1, or Qwen3 235B
5. **Image understanding / visual tasks**: Use Qwen3 VL 235B, Qwen3 VL 30B, or Nemotron 12B VL
6. **Quick simple tasks / utilities**: Use Step 3.5 Flash, Qwen3 4B, or any lightweight model
7. **Multilingual / translation**: Use GLM 4.5 Air, Hermes 3 405B, or Qwen3 235B
8. **Uncensored / creative exploration**: Use Dolphin 24B

## HOW MULTI-AGENT WORKS:

### Step 1: Analyze & Decompose
Break the project into 2-5 independent tasks. For each task, identify:
- What TYPE of task it is (coding, writing, design, analysis, etc.)
- Which specific skill is needed (frontend, backend, marketing, etc.)
- Which model is BEST suited based on the catalog above

### Step 2: Delegate with spawn_agent — ALWAYS specify model!
For EACH task, call spawn_agent with a detailed instruction AND the best model for that task.
NEVER use your own model (the one you're currently running on) for sub-agents.

GOOD examples:
```
spawn_agent({
  "task": "Create file 'src/server.js' with a complete Express.js server...",
  "model": "qwen/qwen3-coder:free"
})
spawn_agent({
  "task": "Write compelling marketing copy for the landing page...",
  "model": "sourceful/riverflow-v2-max-preview"
})
spawn_agent({
  "task": "Analyze this algorithm's complexity and optimize it...",
  "model": "deepseek/deepseek-r1-0528:free"
})
```

BAD examples (NEVER do this):
```
spawn_agent({"task": "Create the backend"})  // NO model specified + too vague
spawn_agent({"task": "Write HTML", "model": "SAME_AS_YOURS"})  // Same model = will fail
```

### Step 3: Review Results
After each agent completes, check the result. If failed, spawn a new agent with a DIFFERENT model.

### Step 4: Integration
After all agents complete, verify everything works together.

## CRITICAL RULES:
1. ALWAYS specify "model" in spawn_agent — pick from the catalog above
2. NEVER use the same model for two simultaneous agents
3. NEVER use your own model for sub-agents
4. Match task type to model strength (coding model for code, writing model for content, etc.)
5. Tell the agent EXACTLY which files to create with COMPLETE content
6. Tell the agent to VERIFY its work with execute_command

## Your Tools:
- spawn_agent(task, model) — delegate with the RIGHT model for the task
- read_file, write_file, execute_command — for verification and integration
- search_files, search_text — to check agent output
- remember_user(key, value) — save user's personal details permanently

## Workflow:
1. Read the user's request carefully
2. Break into clear, detailed subtasks
3. For EACH subtask, pick the BEST model from the catalog
4. spawn_agent for each subtask with exhaustive instructions + specific model
5. Review each result — if failed, retry with a different model
6. Verify the complete project works
7. Report final status to the user

You are the commander. Route tasks intelligently. Demand excellence."#;

const HIGH_BOOST: &str = "\n\n## Enhanced Focus\nThink step by step. Double-check every decision. Consider edge cases carefully. Optimize for correctness and robustness. Show your reasoning.";

const MAX_BOOST: &str = r#"

## MAXIMUM PERFORMANCE MODE ENGAGED
You are operating at ABSOLUTE PEAK capability. This is your finest work.
- Analyze from MULTIPLE angles before responding — consider at least 3 approaches
- Pick the OPTIMAL solution with clear justification
- Verify your logic step-by-step — zero assumptions, zero shortcuts
- Your code must be FLAWLESS — production-ready, mentally tested on every path
- Handle EVERY edge case, EVERY error path, EVERY failure mode — no exceptions
- Choose optimal algorithms and data structures — performance is critical
- Security: validate everything, trust nothing, sanitize all boundaries
- Think like a principal engineer at the world's best tech company
- Your output should be indistinguishable from a 10x engineer with 20 years experience
Take a deep breath. Focus completely. Deliver your masterpiece."#;

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
    // Temporarily disabled to prevent infinite loop until release is ready
    return;

    #[allow(unreachable_code)]
    {
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
                println!("  Update complete!");
                println!();
                println!("  Please restart OpenZax to use the new version:");
                println!("    Close this window and run 'openzax' again");
                println!();
                std::thread::sleep(std::time::Duration::from_secs(3));
                std::process::exit(0);
            }
            Ok(false) => {
                println!("  Update failed. Continuing with current version.");
            }
            Err(_) => {}
        }
    } // End of unreachable block
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
    MultiAgent,
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
    agent_ref: Option<Arc<Agent>>,
    paste_cooldown: Instant,
    update_available: Arc<Mutex<Option<String>>>,
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
            agent_ref: None,
            paste_cooldown: Instant::now(),
            update_available: Arc::new(Mutex::new(None)),
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
            let mut b = self.stream_buf.lock().unwrap_or_else(|p| p.into_inner());
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
        let d = *self.done_flag.lock().unwrap_or_else(|p| p.into_inner());
        if d {
            *self.done_flag.lock().unwrap_or_else(|p| p.into_inner()) = false;
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
            Mode::MultiAgent => "Multi Agent",
        }
    }
    fn mode_prompt(&self) -> String {
        let base = match self.mode {
            Mode::Build => BUILD_PROMPT.to_string(),
            Mode::Plan => PLAN_PROMPT.to_string(),
            Mode::MultiAgent => {
                let mut catalog = String::new();
                for m in FREE_MODELS
                    .iter()
                    .filter(|m| m.category == "text" || m.category == "vision")
                {
                    catalog.push_str(&format!(
                        "- **{}** (`{}`): {}\n",
                        m.display, m.id, m.strength
                    ));
                }
                catalog.push_str("\n### Image Generation Models:\n");
                for m in FREE_MODELS.iter().filter(|m| m.category == "image") {
                    catalog.push_str(&format!(
                        "- **{}** (`{}`): {}\n",
                        m.display, m.id, m.strength
                    ));
                }
                let current_model = &self.model_name;
                let prompt = AGENT_PROMPT_TEMPLATE.replace("{MODEL_CATALOG}", &catalog);
                format!("{}\n\n## YOUR CURRENT MODEL: `{}`\nNEVER assign this model to sub-agents. Always pick a DIFFERENT model.", prompt, current_model)
            }
        };
        let boost = match TIERS[self.tier_idx] {
            "high" => HIGH_BOOST,
            "max" => MAX_BOOST,
            _ => "",
        };
        format!("{}{}", base, boost)
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

fn input_height_with_width(app: &App, width: u16) -> u16 {
    let usable = (width as usize).saturating_sub(6);
    if usable == 0 || app.input.is_empty() {
        return 5;
    }
    let mut total_lines = 0usize;
    for line in app.input.split('\n') {
        let char_count = line.chars().count();
        if char_count == 0 {
            total_lines += 1;
        } else {
            total_lines += (char_count + usable - 1) / usable;
        }
    }
    ((total_lines.max(3) + 2) as u16).min(12)
}

// Helper to detect Ctrl+letter
fn is_ctrl(key: &crossterm::event::KeyEvent, ch: char) -> bool {
    let m = key.modifiers;
    if m.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            if c == ch || c == ch.to_ascii_uppercase() {
                return true;
            }
        }
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
    let ih = input_height_with_width(app, a.width);

    let has_update = app
        .update_available
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .is_some();
    let update_h: u16 = if has_update { 3 } else { 0 };

    let content_h = brand_h + 2 + ih + 2 + 1 + 2 + 1 + update_h;
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
            Constraint::Length(update_h),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(a);

    let mut bl: Vec<Line> = Vec::new();
    for i in 0..BRAND_OPEN.len() {
        bl.push(Line::from(vec![
            Span::styled(
                BRAND_OPEN[i],
                Style::default()
                    .fg(BRAND_OPEN_GRAD[i])
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                BRAND_ZAX[i],
                Style::default()
                    .fg(BRAND_ZAX_COLOR)
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
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(chunks[3]);
    draw_input(f, app, ic[1]);

    let ml = Line::from(vec![
        Span::styled(
            app.mode_label(),
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(G4)),
        Span::styled(app.model_short.as_str(), Style::default().fg(G2)),
        Span::styled("  ·  ", Style::default().fg(G4)),
        Span::styled(app.tier(), Style::default().fg(ACCENT_BRIGHT)),
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

    // Update notification
    if let Some(new_ver) = app.update_available.lock().ok().and_then(|g| g.clone()) {
        let update_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " ★ ",
                    Style::default()
                        .fg(ACCENT_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("Update available: v{} → {}  ", VERSION, new_ver),
                    Style::default().fg(ACCENT_GOLD),
                ),
                Span::styled(
                    "Run: openzax upgrade",
                    Style::default()
                        .fg(ACCENT_BLUE)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(update_lines)
                .alignment(Alignment::Center)
                .style(Style::default().bg(BG)),
            chunks[8],
        );
    }

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
        chunks[10],
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
    let max_w = inner.width.saturating_sub(4) as usize;

    let mut all_lines: Vec<Line> = Vec::new();

    if app.input.is_empty() {
        all_lines.push(Line::from(vec![
            Span::styled(" > ", Style::default().fg(G1).add_modifier(Modifier::BOLD)),
            Span::styled(cursor_char, Style::default().fg(W)),
            Span::styled("Ask anything...  ", Style::default().fg(G4)),
            Span::styled("\"Fix broken tests\"", Style::default().fg(G3)),
        ]));
    } else if max_w == 0 {
        all_lines.push(Line::from(Span::styled(&app.input, Style::default().fg(W))));
    } else {
        let full: String = {
            let before: String = app.input[..app.cursor].to_string();
            let after: String = if app.cursor < app.input.len() {
                app.input[app.cursor..].to_string()
            } else {
                String::new()
            };
            format!("{}\x00{}", before, after)
        };

        let mut first_line = true;
        for logical_line in full.split('\n') {
            let chars: Vec<char> = logical_line.chars().collect();
            if chars.is_empty() {
                let prefix = if first_line { " > " } else { "   " };
                first_line = false;
                all_lines.push(Line::from(Span::styled(
                    prefix,
                    Style::default().fg(G1).add_modifier(Modifier::BOLD),
                )));
                continue;
            }
            let mut pos = 0;
            while pos < chars.len() {
                let end = (pos + max_w).min(chars.len());
                let chunk: String = chars[pos..end].iter().collect();
                let prefix = if first_line { " > " } else { "   " };
                first_line = false;

                if let Some(cursor_pos) = chunk.find('\x00') {
                    let before_cur = &chunk[..cursor_pos];
                    let after_cur = &chunk[cursor_pos + 1..];
                    all_lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(G1).add_modifier(Modifier::BOLD)),
                        Span::styled(before_cur.to_string(), Style::default().fg(W)),
                        Span::styled(cursor_char.to_string(), Style::default().fg(W)),
                        Span::styled(after_cur.to_string(), Style::default().fg(W)),
                    ]));
                } else {
                    all_lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(G1).add_modifier(Modifier::BOLD)),
                        Span::styled(chunk.replace('\x00', ""), Style::default().fg(W)),
                    ]));
                }
                pos = end;
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
    let chat_input_w = a.width.saturating_sub(32 + 4);
    let ih = input_height_with_width(app, chat_input_w);
    let bottom_h = 1 + ih + 1;
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
    if let Some(ref ag) = app.agent_ref {
        draw_sidebar(f, app, cols[1], ag);
    }
}

fn draw_messages(f: &mut Frame, app: &mut App, area: Rect) {
    let w = area.width.saturating_sub(4) as usize;
    let ml = app.mode_label().to_string();
    let mut lines: Vec<Line> = Vec::new();

    let user_bg = Color::Rgb(20, 20, 25);
    let thinking_dots = match (app.session_start.elapsed().as_millis() / 400) % 4 {
        0 => "   ",
        1 => ".  ",
        2 => ".. ",
        _ => "...",
    };

    for (mi, msg) in app.msgs.iter().enumerate() {
        match msg {
            Msg::User(t) => {
                lines.push(Line::default());
                let wrapped_user = wrap(t, w.saturating_sub(3));
                for (i, wl) in wrapped_user.iter().enumerate() {
                    let prefix = if i == 0 { " ? " } else { "   " };
                    let pad = " ".repeat(w.saturating_sub(wl.chars().count() + 3));
                    lines.push(Line::from(vec![
                        Span::styled(
                            prefix,
                            Style::default()
                                .fg(ACCENT_BRIGHT)
                                .bg(user_bg)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            wl.to_string(),
                            Style::default()
                                .fg(W)
                                .bg(user_bg)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(pad, Style::default().bg(user_bg)),
                    ]));
                }
                lines.push(Line::default());
            }
            Msg::Assistant(t) => {
                let is_last = mi == app.msgs.len() - 1;
                let is_streaming = is_last && app.phase == Phase::Stream;
                if t.is_empty() && is_streaming {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(
                            format!("Thinking{}", thinking_dots),
                            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                } else {
                    for wr in wrap(t, w) {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(wr, Style::default().fg(G1)),
                        ]));
                    }
                    if is_streaming {
                        lines.push(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled("▍", Style::default().fg(ACCENT)),
                        ]));
                    }
                }
                if !is_streaming {
                    lines.push(Line::default());
                }
            }
            Msg::Status { model, secs } => {
                lines.push(Line::from(vec![
                    Span::styled("  ✓ ", Style::default().fg(ACCENT_DIM)),
                    Span::styled(ml.as_str(), Style::default().fg(G2)),
                    Span::styled(" · ", Style::default().fg(G4)),
                    Span::styled(model.as_str(), Style::default().fg(G3)),
                    Span::styled(format!(" · {:.1}s", secs), Style::default().fg(G4)),
                ]));
                lines.push(Line::default());
            }
            Msg::System(t) => {
                if t != "__EXIT__" {
                    lines.push(Line::from(vec![
                        Span::styled("  · ", Style::default().fg(G4)),
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
    let ih = input_height_with_width(app, area.width);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(ih),
            Constraint::Length(1),
        ])
        .split(area);

    let mode_bg = match app.mode {
        Mode::Build => ACCENT,
        Mode::Plan => Color::Rgb(60, 130, 200),
        Mode::MultiAgent => Color::Rgb(180, 120, 30),
    };
    let streaming = app.phase == Phase::Stream;

    let info = Line::from(vec![
        Span::styled(
            format!(" {} ", app.mode_label()),
            Style::default()
                .fg(W)
                .bg(mode_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(app.tier(), Style::default().fg(ACCENT_BRIGHT)),
        Span::styled(format!("  {}  ", app.model_short), Style::default().fg(G3)),
        Span::styled(format!("  · OpenZax {}", VERSION), Style::default().fg(G4)),
    ]);
    f.render_widget(Paragraph::new(info).style(Style::default().bg(BG)), rows[0]);
    draw_input(f, app, rows[1]);

    let sc = if streaming {
        let dots = match (app.session_start.elapsed().as_millis() / 500) % 4 {
            0 => "   ",
            1 => ".  ",
            2 => ".. ",
            _ => "...",
        };
        Line::from(vec![
            Span::styled(
                format!(" working{} ", dots),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("│ ", Style::default().fg(G4)),
            Span::styled("Esc ", Style::default().fg(G2)),
            Span::styled("cancel", Style::default().fg(G4)),
        ])
    } else {
        Line::from(vec![
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
        ])
    };
    f.render_widget(Paragraph::new(sc).style(Style::default().bg(BG)), rows[2]);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect, agent: &Agent) {
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

    let mut l = vec![
        Line::from(Span::styled(
            &title,
            Style::default().fg(W).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Context",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {:.0}s elapsed", app.secs()),
            Style::default().fg(G3),
        )),
        Line::from(Span::styled("  $0.00 (free)", Style::default().fg(G2))),
        Line::default(),
        Line::from(Span::styled(
            "Model",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
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
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} · {}", app.mode_label(), app.tier()),
            Style::default().fg(G3),
        )),
        Line::from(Span::styled(format!("  {}", cwd), Style::default().fg(G3))),
    ];

    let subs = agent.sub_agent_statuses();
    if !subs.is_empty() {
        l.push(Line::default());
        let done_count = subs.iter().filter(|s| s.done).count();
        l.push(Line::from(Span::styled(
            format!("Agents ({}/{})", done_count, subs.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        for (i, s) in subs.iter().enumerate() {
            let icon = if s.done { "✓" } else { "…" };
            let color = if s.done {
                Color::Rgb(80, 180, 80)
            } else {
                ACCENT_BRIGHT
            };
            let task_display: String = s.task.chars().take(22).collect();
            l.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!("#{} {}", i + 1, task_display),
                    Style::default().fg(G2),
                ),
            ]));
        }
    }

    let user_mem = agent.get_user_memory();
    if !user_mem.is_empty() {
        l.push(Line::default());
        l.push(Line::from(Span::styled(
            "Memory",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )));
        for (k, v) in user_mem.iter().take(5) {
            let val: String = v.chars().take(18).collect();
            l.push(Line::from(Span::styled(
                format!("  {}: {}", k, val),
                Style::default().fg(G3),
            )));
        }
    }

    l.push(Line::default());
    l.push(Line::from(Span::styled(
        "Free API keys",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    l.push(Line::from(Span::styled(
        "  openrouter.ai/keys",
        Style::default().fg(G1),
    )));
    l.push(Line::from(Span::styled(
        "  console.groq.com",
        Style::default().fg(G1),
    )));
    l.push(Line::from(Span::styled(
        "  cloud.cerebras.ai",
        Style::default().fg(G1),
    )));

    if let Some(new_ver) = app.update_available.lock().ok().and_then(|g| g.clone()) {
        l.push(Line::default());
        l.push(Line::from(Span::styled(
            "★ Update Available",
            Style::default()
                .fg(ACCENT_GOLD)
                .add_modifier(Modifier::BOLD),
        )));
        l.push(Line::from(Span::styled(
            format!("  v{} → {}", VERSION, new_ver),
            Style::default().fg(ACCENT_GOLD),
        )));
        l.push(Line::from(Span::styled(
            "  openzax upgrade",
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )));
    }

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
    let visible_h = f.area().height.saturating_sub(6).min(30);
    let popup = popup_rect(f.area(), 72, visible_h + 6);
    app.ov_rect = popup;
    f.render_widget(Clear, popup);
    f.render_widget(Block::default().style(Style::default().bg(BG_POPUP)), popup);

    let chat_count = FREE_MODELS
        .iter()
        .filter(|m| m.category == "text" || m.category == "vision")
        .count();
    let title_text = format!(" Switch Model ({}/{}) ", app.ov_idx + 1, FREE_MODELS.len());
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(G3))
        .style(Style::default().bg(BG_POPUP))
        .title(Span::styled(
            title_text,
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

    let header = format!(
        " {} text/vision  ·  {} image  ·  {} embed  ↑↓ scroll",
        chat_count,
        FREE_MODELS.iter().filter(|m| m.category == "image").count(),
        FREE_MODELS
            .iter()
            .filter(|m| m.category == "embedding")
            .count(),
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            header,
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
    let list_h = rows[2].height as usize;

    let scroll_off = if app.ov_idx >= list_h {
        app.ov_idx - list_h + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, m) in FREE_MODELS.iter().enumerate().skip(scroll_off).take(list_h) {
        let sel = i == app.ov_idx;
        let cur = app.model_name == m.id;
        let (fg, bg) = if sel { (BLK, BG_SEL) } else { (G1, BG_POPUP) };
        let mark = if cur { "> " } else { "  " };
        let cat_badge = match m.category {
            "vision" => "[V] ",
            "image" => "[I] ",
            "embedding" => "[E] ",
            _ => "    ",
        };
        let cat_color = match m.category {
            "vision" => Color::Rgb(130, 200, 255),
            "image" => Color::Rgb(255, 150, 200),
            "embedding" => Color::Rgb(200, 255, 150),
            _ => {
                if sel {
                    G3
                } else {
                    G4
                }
            }
        };
        lines.push(Line::from(vec![
            Span::styled(mark, Style::default().fg(if cur { W } else { G4 }).bg(bg)),
            Span::styled(
                cat_badge,
                Style::default()
                    .fg(if sel { BLK } else { cat_color })
                    .bg(bg),
            ),
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

fn push_breaking(
    out: &mut Vec<String>,
    cur: &mut String,
    cl: &mut usize,
    word: &str,
    width: usize,
) {
    let chars: Vec<char> = word.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let remaining = width.saturating_sub(*cl);
        if remaining == 0 {
            out.push(std::mem::take(cur));
            *cl = 0;
            continue;
        }
        let take = remaining.min(chars.len() - i);
        let chunk: String = chars[i..i + take].iter().collect();
        cur.push_str(&chunk);
        *cl += take;
        i += take;
        if *cl >= width && i < chars.len() {
            out.push(std::mem::take(cur));
            *cl = 0;
        }
    }
}

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
        let mut cl = 0usize;
        for w in raw.split_whitespace() {
            let wl = w.chars().count();
            if cl == 0 {
                if wl <= width {
                    cur.push_str(w);
                    cl = wl;
                } else {
                    push_breaking(&mut out, &mut cur, &mut cl, w, width);
                }
            } else if cl + 1 + wl <= width {
                cur.push(' ');
                cur.push_str(w);
                cl += 1 + wl;
            } else {
                if wl <= width {
                    out.push(std::mem::take(&mut cur));
                    cur.push_str(w);
                    cl = wl;
                } else {
                    cur.push(' ');
                    cl += 1;
                    push_breaking(&mut out, &mut cur, &mut cl, w, width);
                }
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

fn handle_slash(app: &mut App, cmd: &str, agent: &Agent) -> bool {
    match cmd.trim() {
        "/help" | "/h" => {
            app.push(Msg::System("Tab mode · Ctrl+T tier · Ctrl+P cmds · Ctrl+M model · Ctrl+K skills · Ctrl+N new · Shift+Enter newline · /exit quit".into()));
            true
        }
        "/clear" | "/new" => {
            app.msgs.clear();
            agent.clear_history();
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

    // Panic hook to restore terminal no matter what
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableBracketedPaste
        );
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = main_loop(&mut terminal, model_name, api_key, db_path).await;

    // Always restore terminal — even on error
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    );
    let _ = terminal.show_cursor();
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

    // Background update check
    {
        let update_flag = Arc::clone(&app.update_available);
        tokio::spawn(async move {
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .user_agent("openzax-cli")
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
            let json: serde_json::Value = match resp.json().await {
                Ok(j) => j,
                Err(_) => return,
            };
            if let Some(tag) = json["tag_name"].as_str() {
                let current = format!("v{}", env!("CARGO_PKG_VERSION"));
                if tag != current {
                    if let Ok(mut guard) = update_flag.lock() {
                        *guard = Some(tag.to_string());
                    }
                }
            }
        });
    }

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
        system_prompt: Some(app.mode_prompt()),
        ..Default::default()
    };
    let agent = Arc::new(Agent::new(cfg, eb.clone()));
    app.agent_ref = Some(Arc::clone(&agent));
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
                        if let Ok(mut b) = buf.lock() {
                            b.push_str(&token);
                        }
                    }
                    OzEvent::AgentOutput { .. } => {
                        if let Ok(mut d) = df.lock() {
                            *d = true;
                        }
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
        if terminal.draw(|f| render(f, &mut app)).is_err() {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        match event::poll(Duration::from_millis(40)) {
            Ok(false) => continue,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Ok(true) => {}
        }

        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
        };
        match ev {
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

            Event::Paste(text) => {
                for c in text.chars() {
                    if c != '\r' {
                        if c == '\n' {
                            app.ins('\n');
                        } else {
                            app.ins(c);
                        }
                    }
                }
                app.paste_cooldown = Instant::now();
                continue;
            }

            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // After a paste, ignore key events for 150ms to prevent
                // the terminal's duplicate paste-as-keys from firing
                if app.paste_cooldown.elapsed() < Duration::from_millis(150) {
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
                    agent.clear_history();
                    app.phase = Phase::Empty;
                    app.session_tokens = 0;
                    app.session_start = Instant::now();
                    continue;
                }
                if is_ctrl(&key, 't') {
                    app.tier_idx = (app.tier_idx + 1) % TIERS.len();
                    agent.set_system_prompt(app.mode_prompt());
                    if app.phase != Phase::Empty {
                        app.push(Msg::System(format!("Tier: {}", TIERS[app.tier_idx])));
                    }
                    continue;
                }

                // Ctrl+V: paste from clipboard into input
                if is_ctrl(&key, 'v') {
                    if let Some(text) = clipboard_paste() {
                        for c in text.chars() {
                            if c == '\r' {
                                continue;
                            }
                            app.ins(c);
                        }
                    }
                    app.paste_cooldown = Instant::now();
                    continue;
                }

                // Skip other Ctrl combos
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    continue;
                }

                if key.code == KeyCode::Tab && app.phase != Phase::Stream {
                    app.mode = match app.mode {
                        Mode::Build => Mode::Plan,
                        Mode::Plan => Mode::MultiAgent,
                        Mode::MultiAgent => Mode::Build,
                    };
                    agent.set_system_prompt(app.mode_prompt());
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
                            handle_slash(&mut app, text.trim(), &agent);
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
                            let result = ag.process_streaming(&text).await;
                            if let Err(e) = result {
                                if let Ok(mut b) = sb.lock() {
                                    b.push_str(&format!("\n[Error] {}", e));
                                }
                            }
                            if let Ok(mut d) = df.lock() {
                                *d = true;
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
                        agent.clear_history();
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
                            Mode::Plan => Mode::MultiAgent,
                            Mode::MultiAgent => Mode::Build,
                        };
                        agent.set_system_prompt(app.mode_prompt());
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
