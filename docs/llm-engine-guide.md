# OpenZax LLM Engine Guide

## Overview

The LLM Engine provides intelligent multi-model routing and management for both local and cloud AI models.

## Quick Start

### Model Router

```rust
use openzax_llm_engine::{ModelRouter, RouterConfig, Model, ModelInfo};
use openzax_llm_engine::model::{ModelProvider, ModelCapability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create router with default config
    let config = RouterConfig::default();
    let router = ModelRouter::new(config);

    // Register a model
    let model = Model::new(ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: ModelProvider::OpenAI,
        context_window: 8192,
        capabilities: vec![
            ModelCapability::Chat,
            ModelCapability::Code,
            ModelCapability::Reasoning,
        ],
        size_bytes: None,
        quantization: None,
        is_local: false,
        path: None,
    });

    router.register_model(model).await;

    // Select best model for task
    let best = router.select_best_model(
        &[ModelCapability::Chat, ModelCapability::Code],
        2000 // context size needed
    ).await?;

    println!("Selected: {}", best.info.name);
    Ok(())
}
```

## Model Router

### Scoring Algorithm

The router selects models using a weighted scoring function:

```
score = w_latency * (1 - normalized_latency)
      + w_cost * (1 - normalized_cost)
      + w_capability * capability_match
      + w_quality * quality_score
```

### Configuration

```rust
let config = RouterConfig {
    weight_latency: 0.25,    // Prioritize speed
    weight_cost: 0.20,       // Consider cost
    weight_capability: 0.30, // Match capabilities
    weight_quality: 0.25,    // Consider quality
};
```

### Adjusting Weights

```rust
// Prioritize speed
let fast_config = RouterConfig {
    weight_latency: 0.50,
    weight_cost: 0.10,
    weight_capability: 0.20,
    weight_quality: 0.20,
};

// Prioritize cost
let cheap_config = RouterConfig {
    weight_latency: 0.10,
    weight_cost: 0.50,
    weight_capability: 0.20,
    weight_quality: 0.20,
};

// Prioritize quality
let quality_config = RouterConfig {
    weight_latency: 0.10,
    weight_cost: 0.10,
    weight_capability: 0.20,
    weight_quality: 0.60,
};
```

## Local Models

### Model Discovery

```rust
use openzax_llm_engine::local::LocalModelManager;

let manager = LocalModelManager::new("~/.openzax/models");
let models = manager.discover_models()?;

for model in models {
    println!("Found: {} ({} MB)", 
             model.name, 
             model.size_bytes.unwrap_or(0) / 1_000_000);
}
```

### Model Loading (with llama-cpp feature)

```rust
#[cfg(feature = "llama-cpp")]
use openzax_llm_engine::local::llama::LlamaModel;

let model = LlamaModel::load("~/.openzax/models/llama-3.3-70b-q4.gguf")?;
let response = model.generate("Hello, world!", 100).await?;
```

### GPU Detection

```rust
#[cfg(feature = "llama-cpp")]
use openzax_llm_engine::local::llama::detect_gpu;

let gpu_info = detect_gpu();
println!("CUDA available: {}", gpu_info.has_cuda);
println!("Metal available: {}", gpu_info.has_metal);
println!("Vulkan available: {}", gpu_info.has_vulkan);
println!("VRAM: {} MB", gpu_info.vram_mb);
```

## Cloud Models

### OpenAI

```rust
use openzax_llm_engine::cloud::CloudProvider;

let provider = CloudProvider::new(
    "https://api.openai.com/v1/chat/completions".to_string(),
    std::env::var("OPENAI_API_KEY")?,
    "gpt-4".to_string(),
);

let response = provider.generate("Hello!", 100).await?;
```

### Anthropic

```rust
let provider = CloudProvider::new(
    "https://api.anthropic.com/v1/messages".to_string(),
    std::env::var("ANTHROPIC_API_KEY")?,
    "claude-3-opus-20240229".to_string(),
);
```

### Custom API

```rust
let provider = CloudProvider::new(
    "https://your-api.com/v1/completions".to_string(),
    "your-api-key".to_string(),
    "your-model".to_string(),
);
```

## Model Management

### Registering Models

```rust
// Local model
let local_model = Model::new(ModelInfo {
    id: "llama-3.3-70b-q4".to_string(),
    name: "Llama 3.3 70B Q4".to_string(),
    provider: ModelProvider::Local,
    context_window: 8192,
    capabilities: vec![
        ModelCapability::Chat,
        ModelCapability::Code,
    ],
    size_bytes: Some(40_000_000_000), // 40 GB
    quantization: Some("q4_k_m".to_string()),
    is_local: true,
    path: Some("~/.openzax/models/llama-3.3-70b-q4.gguf".into()),
});

router.register_model(local_model).await;

// Cloud model
let cloud_model = Model::new(ModelInfo {
    id: "gpt-4".to_string(),
    name: "GPT-4".to_string(),
    provider: ModelProvider::OpenAI,
    context_window: 8192,
    capabilities: vec![
        ModelCapability::Chat,
        ModelCapability::Code,
        ModelCapability::Reasoning,
        ModelCapability::Vision,
    ],
    size_bytes: None,
    quantization: None,
    is_local: false,
    path: None,
});

router.register_model(cloud_model).await;
```

### Updating Model Stats

```rust
// After inference, update latency
router.update_model_stats("gpt-4", 1250.0).await?; // 1.25 seconds
```

### Listing Models

```rust
let models = router.list_models().await;
for model in models {
    println!("{}: {} tokens, {:?}", 
             model.info.name,
             model.info.context_window,
             model.info.capabilities);
}
```

## Model Capabilities

```rust
pub enum ModelCapability {
    Chat,       // General conversation
    Code,       // Code generation and understanding
    Reasoning,  // Complex reasoning tasks
    Vision,     // Image understanding
    ToolUse,    // Function calling
    Streaming,  // Streaming responses
}
```

## Cost Estimation

```rust
let mut model = Model::new(model_info);
model.input_cost_per_1m = Some(10.0);  // $10 per 1M input tokens
model.output_cost_per_1m = Some(30.0); // $30 per 1M output tokens

let cost = model.estimate_cost(1000, 500); // 1000 input, 500 output
println!("Estimated cost: ${:.4}", cost.unwrap());
```

## Best Practices

### 1. Register All Available Models

```rust
// Register both local and cloud models
router.register_model(local_llama).await;
router.register_model(gpt4).await;
router.register_model(claude).await;

// Router will automatically select the best one
```

### 2. Update Stats Regularly

```rust
let start = std::time::Instant::now();
let response = provider.generate(prompt, max_tokens).await?;
let latency = start.elapsed().as_millis() as f64;

router.update_model_stats(&model_id, latency).await?;
```

### 3. Handle Context Window Limits

```rust
let context_size = estimate_tokens(prompt);
let max_window = 8192;

if context_size > max_window {
    // Compress context or use a model with larger window
    let large_context_model = router.select_best_model(
        &capabilities,
        context_size
    ).await?;
}
```

### 4. Fallback Strategy

```rust
let primary = router.select_best_model(&capabilities, context_size).await;

match primary {
    Ok(model) => {
        // Use primary model
    }
    Err(_) => {
        // Fallback to any available model
        let fallback = router.list_models().await
            .into_iter()
            .find(|m| m.info.context_window >= context_size);
    }
}
```

## Features

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
openzax-llm-engine = { version = "0.3", features = ["llama-cpp", "cuda"] }
```

Available features:
- `llama-cpp`: Enable local model support
- `cuda`: CUDA GPU acceleration
- `metal`: Metal GPU acceleration (macOS)
- `vulkan`: Vulkan GPU acceleration

## CLI Commands (Planned)

```bash
# List models
openzax model list

# Download model
openzax model download llama-3.3-70b-q4

# Show model info
openzax model info llama-3.3-70b-q4

# Remove model
openzax model remove llama-3.3-70b-q4

# Test model
openzax model test llama-3.3-70b-q4 "Hello, world!"
```

## Troubleshooting

### "Model not found"

Ensure the model is registered:
```rust
router.register_model(model).await;
```

### "Context window exceeded"

Use a model with larger context window:
```rust
let model = router.select_best_model(&capabilities, larger_context).await?;
```

### GPU not detected

Check that:
1. GPU drivers are installed
2. Correct feature is enabled (`cuda`, `metal`, or `vulkan`)
3. llama-cpp is compiled with GPU support

## Next Steps

- See [MCP Client Guide](./mcp-client-guide.md)
- Read [WASM Runtime Guide](./wasm-runtime-guide.md)
- Explore [Master Architecture](./master-architecture-blueprint.md)
