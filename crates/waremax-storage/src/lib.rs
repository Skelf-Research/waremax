//! Waremax Storage - Racks, bins, and inventory management

pub mod rack;
pub mod inventory;
pub mod sku;

pub use rack::{Rack, BinAddress};
pub use inventory::Inventory;
pub use sku::{Sku, SkuCatalog};
