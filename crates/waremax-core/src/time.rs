//! Simulation time type with sub-second precision

use rkyv::{Archive, Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub, AddAssign, SubAssign};

/// Simulation time in seconds (f64 for sub-second precision)
#[derive(Archive, Deserialize, Serialize, Clone, Copy, PartialEq, Default)]
#[rkyv(compare(PartialEq))]
pub struct SimTime(pub f64);

impl SimTime {
    /// Zero time constant
    pub const ZERO: SimTime = SimTime(0.0);

    /// Maximum time constant (for sentinel values)
    pub const MAX: SimTime = SimTime(f64::MAX);

    /// Create from seconds
    #[inline]
    pub fn from_seconds(s: f64) -> Self {
        SimTime(s)
    }

    /// Create from minutes
    #[inline]
    pub fn from_minutes(m: f64) -> Self {
        SimTime(m * 60.0)
    }

    /// Create from hours
    #[inline]
    pub fn from_hours(h: f64) -> Self {
        SimTime(h * 3600.0)
    }

    /// Get time as seconds
    #[inline]
    pub fn as_seconds(&self) -> f64 {
        self.0
    }

    /// Get time as minutes
    #[inline]
    pub fn as_minutes(&self) -> f64 {
        self.0 / 60.0
    }

    /// Get time as hours
    #[inline]
    pub fn as_hours(&self) -> f64 {
        self.0 / 3600.0
    }

    /// Check if time is zero
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Add for SimTime {
    type Output = SimTime;

    #[inline]
    fn add(self, rhs: SimTime) -> SimTime {
        SimTime(self.0 + rhs.0)
    }
}

impl Sub for SimTime {
    type Output = SimTime;

    #[inline]
    fn sub(self, rhs: SimTime) -> SimTime {
        SimTime(self.0 - rhs.0)
    }
}

impl AddAssign for SimTime {
    #[inline]
    fn add_assign(&mut self, rhs: SimTime) {
        self.0 += rhs.0;
    }
}

impl SubAssign for SimTime {
    #[inline]
    fn sub_assign(&mut self, rhs: SimTime) {
        self.0 -= rhs.0;
    }
}

impl PartialOrd for SimTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl fmt::Debug for SimTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimTime({:.3}s)", self.0)
    }
}

impl fmt::Display for SimTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 3600.0 {
            write!(f, "{:.2}h", self.as_hours())
        } else if self.0 >= 60.0 {
            write!(f, "{:.2}m", self.as_minutes())
        } else {
            write!(f, "{:.2}s", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_conversions() {
        let t = SimTime::from_minutes(1.5);
        assert_eq!(t.as_seconds(), 90.0);
        assert_eq!(t.as_minutes(), 1.5);
    }

    #[test]
    fn test_time_arithmetic() {
        let t1 = SimTime::from_seconds(10.0);
        let t2 = SimTime::from_seconds(5.0);
        assert_eq!((t1 + t2).as_seconds(), 15.0);
        assert_eq!((t1 - t2).as_seconds(), 5.0);
    }
}
