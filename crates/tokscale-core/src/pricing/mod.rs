pub mod aliases;
pub mod cache;
pub mod litellm;
pub mod lookup;
pub mod openrouter;

use lookup::{PricingLookup, LookupResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub use litellm::ModelPricing;

static PRICING_SERVICE: OnceCell<Arc<PricingService>> = OnceCell::const_new();

pub struct PricingService {
    lookup: PricingLookup,
}

impl PricingService {
    pub fn new(litellm_data: HashMap<String, ModelPricing>, openrouter_data: HashMap<String, ModelPricing>) -> Self {
        Self {
            lookup: PricingLookup::new(litellm_data, openrouter_data),
        }
    }
    
    async fn fetch_inner() -> Result<Self, String> {
        let (litellm_result, openrouter_data) = tokio::join!(
            litellm::fetch(),
            openrouter::fetch_all_mapped()
        );
        
        let litellm_data = litellm_result.map_err(|e| e.to_string())?;
        
        Ok(Self::new(litellm_data, openrouter_data))
    }
    
    pub async fn get_or_init() -> Result<Arc<PricingService>, String> {
        PRICING_SERVICE.get_or_try_init(|| async {
            Self::fetch_inner().await.map(Arc::new)
        }).await.map(Arc::clone)
    }

    pub fn lookup_with_source(&self, model_id: &str, force_source: Option<&str>) -> Option<LookupResult> {
        self.lookup.lookup_with_source(model_id, force_source)
    }
    
    pub fn calculate_cost(&self, model_id: &str, input: i64, output: i64, cache_read: i64, cache_write: i64, reasoning: i64) -> f64 {
        self.lookup.calculate_cost(model_id, input, output, cache_read, cache_write, reasoning)
    }
}
