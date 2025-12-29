# Tokscale Architecture Refactor: Rust-First Pricing & Remove TS Fallback

## Purpose & Background

### Why This Refactor?

**Problem**: The current architecture has a TypeScript fallback path for session parsing that runs when the native Rust module is unavailable or when specific features (like `agent` field access) require it. This creates:

1. **Maintenance Burden**: Every feature must be implemented twice (Rust + TypeScript)
2. **Inconsistent Behavior**: TS and Rust parsers can diverge over time
3. **Slower Development**: New platform support requires duplicate work
4. **Performance Variability**: Users get vastly different performance depending on which path runs

**Issue #59 Context**: The TypeScript session fallback was originally added as a development convenience and fallback for unsupported platforms. However, with native binaries now available for all major platforms (macOS, Linux, Windows - both x64 and ARM), the fallback is no longer needed and adds complexity.

**The `forceTypescript` Workaround**: `wrapped.ts` uses `forceTypescript: true` to access the `agent` field, which was originally only in TS parsers. The Rust module now fully supports `agent` (see `lib.rs:80`, `sessions/opencode.rs:62-79`), making this workaround obsolete.

### What Does Success Look Like?

**For Users**:
- No visible behavior changes
- Same CLI commands work identically
- Potentially faster startup (no TS fallback check)

**For Developers**:
- Single implementation per feature (Rust only)
- Clear error when native module missing: "Run: bun run build:core"
- Simpler codebase: ~1500 lines of TypeScript deleted

### Success Metrics

| Metric | Target |
|--------|--------|
| Cold pricing fetch | < 500ms |
| Cached pricing fetch | < 50ms |
| Session parsing | Same as current (~100-200ms for typical) |
| All existing tests | Pass |
| All CLI commands | Identical behavior |

---

## Overview

Consolidate all heavy computation (session parsing, pricing fetch, cost calculation) into Rust.
TypeScript becomes a thin CLI/TUI layer for user interaction only.

**Goals:**
1. Move pricing fetching to Rust (LiteLLM + OpenRouter)
2. Remove TypeScript session fallback (Issue #59 - see Purpose section above)
3. Single source of truth for model aliases/mappings
4. Parallel execution for maximum performance

**Phase Dependencies:**
```
Phase 1 (Rust Pricing) ─────────────────┐
                                        ├──► Phase 3 (Parallel Flow)
Phase 2 (Remove TS) ────────────────────┘        │
        │                                        │
        └── BLOCKED until Phase 1 cargo test ◄───┘
                     passes
Phase 4 (Testing) ── Runs after Phase 2 complete
```

**CRITICAL**: Phase 2 (deleting TypeScript files) MUST NOT begin until ALL Phase 1 Rust tests pass. Deleting TS fallback before Rust implementation is complete will break the CLI.

---

## Phase 1: Rust Pricing Module

### 1.1 Add Dependencies to Cargo.toml

```toml
# HTTP client (async)
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

# For disk caching
dirs = "5"  # XDG paths

# Async utilities
futures = "0.3"

# Lazy static initialization
once_cell = "1"
```

### 1.2 Create Pricing Module Structure

```
packages/core/src/
├── lib.rs                      # Add pricing module, new NAPI exports
├── pricing/
│   ├── mod.rs                  # Module entry, PricingService
│   ├── cache.rs                # Disk cache (XDG paths, 1hr TTL)
│   ├── litellm.rs              # LiteLLM fetcher
│   ├── openrouter.rs           # OpenRouter fetcher + provider mappings
│   ├── aliases.rs              # Model aliases (big-pickle → glm-4.7, etc.)
│   └── lookup.rs               # Unified lookup (fuzzy match, normalization)
├── pricing.rs                  # DELETE (replaced by pricing/ module)
└── ...
```

### 1.3 File: `pricing/aliases.rs`

Single source of truth for model aliases and OpenRouter mappings.

**NOTE on `big-pickle` alias:**
- Current TypeScript maps `big-pickle` → `glm-4.6`
- However, actual OpenCode session data shows model ID `big-pickle` used with `glm-4.7-free`
- OpenRouter only has `z-ai/glm-4.7` (no glm-4.6 endpoint)
- **Decision**: Map to `glm-4.7` to match OpenRouter availability

```rust
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Model aliases: user-facing name → canonical model ID
/// Note: "big-pickle" maps to "glm-4.7" (not glm-4.6) because OpenRouter
/// only provides z-ai/glm-4.7 endpoint. This is intentional.
pub static MODEL_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("big-pickle", "glm-4.7");
    m.insert("big pickle", "glm-4.7");
    m.insert("bigpickle", "glm-4.7");
    m
});

/// OpenRouter model ID mappings: canonical ID → OpenRouter provider/model
/// Example: "glm-4.7" → "z-ai/glm-4.7"
pub static OPENROUTER_MAPPINGS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("glm-4.7", "z-ai/glm-4.7");
    m.insert("glm-4.7-free", "z-ai/glm-4.7");
    m
});

/// OpenRouter provider name mappings: author → display name
/// Example: "z-ai" → "Z.AI"
pub static OPENROUTER_PROVIDER_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("z-ai", "Z.AI");
    m
});

/// Resolve alias to canonical model ID
pub fn resolve_alias(model_id: &str) -> Option<&'static str> {
    MODEL_ALIASES.get(model_id.to_lowercase().as_str()).copied()
}

/// Get OpenRouter model ID for a canonical model
pub fn get_openrouter_id(model_id: &str) -> Option<&'static str> {
    OPENROUTER_MAPPINGS.get(model_id.to_lowercase().as_str()).copied()
}
```

### 1.4 File: `pricing/cache.rs`

Disk caching with XDG paths and 1-hour TTL.

```rust
use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};

const CACHE_TTL_SECS: u64 = 3600; // 1 hour

pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("tokscale")
}

pub fn get_cache_path(filename: &str) -> PathBuf {
    get_cache_dir().join(filename)
}

#[derive(Serialize, Deserialize)]
pub struct CachedData<T> {
    pub timestamp: u64,
    pub data: T,
}

pub fn load_cache<T: for<'de> Deserialize<'de>>(filename: &str) -> Option<T> {
    let path = get_cache_path(filename);
    let content = fs::read_to_string(&path).ok()?;
    let cached: CachedData<T> = serde_json::from_str(&content).ok()?;
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if now - cached.timestamp > CACHE_TTL_SECS {
        return None; // Expired
    }
    
    Some(cached.data)
}

pub fn save_cache<T: Serialize>(filename: &str, data: &T) -> Result<(), std::io::Error> {
    let dir = get_cache_dir();
    fs::create_dir_all(&dir)?;
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let cached = CachedData { timestamp: now, data };
    let content = serde_json::to_string(&cached)?;
    fs::write(get_cache_path(filename), content)
}
```

### 1.5 File: `pricing/litellm.rs`

Fetch LiteLLM pricing from GitHub.

```rust
use super::cache;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const CACHE_FILENAME: &str = "pricing-litellm.json";
const PRICING_URL: &str = "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelPricing {
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_creation_input_token_cost: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
}

pub type PricingDataset = HashMap<String, ModelPricing>;

pub fn load_cached() -> Option<PricingDataset> {
    cache::load_cache(CACHE_FILENAME)
}

pub async fn fetch() -> Result<PricingDataset, reqwest::Error> {
    // Try cache first
    if let Some(cached) = load_cached() {
        return Ok(cached);
    }
    
    // Fetch from network
    let client = reqwest::Client::new();
    let data: PricingDataset = client
        .get(PRICING_URL)
        .send()
        .await?
        .json()
        .await?;
    
    // Save to cache (ignore errors)
    let _ = cache::save_cache(CACHE_FILENAME, &data);
    
    Ok(data)
}
```

### 1.6 File: `pricing/openrouter.rs`

Fetch OpenRouter pricing for specific models.

```rust
use super::{cache, aliases};
use super::litellm::ModelPricing;
use std::collections::HashMap;
use serde::Deserialize;

const CACHE_FILENAME: &str = "pricing-openrouter.json";

#[derive(Deserialize)]
struct EndpointPricing {
    prompt: String,
    completion: String,
    input_cache_read: Option<String>,
    input_cache_write: Option<String>,
}

#[derive(Deserialize)]
struct Endpoint {
    provider_name: String,
    pricing: EndpointPricing,
}

#[derive(Deserialize)]
struct EndpointsData {
    endpoints: Vec<Endpoint>,
}

#[derive(Deserialize)]
struct EndpointsResponse {
    data: EndpointsData,
}

pub fn load_cached() -> Option<HashMap<String, ModelPricing>> {
    cache::load_cache(CACHE_FILENAME)
}

/// Fetch pricing for a specific OpenRouter model
async fn fetch_model_endpoints(
    client: &reqwest::Client,
    author: &str,
    slug: &str,
) -> Option<ModelPricing> {
    let url = format!("https://openrouter.ai/api/v1/models/{}/{}/endpoints", author, slug);
    
    let response = client.get(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .ok()?;
    
    if !response.status().is_success() {
        return None;
    }
    
    let data: EndpointsResponse = response.json().await.ok()?;
    
    // Find the author's own endpoint
    let expected_provider = aliases::OPENROUTER_PROVIDER_NAMES
        .get(author)
        .copied()
        .unwrap_or(author);
    
    let endpoint = data.data.endpoints.iter()
        .find(|e| e.provider_name.eq_ignore_ascii_case(expected_provider))?;
    
    let input_cost: f64 = endpoint.pricing.prompt.parse().ok()?;
    let output_cost: f64 = endpoint.pricing.completion.parse().ok()?;
    
    if input_cost < 0.0 || output_cost < 0.0 {
        return None; // TBD pricing
    }
    
    Some(ModelPricing {
        input_cost_per_token: Some(input_cost),
        output_cost_per_token: Some(output_cost),
        cache_read_input_token_cost: endpoint.pricing.input_cache_read
            .as_ref()
            .and_then(|s| s.parse().ok()),
        cache_creation_input_token_cost: endpoint.pricing.input_cache_write
            .as_ref()
            .and_then(|s| s.parse().ok()),
    })
}

/// Fetch pricing for all mapped OpenRouter models
pub async fn fetch_all_mapped() -> HashMap<String, ModelPricing> {
    // Try cache first
    if let Some(cached) = load_cached() {
        return cached;
    }
    
    let client = reqwest::Client::new();
    let mut result = HashMap::new();
    
    // Get unique OpenRouter IDs
    let unique_ids: std::collections::HashSet<&str> = 
        aliases::OPENROUTER_MAPPINGS.values().copied().collect();
    
    // Fetch in parallel using tokio::join_all
    let futures: Vec<_> = unique_ids.iter().map(|id| {
        let client = client.clone();
        let id = id.to_string();
        async move {
            let parts: Vec<&str> = id.split('/').collect();
            if parts.len() == 2 {
                let pricing = fetch_model_endpoints(&client, parts[0], parts[1]).await;
                pricing.map(|p| (id, p))
            } else {
                None
            }
        }
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    for res in results.into_iter().flatten() {
        result.insert(res.0, res.1);
    }
    
    // Save to cache
    if !result.is_empty() {
        let _ = cache::save_cache(CACHE_FILENAME, &result);
    }
    
    result
}

/// Fetch pricing for specific missing models (dynamic)
pub async fn fetch_missing(model_ids: &[String]) -> HashMap<String, ModelPricing> {
    let client = reqwest::Client::new();
    let mut result = HashMap::new();
    
    for model_id in model_ids {
        // Check if we have an OpenRouter mapping
        if let Some(or_id) = aliases::get_openrouter_id(model_id) {
            let parts: Vec<&str> = or_id.split('/').collect();
            if parts.len() == 2 {
                if let Some(pricing) = fetch_model_endpoints(&client, parts[0], parts[1]).await {
                    result.insert(model_id.clone(), pricing);
                }
            }
        }
    }
    
    result
}
```

### 1.7 File: `pricing/lookup.rs`

Unified pricing lookup with fuzzy matching.

```rust
use super::{aliases, litellm::ModelPricing};
use std::collections::HashMap;

const PROVIDER_PREFIXES: &[&str] = &["anthropic/", "openai/", "google/", "bedrock/", "openrouter/"];

pub struct PricingLookup {
    litellm: HashMap<String, ModelPricing>,
    openrouter: HashMap<String, ModelPricing>,
    sorted_keys: Vec<String>,
}

pub struct LookupResult {
    pub pricing: ModelPricing,
    pub source: String,       // "litellm" or "openrouter"
    pub matched_key: String,
}

impl PricingLookup {
    pub fn new(litellm: HashMap<String, ModelPricing>, openrouter: HashMap<String, ModelPricing>) -> Self {
        let mut sorted_keys: Vec<String> = litellm.keys().cloned().collect();
        sorted_keys.sort();
        
        Self { litellm, openrouter, sorted_keys }
    }
    
    /// Look up pricing for a model ID
    pub fn lookup(&self, model_id: &str) -> Option<LookupResult> {
        // 1. Resolve aliases first
        let canonical = aliases::resolve_alias(model_id).unwrap_or(model_id);
        
        // 2. Try LiteLLM direct lookup
        if let Some(result) = self.lookup_litellm(canonical) {
            return Some(result);
        }
        
        // 3. Try OpenRouter for mapped models
        if let Some(result) = self.lookup_openrouter(canonical) {
            return Some(result);
        }
        
        None
    }
    
    fn lookup_litellm(&self, model_id: &str) -> Option<LookupResult> {
        // Direct lookup
        if let Some(p) = self.litellm.get(model_id) {
            return Some(LookupResult {
                pricing: p.clone(),
                source: "litellm".into(),
                matched_key: model_id.into(),
            });
        }
        
        // Try with provider prefixes
        for prefix in PROVIDER_PREFIXES {
            let key = format!("{}{}", prefix, model_id);
            if let Some(p) = self.litellm.get(&key) {
                return Some(LookupResult {
                    pricing: p.clone(),
                    source: "litellm".into(),
                    matched_key: key,
                });
            }
        }
        
        // Normalize (Cursor-style names)
        if let Some(normalized) = normalize_model_name(model_id) {
            if let Some(p) = self.litellm.get(&normalized) {
                return Some(LookupResult {
                    pricing: p.clone(),
                    source: "litellm".into(),
                    matched_key: normalized,
                });
            }
            for prefix in PROVIDER_PREFIXES {
                let key = format!("{}{}", prefix, normalized);
                if let Some(p) = self.litellm.get(&key) {
                    return Some(LookupResult {
                        pricing: p.clone(),
                        source: "litellm".into(),
                        matched_key: key,
                    });
                }
            }
        }
        
        // Fuzzy matching
        self.fuzzy_match_litellm(model_id)
    }
    
    fn fuzzy_match_litellm(&self, model_id: &str) -> Option<LookupResult> {
        let lower = model_id.to_lowercase();
        let normalized = normalize_model_name(model_id);
        let lower_normalized = normalized.as_ref().map(|s| s.to_lowercase());
        
        for key in &self.sorted_keys {
            let lower_key = key.to_lowercase();
            if is_word_boundary_match(&lower_key, &lower) {
                return Some(LookupResult {
                    pricing: self.litellm.get(key).unwrap().clone(),
                    source: "litellm".into(),
                    matched_key: key.clone(),
                });
            }
            if let Some(ref ln) = lower_normalized {
                if is_word_boundary_match(&lower_key, ln) {
                    return Some(LookupResult {
                        pricing: self.litellm.get(key).unwrap().clone(),
                        source: "litellm".into(),
                        matched_key: key.clone(),
                    });
                }
            }
        }
        
        None
    }
    
    fn lookup_openrouter(&self, model_id: &str) -> Option<LookupResult> {
        let or_id = aliases::get_openrouter_id(model_id)?;
        let pricing = self.openrouter.get(or_id)?;
        
        Some(LookupResult {
            pricing: pricing.clone(),
            source: "openrouter".into(),
            matched_key: or_id.into(),
        })
    }
    
    /// Calculate cost for tokens
    pub fn calculate_cost(&self, model_id: &str, input: i64, output: i64, cache_read: i64, cache_write: i64, reasoning: i64) -> f64 {
        let result = match self.lookup(model_id) {
            Some(r) => r,
            None => return 0.0,
        };
        
        let p = &result.pricing;
        let input_cost = input as f64 * p.input_cost_per_token.unwrap_or(0.0);
        let output_cost = (output + reasoning) as f64 * p.output_cost_per_token.unwrap_or(0.0);
        let cache_read_cost = cache_read as f64 * p.cache_read_input_token_cost.unwrap_or(0.0);
        let cache_write_cost = cache_write as f64 * p.cache_creation_input_token_cost.unwrap_or(0.0);
        
        input_cost + output_cost + cache_read_cost + cache_write_cost
    }
}

fn normalize_model_name(model_id: &str) -> Option<String> {
    let lower = model_id.to_lowercase();
    
    // Claude models
    if lower.contains("opus") {
        if lower.contains("4.5") || lower.contains("4-5") {
            return Some("opus-4-5".into());
        } else if lower.contains("4") {
            return Some("opus-4".into());
        }
    }
    if lower.contains("sonnet") {
        if lower.contains("4.5") || lower.contains("4-5") {
            return Some("sonnet-4-5".into());
        } else if lower.contains("4") {
            return Some("sonnet-4".into());
        } else if lower.contains("3.7") || lower.contains("3-7") {
            return Some("sonnet-3-7".into());
        } else if lower.contains("3.5") || lower.contains("3-5") {
            return Some("sonnet-3-5".into());
        }
    }
    if lower.contains("haiku") && (lower.contains("4.5") || lower.contains("4-5")) {
        return Some("haiku-4-5".into());
    }
    
    // OpenAI
    if lower == "o3" { return Some("o3".into()); }
    if lower.starts_with("gpt-4o") || lower == "gpt-4o" { return Some("gpt-4o".into()); }
    if lower.starts_with("gpt-4.1") || lower.contains("gpt-4.1") { return Some("gpt-4.1".into()); }
    
    // Gemini
    if lower.contains("gemini-2.5-pro") { return Some("gemini-2.5-pro".into()); }
    if lower.contains("gemini-2.5-flash") { return Some("gemini-2.5-flash".into()); }
    
    None
}

fn is_word_boundary_match(haystack: &str, needle: &str) -> bool {
    if let Some(pos) = haystack.find(needle) {
        let before_ok = pos == 0 || !haystack[..pos].chars().last().unwrap().is_alphanumeric();
        let after_ok = pos + needle.len() == haystack.len() || 
            !haystack[pos + needle.len()..].chars().next().unwrap().is_alphanumeric();
        before_ok && after_ok
    } else {
        false
    }
}
```

### 1.8 File: `pricing/mod.rs`

Module entry point with PricingService.

```rust
pub mod aliases;
pub mod cache;
pub mod litellm;
pub mod lookup;
pub mod openrouter;

use lookup::{PricingLookup, LookupResult};
use std::collections::HashMap;

pub use litellm::ModelPricing;

/// Main pricing service - fetches, caches, and looks up pricing
pub struct PricingService {
    lookup: PricingLookup,
}

impl PricingService {
    /// Create service with existing data
    pub fn new(litellm_data: HashMap<String, ModelPricing>, openrouter_data: HashMap<String, ModelPricing>) -> Self {
        Self {
            lookup: PricingLookup::new(litellm_data, openrouter_data),
        }
    }
    
    /// Fetch all pricing data (parallel: LiteLLM + OpenRouter mapped models)
    pub async fn fetch() -> Result<Self, String> {
        // Parallel fetch
        let (litellm_result, openrouter_data) = tokio::join!(
            litellm::fetch(),
            openrouter::fetch_all_mapped()
        );
        
        let litellm_data = litellm_result.map_err(|e| e.to_string())?;
        
        Ok(Self::new(litellm_data, openrouter_data))
    }
    
    /// Look up pricing for a model
    pub fn lookup(&self, model_id: &str) -> Option<LookupResult> {
        self.lookup.lookup(model_id)
    }
    
    /// Calculate cost
    pub fn calculate_cost(&self, model_id: &str, input: i64, output: i64, cache_read: i64, cache_write: i64, reasoning: i64) -> f64 {
        self.lookup.calculate_cost(model_id, input, output, cache_read, cache_write, reasoning)
    }
}
```

### 1.9 Update `lib.rs` - New NAPI Exports

```rust
mod pricing;

// New async NAPI exports

/// Look up pricing for a model (for `tokscale pricing <model>` command)
#[napi]
pub async fn lookup_pricing(model_id: String) -> napi::Result<PricingLookupResult> {
    let service = pricing::PricingService::fetch()
        .await
        .map_err(|e| napi::Error::from_reason(e))?;
    
    match service.lookup(&model_id) {
        Some(result) => Ok(PricingLookupResult {
            model_id,
            matched_key: result.matched_key,
            source: result.source,
            pricing: NativePricing {
                input_cost_per_token: result.pricing.input_cost_per_token.unwrap_or(0.0),
                output_cost_per_token: result.pricing.output_cost_per_token.unwrap_or(0.0),
                cache_read_input_token_cost: result.pricing.cache_read_input_token_cost,
                cache_creation_input_token_cost: result.pricing.cache_creation_input_token_cost,
            },
        }),
        None => Err(napi::Error::from_reason("Model not found")),
    }
}

#[napi(object)]
pub struct NativePricing {
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    pub cache_read_input_token_cost: Option<f64>,
    pub cache_creation_input_token_cost: Option<f64>,
}

#[napi(object)]
pub struct PricingLookupResult {
    pub model_id: String,
    pub matched_key: String,
    pub source: String,
    pub pricing: NativePricing,
}

/// Parse all messages WITHOUT pricing calculation (cost = 0.0)
/// This is extracted from existing `parse_all_messages_with_pricing` (lib.rs:514)
/// but skips cost calculation - that happens after we fetch pricing.
fn parse_all_messages_raw(home_dir: &str, sources: &[String]) -> Vec<UnifiedMessage> {
    // Implementation follows exact pattern of parse_all_messages_with_pricing (lib.rs:514-610)
    // but WITHOUT the pricing_data.calculate_cost() calls.
    // Each source parser (opencode, claude, codex, gemini, amp, droid) returns
    // UnifiedMessage with cost = 0.0 (the default from UnifiedMessage::new)
    
    let scan_result = scanner::scan_all_sources(home_dir, sources);
    let mut all_messages: Vec<UnifiedMessage> = Vec::new();
    
    // Pattern: scan_result.{source}_files.par_iter() → parse → collect
    // See lib.rs:523-610 for full implementation
    // Clone that pattern but remove all `msg.cost = pricing_data.calculate_cost(...)` lines
    
    all_messages
}

/// Generate full report (sessions + pricing - all in Rust)
/// This replaces the TS flow of: fetch pricing → pass to Rust → get report
/// Now: Rust fetches pricing internally + parses sessions in parallel
#[napi]
pub async fn generate_full_report(options: FullReportOptions) -> napi::Result<ModelReport> {
    let start = std::time::Instant::now();
    
    // Get home directory and sources
    let home_dir = get_home_dir(&options.home_dir)?;
    let sources = options.sources.clone().unwrap_or_else(|| vec![
        "opencode".to_string(), "claude".to_string(), "codex".to_string(),
        "gemini".to_string(), "cursor".to_string(), "amp".to_string(), "droid".to_string(),
    ]);
    
    // PARALLEL: Parse sessions (CPU-bound) + Fetch pricing (I/O-bound)
    // tokio::join! runs both concurrently
    let (messages_result, pricing_result) = tokio::join!(
        // spawn_blocking for CPU-bound rayon work (prevents blocking async runtime)
        tokio::task::spawn_blocking({
            let home_dir = home_dir.clone();
            let sources = sources.clone();
            move || parse_all_messages_raw(&home_dir, &sources)
        }),
        // Async HTTP fetch for pricing
        pricing::PricingService::fetch()
    );
    
    let messages = messages_result
        .map_err(|e| napi::Error::from_reason(format!("Session parsing failed: {}", e)))?;
    let pricing_service = pricing_result
        .map_err(|e| napi::Error::from_reason(format!("Pricing fetch failed: {}", e)))?;
    
    // Apply date filters using existing filter_messages_for_report logic (lib.rs:911-934)
    // Convert to ReportOptions-compatible format for reuse
    let filtered = {
        let mut msgs = messages;
        // Filter by year
        if let Some(year) = &options.year {
            let year_prefix = format!("{}-", year);
            msgs.retain(|m| m.date.starts_with(&year_prefix));
        }
        // Filter by since date
        if let Some(since) = &options.since {
            msgs.retain(|m| m.date.as_str() >= since.as_str());
        }
        // Filter by until date
        if let Some(until) = &options.until {
            msgs.retain(|m| m.date.as_str() <= until.as_str());
        }
        msgs
    };
    
    // Calculate costs using pricing service (CPU-bound with rayon)
    // Must use spawn_blocking since we're in async context
    let messages_with_costs = tokio::task::spawn_blocking({
        let pricing = pricing_service;
        move || {
            filtered.into_par_iter().map(|mut msg| {
                msg.cost = pricing.calculate_cost(
                    &msg.model_id,
                    msg.tokens.input,
                    msg.tokens.output,
                    msg.tokens.cache_read,
                    msg.tokens.cache_write,
                    msg.tokens.reasoning,
                );
                msg
            }).collect::<Vec<_>>()
        }
    }).await.map_err(|e| napi::Error::from_reason(e.to_string()))?;
    
    // Aggregate by model (same logic as get_model_report, lib.rs:728-753)
    let mut model_map: HashMap<String, ModelUsage> = HashMap::new();
    for msg in &messages_with_costs {
        let key = format!("{}:{}:{}", msg.source, msg.provider_id, msg.model_id);
        let entry = model_map.entry(key).or_insert_with(|| ModelUsage {
            source: msg.source.clone(),
            model: msg.model_id.clone(),
            provider: msg.provider_id.clone(),
            input: 0, output: 0, cache_read: 0, cache_write: 0, reasoning: 0,
            message_count: 0, cost: 0.0,
        });
        entry.input += msg.tokens.input;
        entry.output += msg.tokens.output;
        entry.cache_read += msg.tokens.cache_read;
        entry.cache_write += msg.tokens.cache_write;
        entry.reasoning += msg.tokens.reasoning;
        entry.message_count += 1;
        entry.cost += msg.cost;
    }
    
    let mut entries: Vec<ModelUsage> = model_map.into_values().collect();
    entries.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(ModelReport {
        entries: entries.clone(),
        total_input: entries.iter().map(|e| e.input).sum(),
        total_output: entries.iter().map(|e| e.output).sum(),
        total_cache_read: entries.iter().map(|e| e.cache_read).sum(),
        total_cache_write: entries.iter().map(|e| e.cache_write).sum(),
        total_messages: entries.iter().map(|e| e.message_count).sum(),
        total_cost: entries.iter().map(|e| e.cost).sum(),
        processing_time_ms: start.elapsed().as_millis() as u32,
    })
}

#[napi(object)]
pub struct FullReportOptions {
    pub home_dir: Option<String>,
    pub sources: Option<Vec<String>>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub year: Option<String>,
    pub include_cursor: bool,
}
```

### 1.10 Error Handling Acceptance Criteria

Network operations in pricing fetchers must handle these failure modes gracefully:

#### Network Timeout
- **Behavior**: 30-second timeout per HTTP request
- **Fallback**: Return cached data if available, otherwise continue with empty pricing
- **User Impact**: Cost calculations show $0.00 for unknown models (not an error)

```rust
// In reqwest client configuration
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;
```

#### Rate Limiting (429)
- **Behavior**: Exponential backoff with max 3 retries (1s, 2s, 4s)
- **Fallback**: After 3 retries, use cached data or continue without pricing
- **Logging**: Warn-level log message

```rust
async fn fetch_with_retry<T>(url: &str) -> Result<T, PricingError> {
    let delays = [1, 2, 4]; // seconds
    for (i, delay) in delays.iter().enumerate() {
        match client.get(url).send().await {
            Ok(resp) if resp.status() == 429 => {
                if i < delays.len() - 1 {
                    tokio::time::sleep(Duration::from_secs(*delay)).await;
                    continue;
                }
            }
            Ok(resp) => return resp.json().await.map_err(Into::into),
            Err(e) => return Err(e.into()),
        }
    }
    Err(PricingError::RateLimited)
}
```

#### Server Errors (5xx)
- **Behavior**: Single retry after 2 seconds
- **Fallback**: Use cached data if available
- **Logging**: Warn-level log message with status code

#### Cache Miss + Network Failure
- **Behavior**: Return empty pricing dataset (not an error)
- **User Impact**: All costs show $0.00, CLI displays warning:
  ```
  ⚠ Unable to fetch pricing data. Costs will show as $0.00.
  ```

#### Acceptance Criteria Summary

| Scenario | Retries | Fallback | User Experience |
|----------|---------|----------|-----------------|
| Network timeout | 0 | Cache → empty | Silent if cached, warning if not |
| 429 rate limit | 3 (exp backoff) | Cache → empty | Silent if cached, warning if not |
| 5xx server error | 1 | Cache → empty | Silent if cached, warning if not |
| Invalid JSON | 0 | Cache → empty | Warning logged |
| Cache valid | N/A | Use cache | Instant response |

---

## Phase 2: Remove TypeScript Fallback & Pricing

### 2.1 Files to DELETE

```
packages/cli/src/
├── pricing/                    # DELETE entire directory
│   ├── index.ts
│   ├── utils.ts
│   └── providers/
│       ├── litellm.ts
│       └── openrouter.ts
├── sessions/                   # DELETE all except types.ts
│   ├── amp.ts                  # DELETE
│   ├── claudecode.ts           # DELETE
│   ├── codex.ts                # DELETE
│   ├── droid.ts                # DELETE
│   ├── gemini.ts               # DELETE
│   ├── index.ts                # DELETE
│   ├── opencode.ts             # DELETE
│   ├── reports.ts              # DELETE
│   └── types.ts                # KEEP (shared types for TS consumers)
```

### 2.2 Simplify `native.ts`

Remove all fallback logic. Require native module.

**Before:**
```typescript
export async function parseLocalSourcesAsync(options: LocalParseOptions): Promise<ParsedMessages> {
  if (!isNativeAvailable() || options.forceTypescript) {
    // ... TypeScript fallback
  }
  return runInSubprocess<ParsedMessages>("parseLocalSources", [nativeOptions]);
}
```

**After:**
```typescript
export async function parseLocalSourcesAsync(options: LocalParseOptions): Promise<ParsedMessages> {
  if (!isNativeAvailable()) {
    throw new Error("Native module required. Run: bun run build:core");
  }
  return runInSubprocess<ParsedMessages>("parseLocalSources", [nativeOptions]);
}
```

**NOTE on `forceTypescript` removal:**

The `forceTypescript` option was added to `wrapped.ts` (line 228) as a workaround to access the `agent` field, which was originally only available in the TypeScript session parsers.

**This workaround is NO LONGER NEEDED** because the Rust native module now fully supports the `agent` field:
- `lib.rs:80` - `NativeMessage` struct has `pub agent: Option<String>`
- `lib.rs:1077` and `lib.rs:1120` - Agent field is populated when building messages
- `sessions/opencode.rs:62-79` - Agent field is parsed and normalized (including "OmO" → "Sisyphus" mapping)

The `wrapped.ts` code that forced TypeScript fallback:
```typescript
// OLD: Force TS fallback when agents are needed
forceTypescript: options.includeAgents !== false
```

Can be safely removed because Rust provides the same agent data.

**Remove:**
- `parseLocalSourcesTS` import
- `generateModelReportTS`, `generateMonthlyReportTS`, `generateGraphDataTS` imports
- `buildMessagesForFallback` function
- All `!isNativeAvailable()` branches that call TS fallback
- `forceTypescript` option

### 2.3 Update `cli.ts` - Pricing Command

Change to use Rust `lookup_pricing`:

```typescript
async function handlePricingCommand(modelId: string, options: { json?: boolean; provider?: string }) {
  // Use Rust native module
  const core = await import("@tokscale/core");
  
  try {
    const result = await core.lookupPricing(modelId);
    // ... display result
  } catch (err) {
    // Model not found
  }
}
```

### 2.4 Update Imports in Other Files

Files that import from `./pricing/index.js`:
- `cli.ts` → Use Rust `lookupPricing`
- `submit.ts` → Use Rust `generateFullReport`
- `wrapped.ts` → Use Rust
- `graph.ts` → Use Rust
- `native.ts` → Remove `PricingEntry` import (not needed)
- `sessions/reports.ts` → DELETE file

---

## Phase 3: Parallel Execution Flow

### 3.1 New Main Flow (All in Rust)

```
┌─────────────────────────────────────────────────────────────┐
│  tokio::join! (Parallel)                                    │
│                                                             │
│  Task 1: scan_all_sources()     → file paths               │
│  Task 2: litellm::fetch()       → LiteLLM pricing (cached) │
│  Task 3: openrouter::fetch_all_mapped() → OpenRouter       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  rayon (Parallel CPU)                                       │
│                                                             │
│  files.par_iter()                                           │
│    .filter_map(|path| parse_session(path))                 │
│    .collect()                                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  Sequential (fast)                                          │
│                                                             │
│  1. Collect unique model IDs from messages                  │
│  2. Find missing (not in LiteLLM or OpenRouter cache)       │
│  3. Fetch missing from OpenRouter (parallel HTTP)           │
│  4. Build PricingLookup                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  rayon (Parallel CPU)                                       │
│                                                             │
│  messages.par_iter()                                        │
│    .map(|msg| {                                             │
│        msg.cost = pricing.calculate_cost(msg);              │
│        msg                                                  │
│    })                                                       │
│    .collect()                                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  Aggregate & Return                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 4: Testing & Verification

### 4.1 Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alias_resolution() {
        assert_eq!(aliases::resolve_alias("big-pickle"), Some("glm-4.7"));
        assert_eq!(aliases::resolve_alias("BIG-PICKLE"), Some("glm-4.7"));
    }
    
    #[test]
    fn test_openrouter_mapping() {
        assert_eq!(aliases::get_openrouter_id("glm-4.7"), Some("z-ai/glm-4.7"));
        assert_eq!(aliases::get_openrouter_id("glm-4.7-free"), Some("z-ai/glm-4.7"));
    }
    
    #[tokio::test]
    async fn test_pricing_fetch() {
        let service = PricingService::fetch().await.unwrap();
        
        // Claude should be in LiteLLM
        let result = service.lookup("claude-3-5-sonnet-20241022");
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, "litellm");
        
        // big-pickle should resolve via alias + OpenRouter
        let result = service.lookup("big-pickle");
        assert!(result.is_some());
    }
}
```

### 4.2 CLI Verification

Each command below shows expected output format. Actual values will vary.

```bash
# Pricing lookup - Claude (should hit LiteLLM)
$ tokscale pricing "claude-3-5-sonnet-20241022"
# Expected output:
#   Model: claude-3-5-sonnet-20241022
#   Matched: claude-3-5-sonnet-20241022
#   Source: litellm
#   Input:  $3.00 / 1M tokens
#   Output: $15.00 / 1M tokens
#   Cache Read: $0.30 / 1M tokens
#   Cache Write: $3.75 / 1M tokens

# Pricing lookup - Alias (should resolve via aliases.rs + OpenRouter)
$ tokscale pricing "big-pickle"
# Expected output:
#   Model: big-pickle
#   Matched: z-ai/glm-4.7
#   Source: openrouter
#   Input:  $X.XX / 1M tokens
#   Output: $X.XX / 1M tokens

# Pricing lookup - Model not found
$ tokscale pricing "nonexistent-model-xyz"
# Expected output:
#   Error: Model not found: nonexistent-model-xyz
# Exit code: 1

# Reports should work without TS fallback
$ tokscale models
# Expected: Table with model usage (same format as before)
# Verify: No TypeScript fallback warning in output

$ tokscale monthly
# Expected: Monthly breakdown table (same format as before)

$ tokscale graph --output test.json
# Expected: JSON file created successfully
# Verify: File contains { "data": [...], "year": "2025", ... }

# TUI should launch without issues
$ tokscale tui
# Expected: Interactive TUI launches, all tabs work

# Wrapped should work (was using forceTypescript for agents)
$ tokscale wrapped --year 2025
# Expected: PNG image generated with agent breakdown
# Verify: Agent names appear correctly (Sisyphus, etc.)
```

### 4.3 Regression Testing

```bash
# Run existing test suite
$ cd packages/core && cargo test
# Expected: All tests pass

# Run CLI type checking
$ cd packages/cli && bun run tsc --noEmit
# Expected: No TypeScript errors

# Verify native module loads correctly
$ bunx tokscale@latest --version
# Expected: Version number (no "native module not found" warning)
```

---

## Implementation Checklist

### Rust Changes
- [ ] Add `reqwest`, `tokio`, `dirs`, `futures`, `once_cell` to Cargo.toml
- [ ] Create `pricing/` module directory
- [ ] Implement `pricing/aliases.rs`
- [ ] Implement `pricing/cache.rs`
- [ ] Implement `pricing/litellm.rs`
- [ ] Implement `pricing/openrouter.rs`
- [ ] Implement `pricing/lookup.rs`
- [ ] Implement `pricing/mod.rs` with PricingService
- [ ] Add `lookup_pricing` NAPI export
- [ ] Add `generate_full_report` NAPI export (optional, for cleaner API)
- [ ] Update existing report functions to use new pricing module
- [ ] Delete old `pricing.rs`
- [ ] Add tests
- [ ] Run `cargo build --release`
- [ ] Run `cargo test`

### TypeScript Changes
- [x] Delete `packages/cli/src/pricing/` directory
- [x] Delete `packages/cli/src/sessions/amp.ts`
- [x] Delete `packages/cli/src/sessions/claudecode.ts`
- [x] Delete `packages/cli/src/sessions/codex.ts`
- [x] Delete `packages/cli/src/sessions/droid.ts`
- [x] Delete `packages/cli/src/sessions/gemini.ts`
- [x] Delete `packages/cli/src/sessions/opencode.ts`
- [x] Delete `packages/cli/src/sessions/reports.ts`
- [x] Delete `packages/cli/src/sessions/index.ts`
- [x] Keep `packages/cli/src/sessions/types.ts`
- [ ] Update `native.ts` - remove fallback logic
- [x] Update `cli.ts` - use Rust for pricing command
- [x] Update imports in `submit.ts`, `wrapped.ts`, `graph.ts`
- [x] Run `tsc --noEmit`
- [ ] Test CLI commands

### Integration Testing
- [ ] `tokscale pricing "claude-3-5-sonnet-20241022"` works
- [ ] `tokscale pricing "big-pickle"` works (alias resolution)
- [ ] `tokscale pricing "glm-4.7-free"` works (OpenRouter)
- [ ] `tokscale models` works
- [ ] `tokscale monthly` works
- [ ] `tokscale graph --output test.json` works
- [ ] `tokscale tui` works
- [ ] `tokscale wrapped` works

---

## Notes

### Breaking Changes
- Native module is now REQUIRED (no TS fallback)
- Users on unsupported platforms will not work
- Development requires `bun run build:core` before testing

### Migration for Existing Caches
- Old cache files (`~/.cache/tokscale/pricing.json`) will be ignored
- New cache files: `pricing-litellm.json`, `pricing-openrouter.json`
- First run after upgrade will re-fetch pricing

### Performance Expectations
- First run: ~500ms (fetch LiteLLM + OpenRouter)
- Cached run: ~50ms (disk cache hit)
- Session parsing: unchanged (~100-200ms for typical usage)
