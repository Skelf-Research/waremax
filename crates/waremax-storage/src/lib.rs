//! Waremax Storage - Racks, bins, and inventory management

pub mod inventory;
pub mod rack;
pub mod sku;

pub use inventory::Inventory;
pub use rack::{BinAddress, Rack};
pub use sku::{Sku, SkuCatalog};
