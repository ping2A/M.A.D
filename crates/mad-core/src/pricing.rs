use serde::{Deserialize, Serialize};

/// How vendor prices are quoted (per billing period).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPeriod {
    Monthly,
    Annual,
}

impl Default for BillingPeriod {
    fn default() -> Self {
        Self::Monthly
    }
}

/// Per-vendor list and platform fees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorPricing {
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub billing_period: BillingPeriod,
    /// License cost per managed device per billing period.
    pub price_per_device: Option<f64>,
    /// Flat platform / support fee per billing period (added to per-device total).
    pub global_price: Option<f64>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_currency() -> String {
    "USD".into()
}

impl Default for VendorPricing {
    fn default() -> Self {
        Self {
            currency: default_currency(),
            billing_period: BillingPeriod::default(),
            price_per_device: None,
            global_price: None,
            notes: None,
        }
    }
}

/// Fleet and weighting settings for capability vs. cost decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcurementConfig {
    /// Managed devices used to estimate total contract cost.
    #[serde(default = "default_device_count")]
    pub device_count: u32,
    /// Share of the composite score driven by price (0–100). Capability fills the remainder.
    #[serde(default)]
    pub price_weight_percent: f64,
    #[serde(default)]
    pub use_price_in_ranking: bool,
}

fn default_device_count() -> u32 {
    500
}

impl Default for ProcurementConfig {
    fn default() -> Self {
        Self {
            device_count: default_device_count(),
            price_weight_percent: 25.0,
            use_price_in_ranking: false,
        }
    }
}

/// Annualize a billing-period amount.
pub fn annualize(amount: f64, period: BillingPeriod) -> f64 {
    match period {
        BillingPeriod::Monthly => amount * 12.0,
        BillingPeriod::Annual => amount,
    }
}

/// Returns `(annual_cost_per_device, total_annual_cost)` when pricing data allows.
pub fn compute_annual_costs(
    pricing: &VendorPricing,
    device_count: u32,
) -> (Option<f64>, Option<f64>) {
    let devices = device_count as f64;
    let per_device = pricing.price_per_device.map(|p| annualize(p, pricing.billing_period));
    let global_annual = pricing
        .global_price
        .map(|g| annualize(g, pricing.billing_period));

    let total = match (per_device, global_annual) {
        (Some(p), Some(g)) if devices > 0.0 => Some(g + p * devices),
        (Some(p), None) if devices > 0.0 => Some(p * devices),
        (None, Some(g)) => Some(g),
        (Some(p), _) => Some(p * devices.max(1.0)),
        (None, None) => None,
    };

    let per_device_effective = match (per_device, global_annual, devices) {
        (_, _, d) if d <= 0.0 => per_device,
        (Some(p), Some(g), d) => Some(p + g / d),
        (Some(p), None, _) => Some(p),
        (None, Some(g), d) => Some(g / d),
        (None, None, _) => None,
    };

    (per_device_effective, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_per_device_and_global_total() {
        let pricing = VendorPricing {
            currency: "USD".into(),
            billing_period: BillingPeriod::Monthly,
            price_per_device: Some(8.0),
            global_price: Some(5_000.0),
            notes: None,
        };
        let (per, total) = compute_annual_costs(&pricing, 100);
        assert!((per.unwrap() - (96.0 + 60_000.0 / 100.0)).abs() < 0.01);
        assert!((total.unwrap() - (96.0 * 100.0 + 60_000.0)).abs() < 0.01);
    }
}
