//! Distribution factory - creates distribution instances from configuration

use waremax_config::OrderConfig;

use crate::distributions::{
    ArrivalDistribution, ConstantArrivals, ConstantLines, DistributionSet, ExponentialArrivals,
    LinesDistribution, NegBinomialLines, PoissonLines, SkuDistribution, UniformSkus, ZipfSkus,
};

/// Create a DistributionSet from order configuration
pub fn create_distributions(config: &OrderConfig) -> DistributionSet {
    DistributionSet::new(
        create_arrival_distribution(config),
        create_lines_distribution(config),
        create_sku_distribution(config),
    )
}

fn create_arrival_distribution(config: &OrderConfig) -> Box<dyn ArrivalDistribution> {
    let rate_per_sec = config.arrival_process.rate_per_min / 60.0;

    match config.arrival_process.process_type.as_str() {
        "exponential" | "poisson" => Box::new(ExponentialArrivals::new(rate_per_sec)),
        "constant" => {
            let interval = if rate_per_sec > 0.0 {
                1.0 / rate_per_sec
            } else {
                60.0 // Default 1 per minute
            };
            Box::new(ConstantArrivals::new(interval))
        }
        unknown => {
            eprintln!(
                "Warning: Unknown arrival process type '{}', using exponential",
                unknown
            );
            Box::new(ExponentialArrivals::new(rate_per_sec))
        }
    }
}

fn create_lines_distribution(config: &OrderConfig) -> Box<dyn LinesDistribution> {
    match config.lines_per_order.dist_type.as_str() {
        "negbin" => Box::new(NegBinomialLines::new(
            config.lines_per_order.mean,
            config.lines_per_order.dispersion,
        )),
        "poisson" => Box::new(PoissonLines::new(config.lines_per_order.mean)),
        "constant" => Box::new(ConstantLines::new(
            config.lines_per_order.mean.max(1.0) as u32
        )),
        unknown => {
            eprintln!(
                "Warning: Unknown lines distribution type '{}', using negbin",
                unknown
            );
            Box::new(NegBinomialLines::new(
                config.lines_per_order.mean,
                config.lines_per_order.dispersion,
            ))
        }
    }
}

fn create_sku_distribution(config: &OrderConfig) -> Box<dyn SkuDistribution> {
    match config.sku_popularity.dist_type.as_str() {
        "zipf" => Box::new(ZipfSkus::new(config.sku_popularity.alpha)),
        "uniform" => Box::new(UniformSkus::new()),
        unknown => {
            eprintln!(
                "Warning: Unknown SKU distribution type '{}', using zipf",
                unknown
            );
            Box::new(ZipfSkus::new(config.sku_popularity.alpha))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waremax_config::{ArrivalProcess, LinesConfig, OrderConfig, SkuPopularity};

    fn test_order_config() -> OrderConfig {
        OrderConfig {
            arrival_process: ArrivalProcess {
                process_type: "exponential".to_string(),
                rate_per_min: 4.0,
            },
            lines_per_order: LinesConfig {
                dist_type: "negbin".to_string(),
                mean: 2.0,
                dispersion: 1.0,
            },
            sku_popularity: SkuPopularity {
                dist_type: "zipf".to_string(),
                alpha: 1.0,
            },
            due_times: None,
        }
    }

    #[test]
    fn test_create_distributions_default() {
        let config = test_order_config();
        let dists = create_distributions(&config);
        assert_eq!(dists.arrivals.name(), "exponential");
        assert_eq!(dists.lines.name(), "negbin");
        assert_eq!(dists.skus.name(), "zipf");
    }

    #[test]
    fn test_create_constant_arrivals() {
        let mut config = test_order_config();
        config.arrival_process.process_type = "constant".to_string();
        let dists = create_distributions(&config);
        assert_eq!(dists.arrivals.name(), "constant");
    }

    #[test]
    fn test_create_poisson_lines() {
        let mut config = test_order_config();
        config.lines_per_order.dist_type = "poisson".to_string();
        let dists = create_distributions(&config);
        assert_eq!(dists.lines.name(), "poisson");
    }

    #[test]
    fn test_create_uniform_skus() {
        let mut config = test_order_config();
        config.sku_popularity.dist_type = "uniform".to_string();
        let dists = create_distributions(&config);
        assert_eq!(dists.skus.name(), "uniform");
    }

    #[test]
    fn test_unknown_arrival_type_falls_back() {
        let mut config = test_order_config();
        config.arrival_process.process_type = "unknown_type".to_string();
        let dists = create_distributions(&config);
        // Should fall back to exponential
        assert_eq!(dists.arrivals.name(), "exponential");
    }

    #[test]
    fn test_unknown_lines_type_falls_back() {
        let mut config = test_order_config();
        config.lines_per_order.dist_type = "unknown_type".to_string();
        let dists = create_distributions(&config);
        // Should fall back to negbin
        assert_eq!(dists.lines.name(), "negbin");
    }

    #[test]
    fn test_unknown_sku_type_falls_back() {
        let mut config = test_order_config();
        config.sku_popularity.dist_type = "unknown_type".to_string();
        let dists = create_distributions(&config);
        // Should fall back to zipf
        assert_eq!(dists.skus.name(), "zipf");
    }
}
