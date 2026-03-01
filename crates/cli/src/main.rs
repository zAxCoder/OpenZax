use clap::{Parser, Subcommand};
use openzax_core::{agent::AgentConfig, storage::Storage};
use openzax_shell::TerminalShell;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use colored::Colorize;

#[cfg(feature = "llm-engine")]
use openzax_llm_engine::local::LocalModelManager;

mod ui;

#[derive(Parser)]
#[command(name = "openzax")]
#[command(about = "OpenZax - Secure AI Development Assistant", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive terminal shell
    Shell {
        /// API key for LLM provider
        #[arg(short, long)]
        api_key: Option<String>,

        /// Model to use
        #[arg(short, long, default_value = "gpt-4")]
        model: String,

        /// Database path
        #[arg(short, long, default_value = ".openzax/openzax.db")]
        db_path: PathBuf,
    },

    /// Initialize a new skill project
    Init {
        /// Skill name
        name: String,

        /// Programming language
        #[arg(short, long, default_value = "rust")]
        language: String,
    },

    /// Model management commands
    #[command(subcommand)]
    Model(ModelCommands),

    /// Skill management commands
    #[command(subcommand)]
    Skill(SkillCommands),

    /// Display version information
    Version,
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Initialize a new skill project
    Init {
        /// Skill name
        name: String,

        /// Programming language (rust, typescript, python)
        #[arg(short, long, default_value = "rust")]
        language: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build a skill to WASM
    Build {
        /// Skill directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Release build
        #[arg(short, long)]
        release: bool,
    },

    /// Test a skill
    Test {
        /// Skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Pack a skill for distribution
    Pack {
        /// Skill directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Sign a skill package
    Sign {
        /// Skill package path
        package: PathBuf,

        /// Private key path
        #[arg(short, long)]
        key: PathBuf,
    },

    /// Publish a skill to marketplace
    Publish {
        /// Skill package path
        package: PathBuf,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available local models
    List {
        /// Models directory
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Download a model from Hugging Face
    Download {
        /// Model name or Hugging Face repo ID
        name: String,

        /// Models directory
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Show detailed information about a model
    Info {
        /// Model name or path
        name: String,

        /// Models directory
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,
    },

    /// Remove a local model
    Remove {
        /// Model name
        name: String,

        /// Models directory
        #[arg(short, long, default_value = "~/.openzax/models")]
        models_dir: PathBuf,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        Some(Commands::Shell { api_key, model, db_path }) => {
            // Print beautiful banner
            ui::print_banner();
            
            if api_key.is_none() {
                ui::print_error("API key is required");
                println!();
                ui::print_info("Set OPENZAX_API_KEY environment variable or use --api-key flag");
                println!();
                std::process::exit(1);
            }

            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let config = AgentConfig {
                api_key,
                model: model.clone(),
                ..Default::default()
            };

            ui::print_model_info(&model, "OpenAI");
            ui::print_welcome();

            let storage = Storage::new(&db_path)?;
            let shell = TerminalShell::new(config, storage)?;
            
            shell.run().await?;
        }
        Some(Commands::Init { name, language }) => {
            ui::print_banner();
            ui::print_info(&format!("Initializing new {} skill: {}", language, name));
            println!();
            ui::print_info("This feature is coming soon in Phase 2!");
            println!();
        }
        Some(Commands::Model(model_cmd)) => {
            ui::print_banner();
            handle_model_command(model_cmd).await?;
        }
        Some(Commands::Skill(skill_cmd)) => {
            ui::print_banner();
            handle_skill_command(skill_cmd).await?;
        }
        Some(Commands::Version) => {
            ui::print_banner();
            println!("  {} {}", "Version:".bright_white().dimmed(), env!("CARGO_PKG_VERSION").bright_cyan().bold());
            println!("  {} {}", "Description:".bright_white().dimmed(), "Rust-native AI development assistant".bright_white());
            println!();
        }
        None => {
            ui::print_banner();
            println!("  {} {}", "Version:".bright_white().dimmed(), env!("CARGO_PKG_VERSION").bright_cyan());
            println!();
            ui::print_info("Use 'openzax --help' for usage information");
            println!();
            println!("  {} Quick start:", "→".bright_cyan());
            println!("    {}", "openzax shell --api-key YOUR_API_KEY".bright_yellow());
            println!();
        }
    }

    Ok(())
}

async fn handle_skill_command(cmd: SkillCommands) -> anyhow::Result<()> {
    use std::io::Write;

    match cmd {
        SkillCommands::Init { name, language, output } => {
            let output_dir = output.unwrap_or_else(|| PathBuf::from(&name));
            
            ui::print_info(&format!("Creating new {} skill: {}", language, name));
            println!();
            
            match language.as_str() {
                "rust" => create_rust_skill(&name, &output_dir)?,
                "typescript" | "ts" => {
                    ui::print_error("TypeScript skills coming soon!");
                    println!();
                    ui::print_info("For now, use Rust skills with: openzax skill init --language rust");
                    println!();
                    return Ok(());
                }
                "python" | "py" => {
                    ui::print_error("Python skills coming soon!");
                    println!();
                    ui::print_info("For now, use Rust skills with: openzax skill init --language rust");
                    println!();
                    return Ok(());
                }
                _ => {
                    ui::print_error(&format!("Unsupported language: {}", language));
                    println!();
                    ui::print_info("Supported languages: rust, typescript, python");
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
                println!();
                
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
            ui::print_info(&format!("Packing skill at: {}", path.display()));
            println!();
            ui::print_error("Pack feature coming soon!");
            println!();
        }
        SkillCommands::Sign { package, key } => {
            ui::print_info(&format!("Signing package: {}", package.display()));
            println!();
            ui::print_error("Sign feature coming soon!");
            println!();
        }
        SkillCommands::Publish { package } => {
            ui::print_info(&format!("Publishing package: {}", package.display()));
            println!();
            ui::print_error("Publish feature coming soon (requires marketplace backend)!");
            println!();
        }
    }
    
    Ok(())
}

fn create_rust_skill(name: &str, output_dir: &PathBuf) -> anyhow::Result<()> {
    use std::fs;
    
    // Create directory structure
    fs::create_dir_all(output_dir)?;
    fs::create_dir_all(output_dir.join("src"))?;
    fs::create_dir_all(output_dir.join(".cargo"))?;
    
    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
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
"#, name);
    
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create .cargo/config.toml
    let cargo_config = r#"[build]
target = "wasm32-wasip1"

[target.wasm32-wasip1]
rustflags = ["-C", "link-arg=--export-table"]
"#;
    
    fs::write(output_dir.join(".cargo/config.toml"), cargo_config)?;
    
    // Create src/lib.rs
    let lib_rs = r#"use openzax_skills_sdk::{skill_main, SkillContext, SkillResult};

#[skill_main]
fn run() -> SkillResult<()> {
    let ctx = SkillContext::new();
    
    ctx.log_info("Hello from OpenZax skill!");
    ctx.log_info("This is a minimal skill template.");
    
    // Add your skill logic here
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run() {
        assert!(run().is_ok());
    }
}
"#;
    
    fs::write(output_dir.join("src/lib.rs"), lib_rs)?;
    
    // Create README.md
    let readme = format!(r#"# {}

A skill for OpenZax.

## Building

```bash
openzax skill build
```

## Testing

```bash
openzax skill test
```

## Running

```bash
openzax skill build --release
# Load the skill in OpenZax
```
"#, name);
    
    fs::write(output_dir.join("README.md"), readme)?;
    
    // Create build.sh
    let build_sh = r#"#!/bin/bash
cargo build --target wasm32-wasip1 --release
"#;
    
    fs::write(output_dir.join("build.sh"), build_sh)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output_dir.join("build.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output_dir.join("build.sh"), perms)?;
    }
    
    Ok(())
}

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
                        let path_display = path.display();
                        println!("    Path: {}", path_display);
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
        }
        ModelCommands::Info { name, models_dir } => {
            let models_dir = expand_home(models_dir);
            let manager = LocalModelManager::new(&models_dir);
            
            let models = manager.discover_models()?;
            
            if let Some(model) = models.iter().find(|m| m.id == name || m.name.contains(&name)) {
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
                    let path_display = path.display();
                    println!("  Path: {}", path_display);
                }
                
                #[cfg(feature = "llama-cpp")]
                {
                    println!("\nGPU Information:");
                    let gpu_info = openzax_llm_engine::local::llama::detect_gpu();
                    println!("  CUDA: {}", if gpu_info.has_cuda { "Available" } else { "Not available" });
                    println!("  Metal: {}", if gpu_info.has_metal { "Available" } else { "Not available" });
                    println!("  Vulkan: {}", if gpu_info.has_vulkan { "Available" } else { "Not available" });
                    if gpu_info.vram_mb > 0 {
                        println!("  VRAM: {} MB ({:.2} GB)", gpu_info.vram_mb, gpu_info.vram_mb as f64 / 1024.0);
                    }
                }
            } else {
                eprintln!("Model '{}' not found.", name);
                eprintln!("\nUse 'openzax model list' to see available models.");
                std::process::exit(1);
            }
        }
        ModelCommands::Remove { name, models_dir, yes } => {
            let models_dir = expand_home(models_dir);
            let manager = LocalModelManager::new(&models_dir);
            
            let models = manager.discover_models()?;
            
            if let Some(model) = models.iter().find(|m| m.id == name || m.name.contains(&name)) {
                if let Some(path) = &model.path {
                    if !yes {
                        let path_display = path.display();
                        print!("Are you sure you want to remove '{}' ({})? [y/N] ", model.name, path_display);
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
