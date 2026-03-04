use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "llm-engine")]
use openzax_llm_engine::local::LocalModelManager;

mod tui;
mod ui;

// ── CLI Struct ────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "openzax")]
#[command(about = "OpenZax - Secure AI Development Assistant", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive terminal shell
    Shell {
        #[arg(short, long)]
        api_key: Option<String>,
        #[arg(short, long, default_value = "deepseek/deepseek-r1-0528:free")]
        model: String,
        #[arg(short, long, default_value = ".openzax/openzax.db")]
        db_path: PathBuf,
    },

    /// Initialize a new skill project
    Init {
        name: String,
        #[arg(short, long, default_value = "rust")]
        language: String,
    },

    /// Model management commands
    #[command(subcommand)]
    Model(ModelCommands),

    /// Skill management commands
    #[command(subcommand)]
    Skill(SkillCommands),

    /// MCP server management commands
    #[command(subcommand)]
    Mcp(McpCommands),

    /// Generate an Ed25519 keypair for signing skills
    Keygen {
        /// Base name for output key files (e.g. "mykey" → mykey.private.key + mykey.public.key)
        #[arg(short, long, default_value = "openzax")]
        output: PathBuf,
    },

    /// Store authentication token
    Login {
        #[arg(short, long)]
        token: String,
    },

    /// Show currently logged-in user
    Whoami,

    /// Search the skill marketplace
    Search {
        query: String,
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Install a skill from the marketplace
    Install {
        skill_name: String,
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Run system health checks
    Doctor,

    /// Check for and apply CLI upgrades
    Upgrade {
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Display version information
    Version,
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Initialize a new skill project
    Init {
        name: String,
        #[arg(short, long, default_value = "rust")]
        language: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build a skill to WASM
    Build {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        release: bool,
    },

    /// Test a skill
    Test {
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Package a skill into a distributable .ozskill bundle
    Pack {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Sign a skill package with an Ed25519 private key
    Sign {
        package: PathBuf,
        #[arg(short, long)]
        key: PathBuf,
    },

    /// Sign and publish a skill to the marketplace
    Publish {
        package: PathBuf,
        #[arg(short, long)]
        key: Option<PathBuf>,
        #[arg(short, long, default_value = "https://api.openzax.dev")]
        marketplace: String,
    },

    /// Inspect a .ozskill package (manifest, permissions, WASM info)
    Inspect { path: PathBuf },

    /// Validate a .ozskill package structure
    Validate { path: PathBuf },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available local models
    List {
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Download a model from Hugging Face
    Download {
        name: String,
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Show detailed information about a model
    Info {
        name: String,
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Remove a local model
    Remove {
        name: String,
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Start a mock MCP server on stdio (for testing clients)
    Simulate {
        /// Server identity name shown in capabilities
        server: String,
    },

    /// Connect to an MCP server and list its tools and resources
    Inspect {
        /// Command to start the MCP server (e.g. "npx @mcp/server-fs /tmp")
        server: String,
    },

    /// Record an MCP session to a JSONL file
    Record {
        server: String,
        #[arg(short, long)]
        output: PathBuf,
    },
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn load_api_key_from_config() -> Option<String> {
    let home = dirs::home_dir()?;
    let config = home.join(".openzax").join("config.toml");
    let content = std::fs::read_to_string(config).ok()?;
    for line in content.lines() {
        if line.trim_start().starts_with("api_key") {
            if let Some(val) = line.split_once('=').map(|x| x.1) {
                let key = val.trim().trim_matches('"').trim_matches('\'').to_string();
                if !key.is_empty() {
                    return Some(key);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
async fn check_for_update() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .user_agent("openzax-cli")
        .build()
        .ok()?;
    let resp = client
        .get("https://api.github.com/repos/zAxCoder/OpenZax/releases/latest")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    let tag = json["tag_name"].as_str()?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if tag != current.as_str() {
        Some(format!("{} → {} available", current, tag))
    } else {
        None
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable ANSI escape codes on Windows (CMD / old consoles)
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("openzax={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Some(Commands::Shell {
            api_key,
            model,
            db_path,
        }) => {
            let resolved_key = api_key
                .or_else(|| std::env::var("OPENZAX_API_KEY").ok())
                .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
                .or_else(load_api_key_from_config);
            tui::run_tui(model, resolved_key, db_path).await?;
        }
        Some(Commands::Init { name, language }) => {
            ui::print_banner();
            ui::print_info(&format!("Initializing new {} skill: {}", language, name));
            println!();
            ui::print_info("This feature is coming soon in Phase 2!");
            println!();
        }
        Some(Commands::Model(cmd)) => {
            ui::print_banner();
            handle_model_command(cmd).await?;
        }
        Some(Commands::Skill(cmd)) => {
            ui::print_banner();
            handle_skill_command(cmd).await?;
        }
        Some(Commands::Mcp(cmd)) => {
            ui::print_banner();
            handle_mcp_command(cmd).await?;
        }
        Some(Commands::Keygen { output }) => {
            ui::print_banner();
            handle_keygen(output)?;
        }
        Some(Commands::Login { token }) => {
            ui::print_banner();
            handle_login(token)?;
        }
        Some(Commands::Whoami) => {
            ui::print_banner();
            handle_whoami()?;
        }
        Some(Commands::Search {
            query,
            category,
            limit,
        }) => {
            ui::print_banner();
            handle_search(query, category, limit).await?;
        }
        Some(Commands::Install {
            skill_name,
            version,
        }) => {
            ui::print_banner();
            handle_install(skill_name, version).await?;
        }
        Some(Commands::Doctor) => {
            ui::print_banner();
            handle_doctor().await?;
        }
        Some(Commands::Upgrade { version }) => {
            ui::print_banner();
            handle_upgrade(version).await?;
        }
        Some(Commands::Version) => {
            ui::print_banner();
            println!(
                "  {} {}",
                "Version:".bright_white().dimmed(),
                env!("CARGO_PKG_VERSION").bright_cyan().bold()
            );
            println!(
                "  {} {}",
                "Description:".bright_white().dimmed(),
                "Rust-native AI development assistant".bright_white()
            );
            println!();
        }
        None => {
            let api_key = std::env::var("OPENZAX_API_KEY")
                .ok()
                .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
                .or_else(load_api_key_from_config);
            let model = "deepseek/deepseek-r1-0528:free".to_string();
            let db_path = dirs::home_dir()
                .map(|h| h.join(".openzax").join("openzax.db"))
                .unwrap_or_else(|| std::path::PathBuf::from(".openzax/openzax.db"));
            tui::run_tui(model, api_key, db_path).await?;
        }
    }

    Ok(())
}

// ── keygen ────────────────────────────────────────────────────────────────────

fn handle_keygen(output: PathBuf) -> anyhow::Result<()> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use ed25519_dalek::SigningKey;
    use rand_core::OsRng;
    use sha2::{Digest, Sha256};

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_bytes = signing_key.to_bytes();
    let public_bytes = verifying_key.to_bytes();

    let private_b64 = BASE64.encode(private_bytes);
    let public_b64 = BASE64.encode(public_bytes);

    let mut hasher = Sha256::new();
    hasher.update(public_bytes);
    let hash = hasher.finalize();
    let fingerprint: String = hash[..16]
        .chunks(2)
        .map(|c| format!("{:02x}{:02x}", c[0], c[1]))
        .collect::<Vec<_>>()
        .join(":");

    let parent = output.parent().unwrap_or_else(|| std::path::Path::new("."));
    let stem = output
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "openzax".to_string());

    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent)?;
    }
    let private_path = parent.join(format!("{}.private.key", stem));
    let public_path = parent.join(format!("{}.public.key", stem));

    std::fs::write(&private_path, &private_b64)?;
    std::fs::write(&public_path, &public_b64)?;

    println!("  {} Generated Ed25519 keypair", "✓".green().bold());
    println!(
        "  {} Private key : {}",
        "→".bright_cyan(),
        private_path.display()
    );
    println!(
        "  {} Public key  : {}",
        "→".bright_cyan(),
        public_path.display()
    );
    println!("  {} Fingerprint : {}", "◆".bright_yellow(), fingerprint);
    println!();
    println!(
        "  {} Keep your private key secure and never share it.",
        "⚠".bright_yellow().bold()
    );
    println!();

    Ok(())
}

// ── login / whoami ────────────────────────────────────────────────────────────

fn handle_login(token: String) -> anyhow::Result<()> {
    let auth_path = openzax_config_dir()?.join("auth.json");
    let auth = serde_json::json!({
        "token": token,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(&auth_path, serde_json::to_string_pretty(&auth)?)?;
    println!(
        "  {} Authentication token saved to {}",
        "✓".green().bold(),
        auth_path.display()
    );
    println!();
    Ok(())
}

fn handle_whoami() -> anyhow::Result<()> {
    let auth_path = openzax_config_dir()?.join("auth.json");

    if !auth_path.exists() {
        println!(
            "  {} Not logged in. Use: {}",
            "✗".red().bold(),
            "openzax login --token <token>".bright_cyan()
        );
        println!();
        return Ok(());
    }

    let content = std::fs::read_to_string(&auth_path)?;
    let auth: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(token) = auth.get("token").and_then(|v| v.as_str()) {
        let masked = if token.len() > 8 {
            format!("{}...{}", &token[..4], &token[token.len() - 4..])
        } else {
            "****".to_string()
        };
        println!(
            "  {} Token   : {}",
            "✓".green().bold(),
            masked.bright_cyan()
        );
    }
    if let Some(created) = auth.get("created_at").and_then(|v| v.as_str()) {
        println!("  {} Since   : {}", "◆".bright_cyan(), created);
    }
    println!();
    Ok(())
}

// ── search ────────────────────────────────────────────────────────────────────

async fn handle_search(query: String, category: Option<String>, limit: u32) -> anyhow::Result<()> {
    println!(
        "  {} Searching marketplace for {}...",
        "→".bright_cyan(),
        query.bright_white().bold()
    );
    println!();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(format!("openzax-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut url = format!(
        "https://api.openzax.dev/v1/search?q={}",
        urlencoding(&query)
    );
    if let Some(cat) = &category {
        url.push_str(&format!("&category={}", cat));
    }
    url.push_str(&format!("&limit={}", limit));

    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            let data: serde_json::Value = response.json().await?;
            let items = data["results"]
                .as_array()
                .map(|a| a.as_slice())
                .unwrap_or(&[]);
            if items.is_empty() {
                println!("  {} No results found for '{}'", "◆".bright_yellow(), query);
            } else {
                println!("  Found {} result(s):\n", items.len());
                for item in items {
                    let name = item["name"].as_str().unwrap_or("unknown");
                    let version = item["version"].as_str().unwrap_or("?");
                    let desc = item["description"].as_str().unwrap_or("");
                    let author = item["author"].as_str().unwrap_or("?");
                    println!(
                        "  {} {} {}  by {}",
                        "•".bright_cyan(),
                        name.bright_white().bold(),
                        format!("v{}", version).dimmed(),
                        author.bright_yellow()
                    );
                    if !desc.is_empty() {
                        println!("    {}", desc);
                    }
                    println!();
                }
            }
        }
        Ok(response) => {
            println!(
                "  {} Marketplace returned error: {}",
                "✗".red().bold(),
                response.status()
            );
        }
        Err(_) => {
            println!("  {} Marketplace offline", "✗".red().bold());
            println!(
                "  {} Could not reach {}",
                "→".bright_cyan(),
                "https://api.openzax.dev".dimmed()
            );
        }
    }

    Ok(())
}

// ── install ───────────────────────────────────────────────────────────────────

async fn handle_install(skill_name: String, version: Option<String>) -> anyhow::Result<()> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use ed25519_dalek::{Signature, VerifyingKey};
    use sha2::{Digest, Sha256};

    let ver = version.as_deref().unwrap_or("latest");
    println!(
        "  {} Installing {} {}...",
        "→".bright_cyan(),
        skill_name.bright_white().bold(),
        format!("({})", ver).dimmed()
    );
    println!();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("openzax-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let meta_url = format!(
        "https://api.openzax.dev/v1/skills/{}/{}",
        urlencoding(&skill_name),
        ver
    );

    let meta: serde_json::Value = match client.get(&meta_url).send().await {
        Ok(r) if r.status().is_success() => r.json().await?,
        Ok(r) => {
            anyhow::bail!("Marketplace error: {}", r.status());
        }
        Err(_) => {
            println!("  {} Marketplace offline", "✗".red().bold());
            return Ok(());
        }
    };

    let download_url = meta["download_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No download_url in response"))?;
    let sig_b64 = meta["signature"].as_str().unwrap_or("");
    let pub_key_b64 = meta["publisher_key"].as_str().unwrap_or("");

    println!("  {} Downloading package...", "→".bright_cyan());
    let bytes = client.get(download_url).send().await?.bytes().await?;

    // Verify Ed25519 signature if provided
    if !sig_b64.is_empty() && !pub_key_b64.is_empty() {
        let sig_bytes = BASE64.decode(sig_b64)?;
        let pub_bytes = BASE64.decode(pub_key_b64)?;

        let pub_array: [u8; 32] = pub_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;

        let verifying_key = VerifyingKey::from_bytes(&pub_array)?;
        let signature = Signature::from_bytes(&sig_array);

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hasher.finalize();

        use ed25519_dalek::Verifier;
        verifying_key.verify(&hash, &signature).map_err(|_| {
            anyhow::anyhow!("Signature verification failed – package may be tampered")
        })?;

        println!("  {} Signature verified", "✓".green().bold());
    } else {
        println!(
            "  {} No signature – installing unsigned skill",
            "⚠".bright_yellow().bold()
        );
    }

    let skills_dir = openzax_config_dir()?.join("skills");
    std::fs::create_dir_all(&skills_dir)?;
    let out_path = skills_dir.join(format!("{}.ozskill", skill_name));
    std::fs::write(&out_path, &bytes)?;

    println!(
        "  {} Installed to {}",
        "✓".green().bold(),
        out_path.display()
    );
    println!();

    Ok(())
}

// ── doctor ────────────────────────────────────────────────────────────────────

async fn handle_doctor() -> anyhow::Result<()> {
    println!("  Running system health checks...\n");

    // 1. Rust / cargo
    let cargo_out = std::process::Command::new("cargo")
        .arg("--version")
        .output();
    let (cargo_ok, cargo_msg) = match cargo_out {
        Ok(o) if o.status.success() => {
            (true, String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        _ => (false, "cargo not found".to_string()),
    };
    print_doctor_check("Rust / Cargo", cargo_ok, &cargo_msg);

    // 2. wasmtime binary
    let wt_out = std::process::Command::new("wasmtime")
        .arg("--version")
        .output();
    let (wt_ok, wt_msg) = match wt_out {
        Ok(o) if o.status.success() => {
            (true, String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        _ => (
            false,
            "wasmtime not found (optional – needed to run WASM skills)".to_string(),
        ),
    };
    print_doctor_check("wasmtime CLI", wt_ok, &wt_msg);

    // 3. ~/.openzax directory structure
    let config_dir_res = openzax_config_dir();
    match &config_dir_res {
        Ok(dir) => {
            print_doctor_check("~/.openzax directory", true, &dir.display().to_string());
            let skills_dir = dir.join("skills");
            let models_dir = dir.join("models");
            print_doctor_check(
                "  skills/",
                skills_dir.exists(),
                if skills_dir.exists() {
                    "exists"
                } else {
                    "missing (will be created on first install)"
                },
            );
            print_doctor_check(
                "  models/",
                models_dir.exists(),
                if models_dir.exists() {
                    "exists"
                } else {
                    "missing (optional)"
                },
            );
            let db_path = dir.join("openzax.db");
            let db_accessible = if db_path.exists() {
                std::fs::OpenOptions::new()
                    .read(true)
                    .open(&db_path)
                    .is_ok()
            } else {
                true
            };
            print_doctor_check(
                "Database file",
                db_accessible,
                if db_path.exists() {
                    "accessible"
                } else {
                    "not yet created"
                },
            );
        }
        Err(_) => {
            print_doctor_check("~/.openzax directory", false, "home directory not found");
        }
    }

    // 4. Network connectivity
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(format!("openzax-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let ping = client.get("https://api.openzax.dev/health").send().await;
    let (net_ok, net_msg) = match ping {
        Ok(r) => (true, format!("reachable (HTTP {})", r.status())),
        Err(e) if e.is_timeout() => (false, "timeout".to_string()),
        Err(_) => (false, "unreachable (marketplace offline)".to_string()),
    };
    print_doctor_check("Network (api.openzax.dev)", net_ok, &net_msg);

    println!();
    Ok(())
}

fn print_doctor_check(label: &str, ok: bool, detail: &str) {
    let symbol = if ok {
        "✓".green().bold()
    } else {
        "✗".red().bold()
    };
    let detail_colored = if ok {
        detail.dimmed().to_string()
    } else {
        detail.bright_yellow().to_string()
    };
    println!("  {} {:<35} {}", symbol, label, detail_colored);
}

// ── upgrade ───────────────────────────────────────────────────────────────────

async fn handle_upgrade(version: Option<String>) -> anyhow::Result<()> {
    println!("  {} Checking for updates...", "→".bright_cyan());
    println!();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("openzax-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let url = match &version {
        Some(v) => format!(
            "https://api.github.com/repos/zAxCoder/OpenZax/releases/tags/v{}",
            v
        ),
        None => "https://api.github.com/repos/zAxCoder/OpenZax/releases/latest".to_string(),
    };

    match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => {
            let data: serde_json::Value = r.json().await?;
            let latest = data["tag_name"].as_str().unwrap_or("unknown");
            let current = env!("CARGO_PKG_VERSION");

            println!(
                "  {} Current version : {}",
                "◆".bright_cyan(),
                format!("v{}", current).bright_white()
            );
            println!(
                "  {} Latest version  : {}",
                "◆".bright_cyan(),
                latest.bright_green()
            );
            println!();

            if latest == format!("v{}", current) || latest == current {
                println!("  {} You are on the latest version!", "✓".green().bold());
            } else {
                println!(
                    "  {} A newer version is available!",
                    "★".bright_yellow().bold()
                );
                println!();

                // Try to auto-download and install
                let asset_name = if cfg!(target_os = "windows") {
                    "openzax-windows-x86_64.zip"
                } else if cfg!(target_os = "macos") {
                    "openzax-macos-aarch64.tar.gz"
                } else {
                    "openzax-linux-x86_64.tar.gz"
                };

                let download_url = data["assets"].as_array().and_then(|assets| {
                    assets.iter().find_map(|a| {
                        let name = a["name"].as_str().unwrap_or("");
                        if name == asset_name {
                            a["browser_download_url"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                });

                if let Some(url) = download_url {
                    println!("  {} Downloading {}...", "→".bright_cyan(), asset_name);

                    match client.get(&url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let bytes = resp.bytes().await?;
                            let exe_name = if cfg!(target_os = "windows") {
                                "openzax.exe"
                            } else {
                                "openzax"
                            };

                            let tmp_dir = std::env::temp_dir();
                            let tmp_binary = tmp_dir.join(format!("openzax-update-{}", exe_name));

                            if cfg!(target_os = "windows") {
                                let cursor = std::io::Cursor::new(&bytes);
                                let mut archive = zip::ZipArchive::new(cursor)?;
                                for i in 0..archive.len() {
                                    let mut file = archive.by_index(i)?;
                                    let name = file.name().to_string();
                                    if name.ends_with(exe_name) {
                                        use std::io::Read;
                                        let mut buf = Vec::new();
                                        file.read_to_end(&mut buf)?;
                                        std::fs::write(&tmp_binary, &buf)?;
                                        break;
                                    }
                                }
                            } else {
                                let cursor = std::io::Cursor::new(&bytes);
                                let gz = flate2::read::GzDecoder::new(cursor);
                                let mut archive = tar::Archive::new(gz);
                                for entry in archive.entries()? {
                                    let mut entry = entry?;
                                    let path = entry.path()?.to_path_buf();
                                    if path.file_name().map(|n| n == exe_name).unwrap_or(false) {
                                        entry.unpack(&tmp_binary)?;
                                        break;
                                    }
                                }
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    std::fs::set_permissions(
                                        &tmp_binary,
                                        std::fs::Permissions::from_mode(0o755),
                                    )?;
                                }
                            }

                            let current_exe =
                                std::env::current_exe().unwrap_or_else(|_| PathBuf::from(exe_name));

                            let mut installed_paths: Vec<PathBuf> = Vec::new();

                            // Replace the currently running binary
                            if current_exe.exists() {
                                let old_exe = current_exe.with_extension("old");
                                let _ = std::fs::remove_file(&old_exe);
                                if std::fs::rename(&current_exe, &old_exe).is_ok() {
                                    if std::fs::copy(&tmp_binary, &current_exe).is_ok() {
                                        let _ = std::fs::remove_file(&old_exe);
                                        installed_paths.push(current_exe.clone());
                                    } else {
                                        let _ = std::fs::rename(&old_exe, &current_exe);
                                    }
                                }
                            }

                            // Also install to ~/.openzax/bin/
                            if let Some(home) = dirs::home_dir() {
                                let openzax_bin = home.join(".openzax").join("bin");
                                let _ = std::fs::create_dir_all(&openzax_bin);
                                let dest = openzax_bin.join(exe_name);
                                if dest != current_exe && std::fs::copy(&tmp_binary, &dest).is_ok()
                                {
                                    installed_paths.push(dest);
                                }

                                // Also try ~/.cargo/bin/
                                let cargo_dest = home.join(".cargo").join("bin").join(exe_name);
                                if cargo_dest != current_exe {
                                    let _ = std::fs::copy(&tmp_binary, &cargo_dest);
                                }
                            }

                            let _ = std::fs::remove_file(&tmp_binary);

                            if installed_paths.is_empty() {
                                println!(
                                    "  {} Could not replace binary. Try running as admin.",
                                    "⚠".bright_yellow().bold()
                                );
                                print_manual_upgrade_instructions();
                            } else {
                                for p in &installed_paths {
                                    println!(
                                        "  {} Installed {} to {}",
                                        "✓".green().bold(),
                                        latest,
                                        p.display()
                                    );
                                }
                                println!();
                                println!(
                                    "  {} Restart your terminal to use the new version.",
                                    "→".bright_cyan()
                                );
                            }
                        }
                        _ => {
                            println!("  {} Download failed", "✗".red().bold());
                            print_manual_upgrade_instructions();
                        }
                    }
                } else {
                    println!(
                        "  {} No binary found for your platform ({})",
                        "⚠".bright_yellow(),
                        asset_name
                    );
                    print_manual_upgrade_instructions();
                }
            }
        }
        Ok(r) => {
            println!("  {} GitHub API error: {}", "✗".red(), r.status());
        }
        Err(_) => {
            println!("  {} Could not reach GitHub API", "✗".red().bold());
            println!(
                "  {} Visit {} for the latest releases",
                "→".bright_cyan(),
                "https://github.com/zAxCoder/OpenZax/releases".bright_cyan()
            );
        }
    }

    println!();
    Ok(())
}

fn print_manual_upgrade_instructions() {
    println!();
    println!("  To upgrade manually:");
    println!();
    if cfg!(target_os = "windows") {
        println!(
            "    {}",
            "irm https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex"
                .bright_cyan()
        );
    } else {
        println!(
            "    {}",
            "curl -fsSL https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.sh | bash"
                .bright_cyan()
        );
    }
    println!();
    println!(
        "  Or download from: {}",
        "https://github.com/zAxCoder/OpenZax/releases".bright_cyan()
    );
}

// ── skill commands ────────────────────────────────────────────────────────────

async fn handle_skill_command(cmd: SkillCommands) -> anyhow::Result<()> {
    match cmd {
        SkillCommands::Init {
            name,
            language,
            output,
        } => {
            let output_dir = output.unwrap_or_else(|| PathBuf::from(&name));
            ui::print_info(&format!("Creating new {} skill: {}", language, name));
            println!();
            match language.as_str() {
                "rust" => create_rust_skill(&name, &output_dir)?,
                "typescript" | "ts" => {
                    ui::print_error("TypeScript skills coming soon!");
                    println!();
                    ui::print_info("Use Rust for now: openzax skill init --language rust");
                    println!();
                    return Ok(());
                }
                "python" | "py" => {
                    ui::print_error("Python skills coming soon!");
                    println!();
                    ui::print_info("Use Rust for now: openzax skill init --language rust");
                    println!();
                    return Ok(());
                }
                _ => {
                    ui::print_error(&format!("Unsupported language: {}", language));
                    println!();
                    ui::print_info("Supported: rust, typescript, python");
                    println!();
                    return Ok(());
                }
            }
            println!();
            ui::print_info(&format!("✓ Skill created at: {}", output_dir.display()));
            println!();
            println!("  {} Next steps:", "→".bright_cyan());
            println!("    cd {}", name);
            println!("    openzax skill build");
            println!();
        }

        SkillCommands::Build { path, release } => {
            ui::print_info(&format!("Building skill at: {}", path.display()));
            println!();
            let build_mode = if release { "release" } else { "debug" };
            ui::print_info(&format!("Build mode: {}", build_mode));
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build")
                .arg("--target")
                .arg("wasm32-wasip1")
                .current_dir(&path);
            if release {
                cmd.arg("--release");
            }
            let status = cmd.status()?;
            if status.success() {
                println!();
                ui::print_info("✓ Build successful!");
                let wasm_path = path.join("target/wasm32-wasip1").join(build_mode);
                ui::print_info(&format!("WASM output: {}", wasm_path.display()));
                println!();
            } else {
                ui::print_error("Build failed!");
                std::process::exit(1);
            }
        }

        SkillCommands::Test { path } => {
            ui::print_info(&format!("Testing skill at: {}", path.display()));
            println!();
            let status = std::process::Command::new("cargo")
                .arg("test")
                .current_dir(&path)
                .status()?;
            if !status.success() {
                ui::print_error("Tests failed!");
                std::process::exit(1);
            }
        }

        SkillCommands::Pack { path, output } => {
            skill_pack(&path, output.as_deref())?;
        }

        SkillCommands::Sign { package, key } => {
            skill_sign(&package, &key)?;
        }

        SkillCommands::Publish {
            package,
            key,
            marketplace,
        } => {
            skill_publish(&package, key.as_deref(), &marketplace).await?;
        }

        SkillCommands::Inspect { path } => {
            skill_inspect(&path)?;
        }

        SkillCommands::Validate { path } => {
            skill_validate(&path)?;
        }
    }

    Ok(())
}

// ── skill helpers ─────────────────────────────────────────────────────────────

fn skill_pack(path: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    use std::io::Write;
    use zip::write::{SimpleFileOptions, ZipWriter};

    ui::print_info(&format!("Packing skill at: {}", path.display()));
    println!();

    // Read manifest
    let manifest_path = path.join("manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!(
            "manifest.json not found in {}. Run 'openzax skill build' first.",
            path.display()
        );
    }
    let manifest_data = std::fs::read(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_data)?;

    let skill_name = manifest["name"].as_str().unwrap_or("skill");
    let skill_version = manifest["version"].as_str().unwrap_or("0.1.0");

    // Find .wasm file
    let wasm_candidates = [
        path.join("target/wasm32-wasip1/release")
            .join(format!("{}.wasm", skill_name.replace('-', "_"))),
        path.join("target/wasm32-wasip1/debug")
            .join(format!("{}.wasm", skill_name.replace('-', "_"))),
        path.join(format!("{}.wasm", skill_name.replace('-', "_"))),
    ];

    let wasm_path = wasm_candidates
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("No .wasm file found. Run 'openzax skill build' first."))?;

    ui::print_info(&format!("WASM: {}", wasm_path.display()));

    let wasm_data = std::fs::read(wasm_path)?;

    let default_output = path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join(format!("{}-{}.ozskill", skill_name, skill_version));
    let out_path: PathBuf = output.map(|p| p.to_path_buf()).unwrap_or(default_output);

    let file = std::fs::File::create(&out_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("manifest.json", options)?;
    zip.write_all(&manifest_data)?;

    zip.start_file("skill.wasm", options)?;
    zip.write_all(&wasm_data)?;

    zip.finish()?;

    println!("  {} Packed to {}", "✓".green().bold(), out_path.display());
    println!(
        "  {} Size: {} bytes",
        "◆".bright_cyan(),
        std::fs::metadata(out_path)?.len()
    );
    println!();

    Ok(())
}

fn skill_sign(package: &PathBuf, key_path: &PathBuf) -> anyhow::Result<()> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use ed25519_dalek::SigningKey;
    use sha2::{Digest, Sha256};

    ui::print_info(&format!("Signing: {}", package.display()));
    println!();

    let key_b64 = std::fs::read_to_string(key_path)?.trim().to_string();
    let key_bytes = BASE64.decode(&key_b64)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid private key length (expected 32 bytes)"))?;
    let signing_key = SigningKey::from_bytes(&key_array);

    let package_data = std::fs::read(package)?;
    let mut hasher = Sha256::new();
    hasher.update(&package_data);
    let hash = hasher.finalize();

    use ed25519_dalek::Signer;
    let signature = signing_key.sign(&hash);
    let sig_b64 = BASE64.encode(signature.to_bytes());
    let pub_b64 = BASE64.encode(signing_key.verifying_key().to_bytes());

    let sig_doc = serde_json::json!({
        "algorithm": "ed25519",
        "hash": "sha256",
        "signature": sig_b64,
        "public_key": pub_b64,
        "signed_at": chrono::Utc::now().to_rfc3339(),
    });

    let sig_path = {
        let mut p = package.clone().into_os_string();
        p.push(".sig");
        PathBuf::from(p)
    };
    std::fs::write(&sig_path, serde_json::to_string_pretty(&sig_doc)?)?;

    println!(
        "  {} Signature written to {}",
        "✓".green().bold(),
        sig_path.display()
    );
    println!("  {} Public key: {}", "◆".bright_cyan(), pub_b64.dimmed());
    println!();

    Ok(())
}

async fn skill_publish(
    package: &PathBuf,
    key_path: Option<&std::path::Path>,
    marketplace: &str,
) -> anyhow::Result<()> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    ui::print_info(&format!("Publishing: {}", package.display()));
    println!();

    // Sign first if key is provided
    if let Some(key) = key_path {
        skill_sign(package, &key.to_path_buf())?;
    }

    let package_data = std::fs::read(package)?;

    let sig_path = {
        let mut p = package.clone().into_os_string();
        p.push(".sig");
        PathBuf::from(p)
    };

    let auth_path = openzax_config_dir()?.join("auth.json");
    if !auth_path.exists() {
        anyhow::bail!("Not logged in. Run 'openzax login --token <token>' first.");
    }
    let auth: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&auth_path)?)?;
    let token = auth["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid auth token"))?
        .to_string();

    println!(
        "  {} Uploading package ({} bytes)...",
        "→".bright_cyan(),
        package_data.len()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent(format!("openzax-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let url = format!("{}/v1/skills/publish", marketplace);
    let file_name = package
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "skill.ozskill".to_string());

    let sig_b64 = if sig_path.exists() {
        let sig_data = std::fs::read(&sig_path)?;
        Some(BASE64.encode(&sig_data))
    } else {
        None
    };

    let body = serde_json::json!({
        "filename": file_name,
        "package": BASE64.encode(&package_data),
        "signature": sig_b64,
    });

    match client
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            let data: serde_json::Value = r.json().await.unwrap_or_default();
            println!("  {} Published successfully!", "✓".green().bold());
            if let Some(skill_url) = data["url"].as_str() {
                println!("  {} {}", "→".bright_cyan(), skill_url.bright_cyan());
            }
        }
        Ok(r) => {
            anyhow::bail!("Marketplace error: {}", r.status());
        }
        Err(_) => {
            println!("  {} Marketplace offline", "✗".red().bold());
        }
    }

    println!();
    Ok(())
}

fn skill_inspect(path: &PathBuf) -> anyhow::Result<()> {
    use std::io::Read;
    use zip::ZipArchive;

    ui::print_info(&format!("Inspecting: {}", path.display()));
    println!();

    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read manifest
    let manifest_data = {
        let mut mf = archive.by_name("manifest.json").map_err(|_| {
            anyhow::anyhow!("manifest.json not found in package – may not be a valid .ozskill")
        })?;
        let mut data = Vec::new();
        mf.read_to_end(&mut data)?;
        data
    };

    let manifest: serde_json::Value = serde_json::from_slice(&manifest_data)?;

    println!("  {} Manifest", "◆".bright_cyan().bold());
    println!(
        "    Name        : {}",
        manifest["name"]
            .as_str()
            .unwrap_or("?")
            .bright_white()
            .bold()
    );
    println!(
        "    Version     : {}",
        manifest["version"].as_str().unwrap_or("?").bright_yellow()
    );
    println!(
        "    Description : {}",
        manifest["description"].as_str().unwrap_or("").dimmed()
    );
    println!(
        "    Author      : {}",
        manifest["author"].as_str().unwrap_or("?")
    );
    println!(
        "    License     : {}",
        manifest["license"].as_str().unwrap_or("?")
    );
    println!();

    if let Some(perms) = manifest["permissions"].as_array() {
        println!(
            "  {} Permissions ({})",
            "◆".bright_cyan().bold(),
            perms.len()
        );
        for p in perms {
            let perm = p.as_str().unwrap_or("?");
            let icon = if perm.starts_with("net:") || perm.starts_with("http:") {
                "[net]"
            } else if perm.starts_with("fs:") {
                "[fs]"
            } else {
                "[key]"
            };
            println!("    {} {}", icon, perm.bright_yellow());
        }
        println!();
    }

    // WASM info
    if let Ok(mut wasm_entry) = archive.by_name("skill.wasm") {
        let mut wasm_data = Vec::new();
        wasm_entry.read_to_end(&mut wasm_data)?;

        println!("  {} WASM Module", "◆".bright_cyan().bold());
        println!(
            "    Size        : {} bytes ({:.1} KB)",
            wasm_data.len(),
            wasm_data.len() as f64 / 1024.0
        );

        // Check magic bytes
        if wasm_data.starts_with(b"\0asm") {
            let version =
                u32::from_le_bytes([wasm_data[4], wasm_data[5], wasm_data[6], wasm_data[7]]);
            println!("    WASM version: {}", version);
            println!("    {} Valid WASM module", "✓".green());
        } else {
            println!("    {} Invalid WASM magic bytes", "✗".red());
        }
    }

    // Other files in the package
    let file_count = archive.len();
    if file_count > 2 {
        println!();
        println!(
            "  {} Additional files ({})",
            "◆".bright_cyan().bold(),
            file_count - 2
        );
        for i in 0..file_count {
            let entry = archive.by_index(i)?;
            let name = entry.name().to_string();
            if name != "manifest.json" && name != "skill.wasm" {
                println!("    • {} ({} bytes)", name, entry.size());
            }
        }
    }

    println!();
    Ok(())
}

fn skill_validate(path: &PathBuf) -> anyhow::Result<()> {
    use std::io::Read;
    use zip::ZipArchive;

    ui::print_info(&format!("Validating: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    // Try opening as zip
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => anyhow::bail!("Cannot open file: {}", e),
    };
    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => anyhow::bail!("Not a valid ZIP archive (.ozskill must be a ZIP file)"),
    };

    // Check manifest.json
    let has_manifest = archive.by_name("manifest.json").is_ok();
    if !has_manifest {
        errors.push("Missing manifest.json".to_string());
    } else {
        let mut mf = archive.by_name("manifest.json").unwrap();
        let mut data = Vec::new();
        mf.read_to_end(&mut data).ok();
        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(manifest) => {
                for field in &["name", "version", "description", "author"] {
                    if manifest[field]
                        .as_str()
                        .map(|s| s.is_empty())
                        .unwrap_or(true)
                    {
                        errors.push(format!("manifest.json missing required field: {}", field));
                    }
                }
                if manifest["permissions"].as_array().is_none() {
                    warnings.push(
                        "manifest.json: 'permissions' field missing (defaulting to empty)"
                            .to_string(),
                    );
                }
            }
            Err(e) => {
                errors.push(format!("manifest.json is not valid JSON: {}", e));
            }
        }
    }

    // Check skill.wasm
    let has_wasm = archive.by_name("skill.wasm").is_ok();
    if !has_wasm {
        errors.push("Missing skill.wasm".to_string());
    } else {
        let mut wasm_entry = archive.by_name("skill.wasm").unwrap();
        let mut magic = [0u8; 8];
        if wasm_entry.read_exact(&mut magic).is_ok() && !magic.starts_with(b"\0asm") {
            errors.push("skill.wasm has invalid WASM magic bytes".to_string());
        }
        if wasm_entry.size() < 8 {
            errors.push("skill.wasm is too small to be a valid WASM module".to_string());
        }
    }

    // Report
    for w in &warnings {
        println!("  {} {}", "⚠".bright_yellow().bold(), w);
    }
    for e in &errors {
        println!("  {} {}", "✗".red().bold(), e);
    }

    if errors.is_empty() {
        println!("  {} Package is valid!", "✓".green().bold());
        if !warnings.is_empty() {
            println!("  {} {} warning(s)", "⚠".bright_yellow(), warnings.len());
        }
    } else {
        println!();
        println!("  {} {} error(s) found", "✗".red().bold(), errors.len());
        std::process::exit(1);
    }

    println!();
    Ok(())
}

// ── MCP commands ──────────────────────────────────────────────────────────────

async fn handle_mcp_command(cmd: McpCommands) -> anyhow::Result<()> {
    match cmd {
        McpCommands::Simulate { server } => mcp_simulate(server).await,
        McpCommands::Inspect { server } => mcp_inspect(server).await,
        McpCommands::Record { server, output } => mcp_record(server, output).await,
    }
}

async fn mcp_simulate(server_name: String) -> anyhow::Result<()> {
    use std::io::{BufRead, Write};

    println!(
        "  {} Starting mock MCP server '{}' on stdio",
        "→".bright_cyan(),
        server_name.bright_white().bold()
    );
    println!(
        "  {} Send JSON-RPC requests on stdin, responses on stdout",
        "◆".bright_cyan()
    );
    println!();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mock_tools = serde_json::json!([
        {
            "name": "echo",
            "description": "Echoes the input back",
            "inputSchema": {
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"]
            }
        },
        {
            "name": "time",
            "description": "Returns the current UTC time",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ]);

    let mock_resources = serde_json::json!([
        {
            "uri": "mock://readme",
            "name": "README",
            "description": "Mock README resource",
            "mimeType": "text/plain"
        }
    ]);

    let server_info = serde_json::json!({
        "name": server_name,
        "version": "1.0.0"
    });

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                writeln!(stdout.lock(), "{}", serde_json::to_string(&err_resp)?)?;
                continue;
            }
        };

        let id = request.get("id").cloned();
        let method = request["method"].as_str().unwrap_or("");

        let result: serde_json::Value = match method {
            "initialize" => serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": server_info
            }),
            "notifications/initialized" => {
                continue;
            }
            "tools/list" => serde_json::json!({ "tools": mock_tools }),
            "resources/list" => serde_json::json!({ "resources": mock_resources }),
            "tools/call" => {
                let tool_name = request["params"]["name"].as_str().unwrap_or("");
                match tool_name {
                    "echo" => {
                        let msg = request["params"]["arguments"]["message"]
                            .as_str()
                            .unwrap_or("");
                        serde_json::json!({
                            "content": [{ "type": "text", "text": msg }]
                        })
                    }
                    "time" => serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": chrono::Utc::now().to_rfc3339()
                        }]
                    }),
                    _ => serde_json::json!({
                        "isError": true,
                        "content": [{ "type": "text", "text": format!("Unknown tool: {}", tool_name) }]
                    }),
                }
            }
            "resources/read" => {
                let uri = request["params"]["uri"].as_str().unwrap_or("");
                match uri {
                    "mock://readme" => serde_json::json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "text/plain",
                            "text": format!("Mock MCP server '{}' – OpenZax CLI", server_name)
                        }]
                    }),
                    _ => serde_json::json!({ "contents": [] }),
                }
            }
            _ => {
                let err_resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Method not found: {}", method) }
                });
                writeln!(stdout.lock(), "{}", serde_json::to_string(&err_resp)?)?;
                continue;
            }
        };

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        });

        writeln!(stdout.lock(), "{}", serde_json::to_string(&response)?)?;
    }

    Ok(())
}

async fn mcp_inspect(server: String) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    println!(
        "  {} Inspecting MCP server: {}",
        "→".bright_cyan(),
        server.bright_white().bold()
    );
    println!();

    let parts: Vec<&str> = server.split_whitespace().collect();
    let (cmd, args) = parts
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("Empty server command"))?;

    let mut child = tokio::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    let send_and_receive = async {
        // Send initialize
        let init = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "openzax-cli", "version": env!("CARGO_PKG_VERSION") }
            }
        });
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&init)?).as_bytes())
            .await?;

        // Wait for initialize response
        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let resp: serde_json::Value = serde_json::from_str(&line)?;
            if resp["id"].as_u64() == Some(1) {
                if let Some(info) = resp["result"]["serverInfo"].as_object() {
                    println!(
                        "  {} Server: {} {}",
                        "✓".green().bold(),
                        info["name"].as_str().unwrap_or("?").bright_white().bold(),
                        info["version"].as_str().unwrap_or("").dimmed()
                    );
                }
                if let Some(caps) = resp["result"]["capabilities"].as_object() {
                    let cap_list: Vec<&str> = caps.keys().map(|k| k.as_str()).collect();
                    println!(
                        "  {} Capabilities: {}",
                        "◆".bright_cyan(),
                        cap_list.join(", ").bright_yellow()
                    );
                }
                println!();
                break;
            }
        }

        // Send initialized notification
        let notif = serde_json::json!({
            "jsonrpc": "2.0", "method": "notifications/initialized"
        });
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&notif)?).as_bytes())
            .await?;

        // Request tools/list
        let tools_req = serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/list"
        });
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&tools_req)?).as_bytes())
            .await?;

        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let resp: serde_json::Value = serde_json::from_str(&line)?;
            if resp["id"].as_u64() == Some(2) {
                if let Some(tools) = resp["result"]["tools"].as_array() {
                    println!("  {} Tools ({}):", "◆".bright_cyan().bold(), tools.len());
                    for t in tools {
                        println!(
                            "    {} {}  – {}",
                            "•".bright_cyan(),
                            t["name"].as_str().unwrap_or("?").bright_white().bold(),
                            t["description"].as_str().unwrap_or("").dimmed()
                        );
                    }
                    println!();
                }
                break;
            }
        }

        // Request resources/list
        let res_req = serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "resources/list"
        });
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&res_req)?).as_bytes())
            .await?;

        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let resp: serde_json::Value = serde_json::from_str(&line)?;
            if resp["id"].as_u64() == Some(3) {
                if let Some(resources) = resp["result"]["resources"].as_array() {
                    println!(
                        "  {} Resources ({}):",
                        "◆".bright_cyan().bold(),
                        resources.len()
                    );
                    for r in resources {
                        println!(
                            "    {} {}  [{}]",
                            "•".bright_cyan(),
                            r["name"].as_str().unwrap_or("?").bright_white().bold(),
                            r["uri"].as_str().unwrap_or("?").dimmed()
                        );
                    }
                    println!();
                }
                break;
            }
        }

        anyhow::Ok(())
    };

    let result = tokio::time::timeout(std::time::Duration::from_secs(10), send_and_receive).await;
    let _ = child.kill().await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => anyhow::bail!("MCP communication error: {}", e),
        Err(_) => {
            println!(
                "  {} Timed out waiting for server response",
                "⚠".bright_yellow().bold()
            );
        }
    }

    Ok(())
}

async fn mcp_record(server: String, output: PathBuf) -> anyhow::Result<()> {
    use std::io::Write;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    println!(
        "  {} Recording MCP session: {}",
        "→".bright_cyan(),
        server.bright_white().bold()
    );
    println!("  {} Output: {}", "◆".bright_cyan(), output.display());
    println!();

    let parts: Vec<&str> = server.split_whitespace().collect();
    let (cmd, args) = parts
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("Empty server command"))?;

    let mut child = tokio::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let mut child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(child_stdout).lines();

    let mut out_file = std::fs::File::create(&output)?;
    let mut msg_count = 0u32;

    let record_msg = |file: &mut std::fs::File, direction: &str, content: &str, count: &mut u32| {
        let entry = serde_json::json!({
            "seq": count,
            "direction": direction,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "content": serde_json::from_str::<serde_json::Value>(content)
                .unwrap_or_else(|_| serde_json::Value::String(content.to_string()))
        });
        writeln!(
            file,
            "{}",
            serde_json::to_string(&entry).unwrap_or_default()
        )
        .ok();
        *count += 1;
    };

    // Send initialize
    let init = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "openzax-cli", "version": env!("CARGO_PKG_VERSION") }
        }
    });
    let init_str = serde_json::to_string(&init)?;
    record_msg(&mut out_file, "client→server", &init_str, &mut msg_count);
    child_stdin
        .write_all(format!("{}\n", init_str).as_bytes())
        .await?;

    // Record session until timeout or EOF
    let session = async {
        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            record_msg(&mut out_file, "server→client", &line, &mut msg_count);
            println!("  {} {}", "←".bright_green(), line.dimmed());
        }
        anyhow::Ok(())
    };

    let _ = tokio::time::timeout(std::time::Duration::from_secs(30), session).await;
    let _ = child.kill().await;

    println!();
    println!(
        "  {} Recorded {} messages to {}",
        "✓".green().bold(),
        msg_count,
        output.display()
    );
    println!();

    Ok(())
}

// ── model commands ────────────────────────────────────────────────────────────

#[cfg(feature = "llm-engine")]
async fn handle_model_command(cmd: ModelCommands) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let expand_home = |path: PathBuf| -> PathBuf {
        if let Ok(stripped) = path.strip_prefix("~") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        path
    };

    match cmd {
        ModelCommands::List { models_dir } => {
            let models_dir = expand_home(models_dir);
            let manager = LocalModelManager::new(&models_dir);
            println!("Discovering models in: {}", models_dir.display());
            println!();
            let models = manager.discover_models()?;
            if models.is_empty() {
                println!("No models found.");
                println!("\nTo download a model, use:");
                println!("  openzax model download <model-name>");
            } else {
                println!("Found {} model(s):\n", models.len());
                for model in models {
                    println!("  • {} ({})", model.name, model.id);
                    if let Some(size) = model.size_bytes {
                        let size_gb = size as f64 / 1_073_741_824.0;
                        println!("    Size: {:.2} GB", size_gb);
                    }
                    if let Some(quant) = &model.quantization {
                        println!("    Quantization: {}", quant);
                    }
                    println!("    Context: {} tokens", model.context_window);
                    println!("    Capabilities: {:?}", model.capabilities);
                    if let Some(path) = &model.path {
                        println!("    Path: {}", path.display());
                    }
                    println!();
                }
            }
        }
        ModelCommands::Download { name, models_dir } => {
            let models_dir = expand_home(models_dir);
            println!("Model download feature is coming soon!");
            println!("\nFor now, you can manually download GGUF models from:");
            println!("  • Hugging Face: https://huggingface.co/models?library=gguf");
            println!("  • TheBloke's models: https://huggingface.co/TheBloke");
            println!("\nPlace .gguf files in: {}", models_dir.display());
            println!("\nPopular models:");
            println!("  • llama-3.3-70b-q4_k_m.gguf (recommended)");
            println!("  • mistral-7b-instruct-v0.2-q4_k_m.gguf");
            println!("  • codellama-13b-instruct-q4_k_m.gguf");
            let _ = name;
        }
        ModelCommands::Info { name, models_dir } => {
            let models_dir = expand_home(models_dir);
            let manager = LocalModelManager::new(&models_dir);
            let models = manager.discover_models()?;
            if let Some(model) = models
                .iter()
                .find(|m| m.id == name || m.name.contains(&name))
            {
                println!("Model Information:");
                println!("  ID: {}", model.id);
                println!("  Name: {}", model.name);
                println!("  Provider: {:?}", model.provider);
                println!("  Context Window: {} tokens", model.context_window);
                if let Some(size) = model.size_bytes {
                    let size_gb = size as f64 / 1_073_741_824.0;
                    let size_mb = size as f64 / 1_048_576.0;
                    println!("  Size: {:.2} GB ({:.0} MB)", size_gb, size_mb);
                }
                if let Some(quant) = &model.quantization {
                    println!("  Quantization: {}", quant);
                }
                println!("  Capabilities: {:?}", model.capabilities);
                println!("  Local: {}", model.is_local);
                if let Some(path) = &model.path {
                    println!("  Path: {}", path.display());
                }
            } else {
                eprintln!("Model '{}' not found.", name);
                eprintln!("\nUse 'openzax model list' to see available models.");
                std::process::exit(1);
            }
        }
        ModelCommands::Remove {
            name,
            models_dir,
            yes,
        } => {
            let models_dir = expand_home(models_dir);
            let manager = LocalModelManager::new(&models_dir);
            let models = manager.discover_models()?;
            if let Some(model) = models
                .iter()
                .find(|m| m.id == name || m.name.contains(&name))
            {
                if let Some(path) = &model.path {
                    if !yes {
                        print!(
                            "Are you sure you want to remove '{}' ({})? [y/N] ",
                            model.name,
                            path.display()
                        );
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }
                    std::fs::remove_file(path)?;
                    println!("Removed: {}", path.display());
                } else {
                    eprintln!("Error: Model path not found.");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Model '{}' not found.", name);
                eprintln!("\nUse 'openzax model list' to see available models.");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "llm-engine"))]
async fn handle_model_command(_cmd: ModelCommands) -> anyhow::Result<()> {
    eprintln!("Error: Model management requires the 'llm-engine' feature.");
    eprintln!("Rebuild with: cargo build --features llm-engine");
    std::process::exit(1);
}

// ── skill scaffold ────────────────────────────────────────────────────────────

fn create_rust_skill(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    use std::fs;

    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(output_dir.join("src"))?;
    fs::create_dir_all(output_dir.join(".cargo"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
openzax-skills-sdk = {{ git = "https://github.com/zAxCoder/OpenZax.git" }}

[profile.release]
opt-level = "z"
lto = true
strip = true
"#,
        name
    );
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;

    let cargo_config = r#"[build]
target = "wasm32-wasip1"

[target.wasm32-wasip1]
rustflags = ["-C", "link-arg=--export-table"]
"#;
    fs::write(output_dir.join(".cargo/config.toml"), cargo_config)?;

    let lib_rs = format!(
        r#"use openzax_skills_sdk::{{skill, SkillContext, SkillResult}};

#[skill]
pub fn execute(ctx: &SkillContext) -> SkillResult {{
    let input = ctx.get_input()?;
    
    Ok(format!("Hello from {}! Input: {{}}", input))
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use openzax_skills_sdk::SkillContext;

    #[test]
    fn test_execute() {{
        let ctx = SkillContext::new("test input");
        let result = execute(&ctx);
        assert!(result.is_ok());
    }}
}}
"#,
        name
    );
    fs::write(output_dir.join("src/lib.rs"), lib_rs)?;

    let manifest = serde_json::json!({
        "name": name,
        "version": "0.1.0",
        "description": format!("A skill created with OpenZax CLI"),
        "author": "Your Name",
        "license": "MIT",
        "permissions": []
    });
    fs::write(
        output_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    let readme = format!(
        r#"# {}

A WebAssembly skill for OpenZax.

## Building

```bash
cargo build --release --target wasm32-wasip1
```

Or use the CLI:

```bash
openzax skill build --release
```

## Testing

```bash
cargo test
```

## Packaging

```bash
openzax skill pack
```

## Publishing

```bash
openzax skill publish {}-0.1.0.ozskill --key your-key.private.key
```
"#,
        name, name
    );
    fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn openzax_config_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home.join(".openzax");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        })
        .collect()
}
