//! Distribution generators for simulation randomness
//!
//! Provides configurable distributions for:
//! - Order inter-arrival times
//! - Lines per order
//! - SKU popularity/selection

use waremax_core::SimRng;

// ============================================================================
// Traits
// ============================================================================

/// Generates inter-arrival times for orders
pub trait ArrivalDistribution: Send + Sync {
    /// Distribution name for logging
    fn name(&self) -> &str;
    /// Generate next inter-arrival time in seconds
    fn next_interarrival(&self, rng: &mut SimRng) -> f64;
}

/// Generates number of lines per order
pub trait LinesDistribution: Send + Sync {
    /// Distribution name for logging
    fn name(&self) -> &str;
    /// Generate next line count (always >= 1)
    fn next_lines(&self, rng: &mut SimRng) -> u32;
}

/// Generates SKU indices based on popularity
pub trait SkuDistribution: Send + Sync {
    /// Distribution name for logging
    fn name(&self) -> &str;
    /// Generate next SKU index (0 to num_skus-1)
    fn next_sku(&self, rng: &mut SimRng, num_skus: u32) -> u32;
}

// ============================================================================
// Arrival Distributions
// ============================================================================

/// Exponential inter-arrival times (Poisson process)
pub struct ExponentialArrivals {
    rate_per_sec: f64,
}

impl ExponentialArrivals {
    pub fn new(rate_per_sec: f64) -> Self {
        Self { rate_per_sec }
    }
}

impl ArrivalDistribution for ExponentialArrivals {
    fn name(&self) -> &str {
        "exponential"
    }

    fn next_interarrival(&self, rng: &mut SimRng) -> f64 {
        rng.exponential(self.rate_per_sec)
    }
}

/// Constant inter-arrival times (deterministic)
pub struct ConstantArrivals {
    interval_sec: f64,
}

impl ConstantArrivals {
    pub fn new(interval_sec: f64) -> Self {
        Self { interval_sec }
    }
}

impl ArrivalDistribution for ConstantArrivals {
    fn name(&self) -> &str {
        "constant"
    }

    fn next_interarrival(&self, _rng: &mut SimRng) -> f64 {
        self.interval_sec
    }
}

// ============================================================================
// Lines Distributions
// ============================================================================

/// Negative binomial distribution for lines per order
pub struct NegBinomialLines {
    mean: f64,
    dispersion: f64,
}

impl NegBinomialLines {
    pub fn new(mean: f64, dispersion: f64) -> Self {
        Self { mean, dispersion }
    }
}

impl LinesDistribution for NegBinomialLines {
    fn name(&self) -> &str {
        "negbin"
    }

    fn next_lines(&self, rng: &mut SimRng) -> u32 {
        rng.negbin(self.mean, self.dispersion).max(1) as u32
    }
}

/// Poisson distribution for lines per order
pub struct PoissonLines {
    mean: f64,
}

impl PoissonLines {
    pub fn new(mean: f64) -> Self {
        Self { mean }
    }
}

impl LinesDistribution for PoissonLines {
    fn name(&self) -> &str {
        "poisson"
    }

    fn next_lines(&self, rng: &mut SimRng) -> u32 {
        rng.poisson(self.mean).max(1) as u32
    }
}

/// Constant number of lines per order
pub struct ConstantLines {
    count: u32,
}

impl ConstantLines {
    pub fn new(count: u32) -> Self {
        Self {
            count: count.max(1),
        }
    }
}

impl LinesDistribution for ConstantLines {
    fn name(&self) -> &str {
        "constant"
    }

    fn next_lines(&self, _rng: &mut SimRng) -> u32 {
        self.count
    }
}

// ============================================================================
// SKU Distributions
// ============================================================================

/// Zipf distribution for SKU popularity (power-law)
pub struct ZipfSkus {
    alpha: f64,
}

impl ZipfSkus {
    pub fn new(alpha: f64) -> Self {
        Self { alpha }
    }
}

impl SkuDistribution for ZipfSkus {
    fn name(&self) -> &str {
        "zipf"
    }

    fn next_sku(&self, rng: &mut SimRng, num_skus: u32) -> u32 {
        let idx = rng.zipf(num_skus as usize, self.alpha) as u32;
        // Ensure in valid range (zipf returns 1-indexed)
        idx.saturating_sub(1).min(num_skus.saturating_sub(1))
    }
}

/// Uniform distribution for SKU selection (equal popularity)
pub struct UniformSkus;

impl UniformSkus {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UniformSkus {
    fn default() -> Self {
        Self::new()
    }
}

impl SkuDistribution for UniformSkus {
    fn name(&self) -> &str {
        "uniform"
    }

    fn next_sku(&self, rng: &mut SimRng, num_skus: u32) -> u32 {
        if num_skus == 0 {
            return 0;
        }
        rng.gen_range(0..num_skus)
    }
}

// ============================================================================
// Distribution Container
// ============================================================================

/// Container for all distribution generators
pub struct DistributionSet {
    pub arrivals: Box<dyn ArrivalDistribution>,
    pub lines: Box<dyn LinesDistribution>,
    pub skus: Box<dyn SkuDistribution>,
}

impl DistributionSet {
    /// Create a new distribution set with custom distributions
    pub fn new(
        arrivals: Box<dyn ArrivalDistribution>,
        lines: Box<dyn LinesDistribution>,
        skus: Box<dyn SkuDistribution>,
    ) -> Self {
        Self {
            arrivals,
            lines,
            skus,
        }
    }

    /// Get distribution names for logging
    pub fn names(&self) -> (&str, &str, &str) {
        (self.arrivals.name(), self.lines.name(), self.skus.name())
    }
}

impl Default for DistributionSet {
    fn default() -> Self {
        Self {
            arrivals: Box::new(ExponentialArrivals::new(4.0 / 60.0)), // 4 orders/min
            lines: Box::new(NegBinomialLines::new(2.0, 1.0)),
            skus: Box::new(ZipfSkus::new(1.0)),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> SimRng {
        SimRng::new(42)
    }

    #[test]
    fn test_exponential_arrivals_positive() {
        let dist = ExponentialArrivals::new(1.0);
        let mut rng = test_rng();
        for _ in 0..100 {
            let val = dist.next_interarrival(&mut rng);
            assert!(val > 0.0, "Exponential should produce positive values");
        }
    }

    #[test]
    fn test_constant_arrivals() {
        let dist = ConstantArrivals::new(5.0);
        let mut rng = test_rng();
        for _ in 0..10 {
            let val = dist.next_interarrival(&mut rng);
            assert!(
                (val - 5.0).abs() < f64::EPSILON,
                "Constant should return exact value"
            );
        }
    }

    #[test]
    fn test_negbin_lines_at_least_one() {
        let dist = NegBinomialLines::new(2.0, 1.0);
        let mut rng = test_rng();
        for _ in 0..100 {
            let val = dist.next_lines(&mut rng);
            assert!(val >= 1, "Lines should be at least 1");
        }
    }

    #[test]
    fn test_poisson_lines_at_least_one() {
        let dist = PoissonLines::new(2.0);
        let mut rng = test_rng();
        for _ in 0..100 {
            let val = dist.next_lines(&mut rng);
            assert!(val >= 1, "Lines should be at least 1");
        }
    }

    #[test]
    fn test_constant_lines() {
        let dist = ConstantLines::new(3);
        let mut rng = test_rng();
        for _ in 0..10 {
            let val = dist.next_lines(&mut rng);
            assert_eq!(val, 3, "Constant should return exact value");
        }
    }

    #[test]
    fn test_zipf_skus_in_range() {
        let dist = ZipfSkus::new(1.0);
        let mut rng = test_rng();
        let num_skus = 20;
        for _ in 0..100 {
            let val = dist.next_sku(&mut rng, num_skus);
            assert!(
                val < num_skus,
                "SKU index should be in range 0..{}",
                num_skus
            );
        }
    }

    #[test]
    fn test_uniform_skus_in_range() {
        let dist = UniformSkus::new();
        let mut rng = test_rng();
        let num_skus = 20;
        for _ in 0..100 {
            let val = dist.next_sku(&mut rng, num_skus);
            assert!(
                val < num_skus,
                "SKU index should be in range 0..{}",
                num_skus
            );
        }
    }

    #[test]
    fn test_uniform_skus_zero_skus() {
        let dist = UniformSkus::new();
        let mut rng = test_rng();
        let val = dist.next_sku(&mut rng, 0);
        assert_eq!(val, 0, "Should return 0 for empty SKU set");
    }

    #[test]
    fn test_distribution_set_default() {
        let set = DistributionSet::default();
        assert_eq!(set.arrivals.name(), "exponential");
        assert_eq!(set.lines.name(), "negbin");
        assert_eq!(set.skus.name(), "zipf");
    }

    #[test]
    fn test_distribution_set_names() {
        let set = DistributionSet::default();
        let (arrivals, lines, skus) = set.names();
        assert_eq!(arrivals, "exponential");
        assert_eq!(lines, "negbin");
        assert_eq!(skus, "zipf");
    }
}
