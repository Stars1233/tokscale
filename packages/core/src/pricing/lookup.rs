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
    pub source: String,
    pub matched_key: String,
}

impl PricingLookup {
    pub fn new(litellm: HashMap<String, ModelPricing>, openrouter: HashMap<String, ModelPricing>) -> Self {
        let mut sorted_keys: Vec<String> = litellm.keys().cloned().collect();
        sorted_keys.sort();
        
        Self { litellm, openrouter, sorted_keys }
    }
    
    pub fn lookup(&self, model_id: &str) -> Option<LookupResult> {
        let canonical = aliases::resolve_alias(model_id).unwrap_or(model_id);
        
        if let Some(result) = self.lookup_litellm(canonical) {
            return Some(result);
        }
        
        if let Some(result) = self.lookup_openrouter(canonical) {
            return Some(result);
        }
        
        None
    }
    
    fn lookup_litellm(&self, model_id: &str) -> Option<LookupResult> {
        if let Some(p) = self.litellm.get(model_id) {
            return Some(LookupResult {
                pricing: p.clone(),
                source: "litellm".into(),
                matched_key: model_id.into(),
            });
        }
        
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
    
    pub fn calculate_cost(&self, model_id: &str, input: i64, output: i64, cache_read: i64, cache_write: i64, reasoning: i64) -> f64 {
        let result = match self.lookup(model_id) {
            Some(r) => r,
            None => return 0.0,
        };
        
        let p = &result.pricing;
        let safe_price = |opt: Option<f64>| opt.filter(|v| v.is_finite() && *v >= 0.0).unwrap_or(0.0);
        
        let input_cost = input as f64 * safe_price(p.input_cost_per_token);
        let output_cost = (output + reasoning) as f64 * safe_price(p.output_cost_per_token);
        let cache_read_cost = cache_read as f64 * safe_price(p.cache_read_input_token_cost);
        let cache_write_cost = cache_write as f64 * safe_price(p.cache_creation_input_token_cost);
        
        input_cost + output_cost + cache_read_cost + cache_write_cost
    }
}

fn normalize_model_name(model_id: &str) -> Option<String> {
    let lower = model_id.to_lowercase();
    
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
    
    if lower == "o3" { return Some("o3".into()); }
    if lower.starts_with("gpt-4o") || lower == "gpt-4o" { return Some("gpt-4o".into()); }
    if lower.starts_with("gpt-4.1") || lower.contains("gpt-4.1") { return Some("gpt-4.1".into()); }
    
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
