//! Waremax Entities - Robots, stations, orders, and tasks

pub mod charging_station;
pub mod maintenance_station;
pub mod order;
pub mod robot;
pub mod station;
pub mod task;

pub use charging_station::ChargingStation;
pub use maintenance_station::MaintenanceStation;
pub use order::{Order, OrderLine, OrderStatus};
pub use robot::{
    BatteryConsumptionModel, BatteryState, FailureModel, MaintenanceState, Robot, RobotState,
};
pub use station::{ServiceTimeModel, Station, StationType};
pub use task::{BinLocation, Task, TaskStatus, TaskType};
