//! Typed ID wrappers for type-safe entity references

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::fmt;
use std::hash::Hash;

/// Macro to define typed ID wrappers
macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Archive, Deserialize, Serialize,
            SerdeDeserialize, SerdeSerialize,
            Clone, Copy, PartialEq, Eq, Hash, Default
        )]
        #[rkyv(compare(PartialEq))]
        pub struct $name(pub u32);

        impl $name {
            /// Create a new ID from a u32 value
            #[inline]
            pub const fn new(id: u32) -> Self {
                Self(id)
            }

            /// Get the inner u32 value
            #[inline]
            pub const fn as_u32(&self) -> u32 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u32> for $name {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }

        impl From<$name> for u32 {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(RobotId, "Unique identifier for a robot");
define_id!(NodeId, "Unique identifier for a map node");
define_id!(EdgeId, "Unique identifier for a map edge");
define_id!(StationId, "Unique identifier for a station");
define_id!(RackId, "Unique identifier for a storage rack");
define_id!(BinId, "Unique identifier for a storage bin");
define_id!(SkuId, "Unique identifier for a SKU");
define_id!(OrderId, "Unique identifier for an order");
define_id!(TaskId, "Unique identifier for a task");
define_id!(EventId, "Unique identifier for a scheduled event");
define_id!(ShipmentId, "Unique identifier for an inbound/outbound shipment");
define_id!(ChargingStationId, "Unique identifier for a charging station");

/// ID generator for creating sequential IDs
#[derive(Debug, Clone, Default)]
pub struct IdGenerator<T> {
    next: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> IdGenerator<T> {
    /// Create a new ID generator starting from 0
    pub fn new() -> Self {
        Self {
            next: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a new ID generator starting from a specific value
    pub fn starting_from(start: u32) -> Self {
        Self {
            next: start,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the next ID value without incrementing
    pub fn peek(&self) -> u32 {
        self.next
    }
}

macro_rules! impl_id_generator {
    ($id_type:ident) => {
        impl IdGenerator<$id_type> {
            /// Generate the next ID
            pub fn next(&mut self) -> $id_type {
                let id = $id_type(self.next);
                self.next += 1;
                id
            }
        }
    };
}

impl_id_generator!(RobotId);
impl_id_generator!(NodeId);
impl_id_generator!(EdgeId);
impl_id_generator!(StationId);
impl_id_generator!(RackId);
impl_id_generator!(BinId);
impl_id_generator!(SkuId);
impl_id_generator!(OrderId);
impl_id_generator!(TaskId);
impl_id_generator!(EventId);
impl_id_generator!(ShipmentId);
impl_id_generator!(ChargingStationId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let mut gen = IdGenerator::<RobotId>::new();
        assert_eq!(gen.next(), RobotId(0));
        assert_eq!(gen.next(), RobotId(1));
        assert_eq!(gen.next(), RobotId(2));
    }

    #[test]
    fn test_id_equality() {
        let id1 = RobotId(42);
        let id2 = RobotId(42);
        let id3 = RobotId(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
