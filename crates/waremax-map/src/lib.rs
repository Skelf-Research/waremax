//! Waremax Map - Graph-based map, routing, and traffic management

pub mod graph;
pub mod routing;
pub mod traffic;
pub mod deadlock;
pub mod reservation;

pub use graph::{Node, Edge, NodeType, EdgeDirection, WarehouseMap};
pub use routing::{Route, RouteCache, Router};
pub use traffic::TrafficManager;

// v2: Deadlock detection
pub use deadlock::{WaitForGraph, WaitingFor};

// v2: Reservation-based traffic control
pub use reservation::{ReservableResource, Reservation, ReservationConflict, ReservationManager};
