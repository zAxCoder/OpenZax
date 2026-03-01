# OpenZax LLM Engine

Multi-model AI inference engine with local and cloud model support.

## Features

- **Model Router**: Intelligent model selection based on latency, cost, capability, and quality
- **Local Models**: llama.cpp integration for local inference (optional)
- **Cloud Models**: Support for OpenAI, Anthropic, Google, and custom APIs
- **GPU Acceleration**: CUDA, Metal, and Vulkan support
- **Model Management**: Discovery, loading, and hot-swapping

## Usage

### Model Router

```rust
use openzax_llm_engine::{ModelRouter, RouterConfig, Model, ModelInfo};

// Create router
let config = RouterConfig::default();
let router = ModelRouter::new(config);

// Register models
let model = Model::new(ModelInfo {
    id: "gpt-4".to_string(),
    name: "GPT-4".to_string(),
    provider: ModelProvider::OpenAI,
    context_window: 8192,
    capabilities: vec![ModelCapability::Chat, ModelCapability::Code],
    // ...
});

router.register_model(model).await;

// Select best model
let best = router.select_best_model(
    &[ModelCapability::Chat],
    1000 // context size
).await?;
```

### Local Models (with llama-cpp feature)

```rust
use openzax_llm_engine::local::{LocalModelManager, llama::LlamaModel};

// Discover models
let manager = LocalModelManager::new("~/.openzax/models");
let models = manager.discover_models()?;

// Load model
let model = LlamaModel::load("~/.openzax/models/llama-3.3-70b-q4.gguf")?;

// Generate
let response = model.generate("Hello, world!", 100).await?;
```

### Cloud Models

```rust
use openzax_llm_engine::cloud::CloudProvider;

let provider = CloudProvider::new(
    "https://api.openai.com/v1/chat/completions".to_string(),
    "your-api-key".to_string(),
    "gpt-4".to_string(),
);

let response = provider.generate("Hello!", 100).await?;
```

## Features

- `default`: Core functionality only
- `llama-cpp`: Enable local model support via llama.cpp
- `cuda`: CUDA GPU acceleration
- `metal`: Metal GPU acceleration (macOS)
- `vulkan`: Vulkan GPU acceleration

## Model Router Scoring

The router selects models using a weighted scoring function:

```
score = w_latency * (1 - normalized_latency)
      + w_cost * (1 - normalized_cost)
      + w_capability * capability_match
      + w_quality * quality_score
```

Default weights:
- Latency: 0.25
- Cost: 0.20
- Capability: 0.30
- Quality: 0.25

## GPU Detection

```rust
use openzax_llm_engine::local::llama::detect_gpu;

let gpu_info = detect_gpu();
println!("CUDA: {}", gpu_info.has_cuda);
println!("Metal: {}", gpu_info.has_metal);
println!("Vulkan: {}", gpu_info.has_vulkan);
println!("VRAM: {} MB", gpu_info.vram_mb);
```

## Model Management

```bash
# List models
openzax model list

# Download model
openzax model download llama-3.3-70b-q4

# Show model info
openzax model info llama-3.3-70b-q4

# Remove model
openzax model remove llama-3.3-70b-q4
```
