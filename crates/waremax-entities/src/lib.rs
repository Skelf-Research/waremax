//! Waremax Entities - Robots, stations, orders, and tasks

pub mod robot;
pub mod station;
pub mod order;
pub mod task;
pub mod charging_station;
pub mod maintenance_station;

pub use robot::{Robot, RobotState, BatteryState, BatteryConsumptionModel, MaintenanceState, FailureModel};
pub use station::{Station, StationType, ServiceTimeModel};
pub use order::{Order, OrderLine, OrderStatus};
pub use task::{Task, TaskType, TaskStatus, BinLocation};
pub use charging_station::ChargingStation;
pub use maintenance_station::MaintenanceStation;
