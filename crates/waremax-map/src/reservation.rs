//! Reservation-based traffic control for proactive conflict prevention
//!
//! This module provides time-windowed resource reservations that allow robots
//! to reserve edges and nodes ahead of time, preventing conflicts proactively
//! rather than reactively.

use std::collections::HashMap;
use waremax_core::{EdgeId, NodeId, RobotId, SimTime};

/// A resource that can be reserved (edge or node)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReservableResource {
    /// An edge between two nodes
    Edge(EdgeId),
    /// A node in the warehouse map
    Node(NodeId),
}

/// A time-windowed reservation of a resource
#[derive(Clone, Debug)]
pub struct Reservation {
    /// Robot that holds this reservation
    pub robot_id: RobotId,
    /// Resource being reserved
    pub resource: ReservableResource,
    /// Start time of the reservation window
    pub start_time: SimTime,
    /// End time of the reservation window
    pub end_time: SimTime,
}

impl Reservation {
    /// Create a new reservation
    pub fn new(
        robot_id: RobotId,
        resource: ReservableResource,
        start_time: SimTime,
        end_time: SimTime,
    ) -> Self {
        Self {
            robot_id,
            resource,
            start_time,
            end_time,
        }
    }

    /// Check if this reservation overlaps with a time window
    pub fn overlaps(&self, start: SimTime, end: SimTime) -> bool {
        self.start_time < end && self.end_time > start
    }
}

/// Error when a reservation conflicts with existing reservations
#[derive(Clone, Debug)]
pub struct ReservationConflict {
    /// The resource that has a conflict
    pub resource: ReservableResource,
    /// The robot that holds the conflicting reservation
    pub conflicting_robot: RobotId,
    /// Start of the conflicting time window
    pub conflict_start: SimTime,
    /// End of the conflicting time window
    pub conflict_end: SimTime,
}

/// Manages time-windowed reservations for edges and nodes
///
/// Allows robots to reserve resources for specific time windows,
/// enabling proactive conflict detection and prevention.
#[derive(Clone, Default)]
pub struct ReservationManager {
    /// Map from resource to list of reservations
    reservations: HashMap<ReservableResource, Vec<Reservation>>,
    /// Whether reservation system is enabled
    pub enabled: bool,
}

impl ReservationManager {
    /// Create a new reservation manager (disabled by default)
    pub fn new() -> Self {
        Self {
            reservations: HashMap::new(),
            enabled: false,
        }
    }

    /// Create a new enabled reservation manager
    pub fn new_enabled() -> Self {
        Self {
            reservations: HashMap::new(),
            enabled: true,
        }
    }

    /// Check if a resource can be reserved for a time window
    ///
    /// Returns true if no conflicting reservations exist, or if the
    /// reservation system is disabled.
    pub fn can_reserve(
        &self,
        resource: &ReservableResource,
        robot: RobotId,
        start: SimTime,
        end: SimTime,
    ) -> bool {
        if !self.enabled {
            return true;
        }

        if let Some(reservations) = self.reservations.get(resource) {
            !reservations
                .iter()
                .any(|r| r.robot_id != robot && r.overlaps(start, end))
        } else {
            true
        }
    }

    /// Reserve a resource for a time window
    ///
    /// Returns Ok(()) if the reservation was successful, or an error
    /// describing the conflict if another robot has a conflicting reservation.
    pub fn reserve(
        &mut self,
        resource: ReservableResource,
        robot: RobotId,
        start: SimTime,
        end: SimTime,
    ) -> Result<(), ReservationConflict> {
        if !self.enabled {
            return Ok(());
        }

        // Check for conflicts
        if let Some(reservations) = self.reservations.get(&resource) {
            for r in reservations {
                if r.robot_id != robot && r.overlaps(start, end) {
                    return Err(ReservationConflict {
                        resource: resource.clone(),
                        conflicting_robot: r.robot_id,
                        conflict_start: r.start_time,
                        conflict_end: r.end_time,
                    });
                }
            }
        }

        // Add reservation
        self.reservations
            .entry(resource.clone())
            .or_default()
            .push(Reservation::new(robot, resource, start, end));
        Ok(())
    }

    /// Release all reservations for a robot
    ///
    /// Called when a robot completes its task or aborts.
    pub fn release_all(&mut self, robot: RobotId) {
        for reservations in self.reservations.values_mut() {
            reservations.retain(|r| r.robot_id != robot);
        }
    }

    /// Release reservations for a robot on a specific resource
    pub fn release(&mut self, resource: &ReservableResource, robot: RobotId) {
        if let Some(reservations) = self.reservations.get_mut(resource) {
            reservations.retain(|r| r.robot_id != robot);
        }
    }

    /// Clean up expired reservations (those that ended before current_time)
    ///
    /// Should be called periodically to prevent memory growth.
    pub fn cleanup_expired(&mut self, current_time: SimTime) {
        for reservations in self.reservations.values_mut() {
            reservations.retain(|r| r.end_time > current_time);
        }
    }

    /// Get all conflicts for a proposed reservation
    ///
    /// Returns a list of reservations that would conflict with the proposed
    /// reservation window.
    pub fn get_conflicts(
        &self,
        resource: &ReservableResource,
        robot: RobotId,
        start: SimTime,
        end: SimTime,
    ) -> Vec<&Reservation> {
        if let Some(reservations) = self.reservations.get(resource) {
            reservations
                .iter()
                .filter(|r| r.robot_id != robot && r.overlaps(start, end))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all reservations for a specific robot
    pub fn get_robot_reservations(&self, robot: RobotId) -> Vec<&Reservation> {
        self.reservations
            .values()
            .flat_map(|v| v.iter())
            .filter(|r| r.robot_id == robot)
            .collect()
    }

    /// Get the number of active reservations
    pub fn reservation_count(&self) -> usize {
        self.reservations.values().map(|v| v.len()).sum()
    }

    /// Check if a resource has any reservations in a time window
    pub fn has_reservations(
        &self,
        resource: &ReservableResource,
        start: SimTime,
        end: SimTime,
    ) -> bool {
        if let Some(reservations) = self.reservations.get(resource) {
            reservations.iter().any(|r| r.overlaps(start, end))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(seconds: f64) -> SimTime {
        SimTime::from_seconds(seconds)
    }

    #[test]
    fn test_reservation_overlaps() {
        let res = Reservation::new(
            RobotId(1),
            ReservableResource::Edge(EdgeId(1)),
            t(10.0),
            t(20.0),
        );

        // Overlapping cases
        assert!(res.overlaps(t(15.0), t(25.0))); // Overlaps at end
        assert!(res.overlaps(t(5.0), t(15.0))); // Overlaps at start
        assert!(res.overlaps(t(12.0), t(18.0))); // Fully inside
        assert!(res.overlaps(t(5.0), t(25.0))); // Fully contains

        // Non-overlapping cases
        assert!(!res.overlaps(t(0.0), t(10.0))); // Ends exactly at start
        assert!(!res.overlaps(t(20.0), t(30.0))); // Starts exactly at end
        assert!(!res.overlaps(t(0.0), t(5.0))); // Before
        assert!(!res.overlaps(t(25.0), t(30.0))); // After
    }

    #[test]
    fn test_reservation_manager_disabled() {
        let mut mgr = ReservationManager::new();
        assert!(!mgr.enabled);

        // Can always reserve when disabled
        assert!(mgr.can_reserve(
            &ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0)
        ));

        // Reserve succeeds even with conflicts when disabled
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();

        // Overlapping reservation by different robot succeeds when disabled
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(2),
            t(5.0),
            t(15.0),
        )
        .unwrap();
    }

    #[test]
    fn test_reservation_manager_enabled() {
        let mut mgr = ReservationManager::new_enabled();
        assert!(mgr.enabled);

        // First reservation succeeds
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();

        // Same robot can reserve overlapping window
        assert!(mgr.can_reserve(
            &ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(5.0),
            t(15.0)
        ));

        // Different robot cannot reserve overlapping window
        assert!(!mgr.can_reserve(
            &ReservableResource::Edge(EdgeId(1)),
            RobotId(2),
            t(5.0),
            t(15.0)
        ));

        // Different robot can reserve non-overlapping window
        assert!(mgr.can_reserve(
            &ReservableResource::Edge(EdgeId(1)),
            RobotId(2),
            t(10.0),
            t(20.0)
        ));
    }

    #[test]
    fn test_reservation_conflict() {
        let mut mgr = ReservationManager::new_enabled();

        // First reservation
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();

        // Conflicting reservation fails
        let result = mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(2),
            t(5.0),
            t(15.0),
        );

        assert!(result.is_err());
        let conflict = result.unwrap_err();
        assert_eq!(conflict.conflicting_robot, RobotId(1));
        assert_eq!(conflict.conflict_start, t(0.0));
        assert_eq!(conflict.conflict_end, t(10.0));
    }

    #[test]
    fn test_release_all() {
        let mut mgr = ReservationManager::new_enabled();

        // Robot 1 reserves multiple resources
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(2)),
            RobotId(1),
            t(10.0),
            t(20.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Node(NodeId(5)),
            RobotId(1),
            t(0.0),
            t(5.0),
        )
        .unwrap();

        assert_eq!(mgr.reservation_count(), 3);

        // Release all for robot 1
        mgr.release_all(RobotId(1));

        assert_eq!(mgr.reservation_count(), 0);
    }

    #[test]
    fn test_release_specific() {
        let mut mgr = ReservationManager::new_enabled();

        let edge1 = ReservableResource::Edge(EdgeId(1));
        let edge2 = ReservableResource::Edge(EdgeId(2));

        mgr.reserve(edge1.clone(), RobotId(1), t(0.0), t(10.0))
            .unwrap();
        mgr.reserve(edge2.clone(), RobotId(1), t(0.0), t(10.0))
            .unwrap();

        assert_eq!(mgr.reservation_count(), 2);

        // Release only edge1
        mgr.release(&edge1, RobotId(1));

        assert_eq!(mgr.reservation_count(), 1);

        // Robot 2 can now reserve edge1
        assert!(mgr.can_reserve(&edge1, RobotId(2), t(0.0), t(10.0)));
        // But not edge2
        assert!(!mgr.can_reserve(&edge2, RobotId(2), t(0.0), t(10.0)));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut mgr = ReservationManager::new_enabled();

        // Add reservations at different times
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(2)),
            RobotId(1),
            t(5.0),
            t(15.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(3)),
            RobotId(1),
            t(10.0),
            t(20.0),
        )
        .unwrap();

        assert_eq!(mgr.reservation_count(), 3);

        // Cleanup at t=10 should remove first reservation
        mgr.cleanup_expired(t(10.0));

        assert_eq!(mgr.reservation_count(), 2);

        // Cleanup at t=15 should remove second reservation
        mgr.cleanup_expired(t(15.0));

        assert_eq!(mgr.reservation_count(), 1);
    }

    #[test]
    fn test_get_conflicts() {
        let mut mgr = ReservationManager::new_enabled();

        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(2),
            t(15.0),
            t(25.0),
        )
        .unwrap();

        let edge1 = ReservableResource::Edge(EdgeId(1));

        // Robot 3 checking window that overlaps robot 1
        let conflicts = mgr.get_conflicts(&edge1, RobotId(3), t(5.0), t(12.0));
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].robot_id, RobotId(1));

        // Robot 3 checking window that overlaps both
        let conflicts = mgr.get_conflicts(&edge1, RobotId(3), t(5.0), t(20.0));
        assert_eq!(conflicts.len(), 2);

        // Robot 1 checking its own reservation (no conflict with self)
        let conflicts = mgr.get_conflicts(&edge1, RobotId(1), t(0.0), t(10.0));
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_node_reservations() {
        let mut mgr = ReservationManager::new_enabled();

        let node = ReservableResource::Node(NodeId(5));

        mgr.reserve(node.clone(), RobotId(1), t(0.0), t(10.0))
            .unwrap();

        // Different robot cannot reserve same node at overlapping time
        assert!(!mgr.can_reserve(&node, RobotId(2), t(5.0), t(15.0)));

        // But can reserve after
        assert!(mgr.can_reserve(&node, RobotId(2), t(10.0), t(20.0)));
    }

    #[test]
    fn test_get_robot_reservations() {
        let mut mgr = ReservationManager::new_enabled();

        mgr.reserve(
            ReservableResource::Edge(EdgeId(1)),
            RobotId(1),
            t(0.0),
            t(10.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(2)),
            RobotId(1),
            t(10.0),
            t(20.0),
        )
        .unwrap();
        mgr.reserve(
            ReservableResource::Edge(EdgeId(3)),
            RobotId(2),
            t(0.0),
            t(10.0),
        )
        .unwrap();

        let robot1_res = mgr.get_robot_reservations(RobotId(1));
        assert_eq!(robot1_res.len(), 2);

        let robot2_res = mgr.get_robot_reservations(RobotId(2));
        assert_eq!(robot2_res.len(), 1);

        let robot3_res = mgr.get_robot_reservations(RobotId(3));
        assert_eq!(robot3_res.len(), 0);
    }
}
