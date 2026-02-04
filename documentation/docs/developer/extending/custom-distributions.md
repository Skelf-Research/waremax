# Custom Distributions

Add statistical distributions for simulation variability.

---

## Overview

Distributions control randomness in:

- Service times
- Order arrival
- Failure rates
- Battery consumption

---

## Distribution Trait

```rust
pub trait Distribution: Send + Sync {
    /// Sample a random value
    fn sample(&self, rng: &mut impl Rng) -> f64;

    /// Expected value (for planning)
    fn mean(&self) -> f64;

    /// Variance (for analysis)
    fn variance(&self) -> f64;

    /// Name for logging
    fn name(&self) -> &str;
}
```

---

## Built-in Distributions

### Constant

Fixed value (no randomness):

```rust
pub struct Constant {
    value: f64,
}

impl Distribution for Constant {
    fn sample(&self, _rng: &mut impl Rng) -> f64 {
        self.value
    }

    fn mean(&self) -> f64 {
        self.value
    }

    fn variance(&self) -> f64 {
        0.0
    }
}
```

### Uniform

Even probability across range:

```rust
pub struct Uniform {
    min: f64,
    max: f64,
}

impl Distribution for Uniform {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        rng.gen_range(self.min..=self.max)
    }

    fn mean(&self) -> f64 {
        (self.min + self.max) / 2.0
    }

    fn variance(&self) -> f64 {
        (self.max - self.min).powi(2) / 12.0
    }
}
```

### Exponential

Memoryless (for queuing theory):

```rust
pub struct Exponential {
    rate: f64,  // λ
}

impl Distribution for Exponential {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen();
        -u.ln() / self.rate
    }

    fn mean(&self) -> f64 {
        1.0 / self.rate
    }

    fn variance(&self) -> f64 {
        1.0 / (self.rate * self.rate)
    }
}
```

---

## Example: Triangular Distribution

### Step 1: Implement the Distribution

```rust
// crates/waremax-core/src/distributions/triangular.rs

use rand::Rng;
use crate::Distribution;

/// Triangular distribution with min, mode, max
pub struct Triangular {
    min: f64,
    mode: f64,
    max: f64,
}

impl Triangular {
    pub fn new(min: f64, mode: f64, max: f64) -> Self {
        assert!(min <= mode && mode <= max);
        Self { min, mode, max }
    }
}

impl Distribution for Triangular {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen();
        let fc = (self.mode - self.min) / (self.max - self.min);

        if u < fc {
            self.min + ((self.max - self.min) * (self.mode - self.min) * u).sqrt()
        } else {
            self.max - ((self.max - self.min) * (self.max - self.mode) * (1.0 - u)).sqrt()
        }
    }

    fn mean(&self) -> f64 {
        (self.min + self.mode + self.max) / 3.0
    }

    fn variance(&self) -> f64 {
        let a = self.min;
        let b = self.mode;
        let c = self.max;
        (a*a + b*b + c*c - a*b - a*c - b*c) / 18.0
    }

    fn name(&self) -> &str {
        "triangular"
    }
}
```

### Step 2: Register the Distribution

```rust
// crates/waremax-config/src/distributions.rs

#[derive(Deserialize)]
#[serde(tag = "distribution")]
pub enum DistributionConfig {
    #[serde(rename = "constant")]
    Constant { value: f64 },

    #[serde(rename = "uniform")]
    Uniform { min: f64, max: f64 },

    #[serde(rename = "triangular")]
    Triangular { min: f64, mode: f64, max: f64 },

    // ...
}

impl DistributionConfig {
    pub fn build(&self) -> Box<dyn Distribution> {
        match self {
            Self::Constant { value } =>
                Box::new(Constant::new(*value)),
            Self::Uniform { min, max } =>
                Box::new(Uniform::new(*min, *max)),
            Self::Triangular { min, mode, max } =>
                Box::new(Triangular::new(*min, *mode, *max)),
            // ...
        }
    }
}
```

### Step 3: Use in Configuration

```yaml
# scenario.yaml
stations:
  - id: "S1"
    service_time_s:
      distribution: triangular
      min: 3.0
      mode: 5.0
      max: 10.0
```

---

## Example: Bimodal Distribution

For scenarios with two distinct modes:

```rust
pub struct Bimodal {
    dist1: Normal,
    dist2: Normal,
    weight1: f64,  // Probability of sampling from dist1
}

impl Bimodal {
    pub fn new(
        mean1: f64, std1: f64,
        mean2: f64, std2: f64,
        weight1: f64,
    ) -> Self {
        Self {
            dist1: Normal::new(mean1, std1),
            dist2: Normal::new(mean2, std2),
            weight1,
        }
    }
}

impl Distribution for Bimodal {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        if rng.gen::<f64>() < self.weight1 {
            self.dist1.sample(rng)
        } else {
            self.dist2.sample(rng)
        }
    }

    fn mean(&self) -> f64 {
        self.weight1 * self.dist1.mean()
            + (1.0 - self.weight1) * self.dist2.mean()
    }

    fn variance(&self) -> f64 {
        // Mixture variance formula
        let m1 = self.dist1.mean();
        let m2 = self.dist2.mean();
        let m = self.mean();
        let w1 = self.weight1;
        let w2 = 1.0 - w1;

        w1 * (self.dist1.variance() + (m1 - m).powi(2))
            + w2 * (self.dist2.variance() + (m2 - m).powi(2))
    }
}
```

---

## Testing Distributions

### Statistical Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_triangular_mean() {
        let dist = Triangular::new(0.0, 5.0, 10.0);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let samples: Vec<f64> = (0..10000)
            .map(|_| dist.sample(&mut rng))
            .collect();

        let empirical_mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let theoretical_mean = dist.mean();

        assert!((empirical_mean - theoretical_mean).abs() < 0.1);
    }

    #[test]
    fn test_triangular_range() {
        let dist = Triangular::new(2.0, 5.0, 8.0);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for _ in 0..1000 {
            let sample = dist.sample(&mut rng);
            assert!(sample >= 2.0 && sample <= 8.0);
        }
    }
}
```

### Visual Verification

```rust
// Generate histogram for manual verification
fn print_histogram(dist: &dyn Distribution, bins: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let samples: Vec<f64> = (0..10000)
        .map(|_| dist.sample(&mut rng))
        .collect();

    let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = (max - min) / bins as f64;

    // ... histogram logic
}
```

---

## Best Practices

### Determinism

Always use the provided RNG:

```rust
// Good
fn sample(&self, rng: &mut impl Rng) -> f64 {
    rng.gen_range(0.0..1.0)
}

// Bad - not deterministic
fn sample(&self, _rng: &mut impl Rng) -> f64 {
    rand::random()  // Uses thread-local RNG
}
```

### Validation

Validate parameters:

```rust
impl Triangular {
    pub fn new(min: f64, mode: f64, max: f64) -> Self {
        assert!(min <= mode, "min must be <= mode");
        assert!(mode <= max, "mode must be <= max");
        assert!(min < max, "min must be < max");
        // ...
    }
}
```

### Documentation

Document the distribution:

```rust
/// Triangular distribution
///
/// PDF is triangular with peak at `mode`.
/// Useful when you know the likely value but not the exact distribution.
///
/// # Parameters
/// - `min`: Minimum possible value
/// - `mode`: Most likely value
/// - `max`: Maximum possible value
```

---

## Related

- [Service Time](../../concepts/warehouse/stations.md#service-time)
- [Order Generation](../../configuration/orders.md)
- [Determinism](../../concepts/simulation/determinism.md)
