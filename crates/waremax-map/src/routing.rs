//! Routing algorithms for finding paths in the warehouse map

use waremax_core::NodeId;
use crate::graph::WarehouseMap;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// A computed route through the warehouse
#[derive(Clone, Debug)]
pub struct Route {
    pub path: Vec<NodeId>,
    pub total_distance: f64,
}

impl Route {
    pub fn empty(start: NodeId) -> Self {
        Self {
            path: vec![start],
            total_distance: 0.0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.path.len() <= 1
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }
}

/// Cache for computed routes
pub struct RouteCache {
    cache: HashMap<(NodeId, NodeId), Route>,
    max_size: usize,
}

impl RouteCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, from: NodeId, to: NodeId) -> Option<&Route> {
        self.cache.get(&(from, to))
    }

    pub fn insert(&mut self, from: NodeId, to: NodeId, route: Route) {
        if self.cache.len() >= self.max_size {
            let keys: Vec<_> = self.cache.keys().take(self.max_size / 2).copied().collect();
            for key in keys {
                self.cache.remove(&key);
            }
        }
        self.cache.insert((from, to), route);
    }

    pub fn invalidate(&mut self) {
        self.cache.clear();
    }
}

/// Router for finding paths in the warehouse
pub struct Router {
    cache: RouteCache,
    cache_enabled: bool,
}

impl Router {
    pub fn new(cache_enabled: bool) -> Self {
        Self {
            cache: RouteCache::new(10000),
            cache_enabled,
        }
    }

    pub fn find_route(&mut self, map: &WarehouseMap, from: NodeId, to: NodeId) -> Option<Route> {
        if from == to {
            return Some(Route::empty(from));
        }

        if self.cache_enabled {
            if let Some(route) = self.cache.get(from, to) {
                return Some(route.clone());
            }
        }

        let route = self.dijkstra(map, from, to)?;

        if self.cache_enabled {
            self.cache.insert(from, to, route.clone());
        }

        Some(route)
    }

    fn dijkstra(&self, map: &WarehouseMap, from: NodeId, to: NodeId) -> Option<Route> {
        #[derive(Clone, PartialEq)]
        struct State {
            cost: f64,
            node: NodeId,
        }

        impl Eq for State {}

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                other.cost.partial_cmp(&self.cost)
            }
        }

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap_or(Ordering::Equal)
            }
        }

        let mut dist: HashMap<NodeId, f64> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(from, 0.0);
        heap.push(State { cost: 0.0, node: from });

        while let Some(State { cost, node }) = heap.pop() {
            if node == to {
                let mut path = vec![to];
                let mut current = to;

                while let Some(&prev_node) = prev.get(&current) {
                    path.push(prev_node);
                    current = prev_node;
                }

                path.reverse();
                return Some(Route {
                    path,
                    total_distance: cost,
                });
            }

            if let Some(&d) = dist.get(&node) {
                if cost > d {
                    continue;
                }
            }

            for (neighbor, _edge_id, length) in map.neighbors(node) {
                let next_cost = cost + length;

                if dist.get(&neighbor).map_or(true, |&d| next_cost < d) {
                    dist.insert(neighbor, next_cost);
                    prev.insert(neighbor, node);
                    heap.push(State {
                        cost: next_cost,
                        node: neighbor,
                    });
                }
            }
        }

        None
    }

    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate();
    }
}
