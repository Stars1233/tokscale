use super::{cache, aliases};
use super::litellm::ModelPricing;
use std::collections::HashMap;
use serde::Deserialize;

const CACHE_FILENAME: &str = "pricing-openrouter.json";
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 200;

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

async fn fetch_model_endpoints(
    client: &reqwest::Client,
    author: &str,
    slug: &str,
) -> Option<ModelPricing> {
    let url = format!("https://openrouter.ai/api/v1/models/{}/{}/endpoints", author, slug);
    
    let mut last_error: Option<String> = None;
    
    for attempt in 0..MAX_RETRIES {
        let response = match client.get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(format!("network error: {}", e));
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            INITIAL_BACKOFF_MS * (1 << attempt)
                        )).await;
                    }
                    continue;
                }
            };
        
        let status = response.status();
        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            last_error = Some(format!("HTTP {}", status));
            let _ = response.bytes().await;
            if attempt < MAX_RETRIES - 1 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    INITIAL_BACKOFF_MS * (1 << attempt)
                )).await;
            }
            continue;
        }
        
        if !status.is_success() {
            eprintln!("[tokscale] OpenRouter {} for {}/{}", status, author, slug);
            return None;
        }
        
        let data: EndpointsResponse = match response.json().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[tokscale] OpenRouter JSON parse failed for {}/{}: {}", author, slug, e);
                return None;
            }
        };
        
        let expected_provider = aliases::OPENROUTER_PROVIDER_NAMES
            .get(author)
            .copied()
            .unwrap_or(author);
        
        let endpoint = match data.data.endpoints.iter()
            .find(|e| e.provider_name.eq_ignore_ascii_case(expected_provider)) {
                Some(e) => e,
                None => {
                    eprintln!("[tokscale] OpenRouter provider '{}' not found for {}/{}", expected_provider, author, slug);
                    return None;
                }
            };
        
        let input_cost: f64 = match endpoint.pricing.prompt.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("[tokscale] Invalid input price '{}' for {}/{}", endpoint.pricing.prompt, author, slug);
                return None;
            }
        };
        
        let output_cost: f64 = match endpoint.pricing.completion.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("[tokscale] Invalid output price '{}' for {}/{}", endpoint.pricing.completion, author, slug);
                return None;
            }
        };
        
        if !input_cost.is_finite() || !output_cost.is_finite() || input_cost < 0.0 || output_cost < 0.0 {
            eprintln!("[tokscale] Invalid pricing values for {}/{}: input={}, output={}", author, slug, input_cost, output_cost);
            return None;
        }
        
        return Some(ModelPricing {
            input_cost_per_token: Some(input_cost),
            output_cost_per_token: Some(output_cost),
            cache_read_input_token_cost: endpoint.pricing.input_cache_read
                .as_ref()
                .and_then(|s| s.trim().parse().ok())
                .filter(|v: &f64| v.is_finite() && *v >= 0.0),
            cache_creation_input_token_cost: endpoint.pricing.input_cache_write
                .as_ref()
                .and_then(|s| s.trim().parse().ok())
                .filter(|v: &f64| v.is_finite() && *v >= 0.0),
        });
    }
    
    if let Some(err) = last_error {
        eprintln!("[tokscale] OpenRouter fetch failed for {}/{} after {} retries: {}", author, slug, MAX_RETRIES, err);
    }
    None
}

pub async fn fetch_all_mapped() -> HashMap<String, ModelPricing> {
    if let Some(cached) = load_cached() {
        return cached;
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    
    let mut result = HashMap::new();
    
    let unique_ids: std::collections::HashSet<&str> = 
        aliases::OPENROUTER_MAPPINGS.values().copied().collect();
    
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
    
    if !result.is_empty() {
        let _ = cache::save_cache(CACHE_FILENAME, &result);
    }
    
    result
}
