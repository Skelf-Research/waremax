//! Seeded random number generator for deterministic simulation

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Exp, Gamma, Poisson};

/// Seeded RNG wrapper for deterministic simulation
#[derive(Debug)]
pub struct SimRng {
    rng: ChaCha8Rng,
}

impl SimRng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Generate a random value in the given range
    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.rng.gen_range(range)
    }

    /// Generate a random f64 in [0, 1)
    pub fn gen_f64(&mut self) -> f64 {
        self.rng.gen()
    }

    /// Generate a random bool with the given probability of true
    pub fn gen_bool(&mut self, p: f64) -> bool {
        self.rng.gen_bool(p)
    }

    /// Generate an exponential random variable
    ///
    /// Used for inter-arrival times in Poisson processes.
    /// Mean = 1/rate
    pub fn exponential(&mut self, rate: f64) -> f64 {
        if rate <= 0.0 {
            return f64::INFINITY;
        }
        let exp = Exp::new(rate).unwrap();
        exp.sample(&mut self.rng)
    }

    /// Generate a Poisson random variable
    ///
    /// Returns the number of events in a unit interval given the rate.
    pub fn poisson(&mut self, lambda: f64) -> u32 {
        if lambda <= 0.0 {
            return 0;
        }
        let pois = Poisson::new(lambda).unwrap();
        pois.sample(&mut self.rng) as u32
    }

    /// Generate from a negative binomial distribution
    ///
    /// Used for order line counts. Returns at least 1.
    /// Uses Gamma-Poisson mixture.
    pub fn negbin(&mut self, mean: f64, dispersion: f64) -> u32 {
        if mean <= 0.0 || dispersion <= 0.0 {
            return 1;
        }

        // Negative binomial via Gamma-Poisson mixture
        // r = dispersion, p = r / (r + mean)
        let r = dispersion;
        let p = r / (r + mean);
        let gamma_shape = r;
        let gamma_scale = (1.0 - p) / p;

        let gamma = Gamma::new(gamma_shape, gamma_scale).unwrap();
        let lambda = gamma.sample(&mut self.rng);

        if lambda <= 0.0 {
            return 1;
        }

        let pois = Poisson::new(lambda).unwrap();
        let value = pois.sample(&mut self.rng) as u32;

        // Return at least 1
        value.max(1)
    }

    /// Generate from a Zipf distribution
    ///
    /// Used for SKU popularity. Returns an index in [0, n).
    pub fn zipf(&mut self, n: usize, alpha: f64) -> usize {
        if n == 0 {
            return 0;
        }
        if n == 1 {
            return 0;
        }

        // Calculate normalization constant (Hurwitz zeta approximation)
        let mut h_sum = 0.0;
        for k in 1..=n {
            h_sum += 1.0 / (k as f64).powf(alpha);
        }

        // Generate uniform and find corresponding rank
        let u: f64 = self.rng.gen();
        let target = u * h_sum;

        let mut cumsum = 0.0;
        for k in 1..=n {
            cumsum += 1.0 / (k as f64).powf(alpha);
            if cumsum >= target {
                return k - 1; // Return 0-indexed
            }
        }

        n - 1
    }

    /// Generate a normal random variable
    pub fn normal(&mut self, mean: f64, stddev: f64) -> f64 {
        use rand_distr::Normal;
        let normal = Normal::new(mean, stddev).unwrap();
        normal.sample(&mut self.rng)
    }

    /// Generate a lognormal random variable
    pub fn lognormal(&mut self, mean: f64, stddev: f64) -> f64 {
        use rand_distr::LogNormal;
        // Convert mean/stddev to mu/sigma for lognormal
        let variance = stddev * stddev;
        let mu = (mean * mean / (mean * mean + variance).sqrt()).ln();
        let sigma = (1.0 + variance / (mean * mean)).ln().sqrt();
        let lognormal = LogNormal::new(mu, sigma).unwrap();
        lognormal.sample(&mut self.rng)
    }

    /// Generate a uniform random variable
    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        self.rng.gen_range(min..max)
    }

    /// Choose a random element from a slice
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let idx = self.gen_range(0..slice.len());
            Some(&slice[idx])
        }
    }

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism() {
        let mut rng1 = SimRng::new(42);
        let mut rng2 = SimRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.gen_f64(), rng2.gen_f64());
        }
    }

    #[test]
    fn test_exponential() {
        let mut rng = SimRng::new(42);

        // Generate many samples and check mean
        let rate = 2.0;
        let expected_mean = 1.0 / rate;
        let samples: Vec<f64> = (0..10000).map(|_| rng.exponential(rate)).collect();
        let actual_mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;

        // Should be close to expected mean
        assert!((actual_mean - expected_mean).abs() < 0.1);
    }

    #[test]
    fn test_negbin() {
        let mut rng = SimRng::new(42);

        // Generate samples and verify all >= 1
        for _ in 0..100 {
            let value = rng.negbin(2.2, 1.3);
            assert!(value >= 1);
        }
    }

    #[test]
    fn test_zipf() {
        let mut rng = SimRng::new(42);
        let n = 100;

        // Lower indices should be more common
        let mut counts = vec![0u32; n];
        for _ in 0..10000 {
            let idx = rng.zipf(n, 1.0);
            assert!(idx < n);
            counts[idx] += 1;
        }

        // First element should be most common
        assert!(counts[0] > counts[n - 1]);
    }
}
