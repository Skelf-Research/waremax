//! Routing algorithms for finding paths in the warehouse map

use crate::graph::WarehouseMap;
use crate::traffic::TrafficManager;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use waremax_core::{EdgeId, NodeId};

/// v1: Routing algorithm selection
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RoutingAlgorithm {
    #[default]
    Dijkstra,
    AStar,
}

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
    /// v1: Routing algorithm
    algorithm: RoutingAlgorithm,
    /// v1: Congestion weight for congestion-aware routing (0.0 = no penalty)
    congestion_weight: f64,
}

impl Router {
    pub fn new(cache_enabled: bool) -> Self {
        Self {
            cache: RouteCache::new(10000),
            cache_enabled,
            algorithm: RoutingAlgorithm::default(),
            congestion_weight: 0.0,
        }
    }

    /// v1: Create router with specific algorithm
    pub fn with_algorithm(cache_enabled: bool, algorithm: RoutingAlgorithm) -> Self {
        Self {
            cache: RouteCache::new(10000),
            cache_enabled,
            algorithm,
            congestion_weight: 0.0,
        }
    }

    /// v1: Set congestion weight for congestion-aware routing
    pub fn set_congestion_weight(&mut self, weight: f64) {
        self.congestion_weight = weight;
    }

    /// v1: Get current routing algorithm
    pub fn algorithm(&self) -> &RoutingAlgorithm {
        &self.algorithm
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

        let route = match self.algorithm {
            RoutingAlgorithm::Dijkstra => self.dijkstra(map, from, to, None),
            RoutingAlgorithm::AStar => self.astar(map, from, to, None),
        }?;

        if self.cache_enabled {
            self.cache.insert(from, to, route.clone());
        }

        Some(route)
    }

    /// v1: Find route with congestion awareness
    pub fn find_route_with_traffic(
        &mut self,
        map: &WarehouseMap,
        from: NodeId,
        to: NodeId,
        traffic: &TrafficManager,
    ) -> Option<Route> {
        if from == to {
            return Some(Route::empty(from));
        }

        // Don't use cache when congestion-aware (traffic state changes)
        match self.algorithm {
            RoutingAlgorithm::Dijkstra => self.dijkstra(map, from, to, Some(traffic)),
            RoutingAlgorithm::AStar => self.astar(map, from, to, Some(traffic)),
        }
    }

    /// v1: Find route avoiding specific edges
    pub fn find_route_avoiding(
        &mut self,
        map: &WarehouseMap,
        from: NodeId,
        to: NodeId,
        avoid_edges: &[EdgeId],
        traffic: Option<&TrafficManager>,
    ) -> Option<Route> {
        if from == to {
            return Some(Route::empty(from));
        }

        let avoid_set: HashSet<EdgeId> = avoid_edges.iter().copied().collect();
        self.dijkstra_avoiding(map, from, to, &avoid_set, traffic)
    }

    /// Calculate edge cost with optional congestion penalty and speed multiplier
    fn edge_cost(
        &self,
        map: &WarehouseMap,
        length: f64,
        edge_id: EdgeId,
        traffic: Option<&TrafficManager>,
    ) -> f64 {
        // Apply speed multiplier (v2: fast lanes/express paths)
        let speed_multiplier = map
            .get_edge(edge_id)
            .map(|e| e.speed_multiplier)
            .unwrap_or(1.0);
        let base_cost = length * speed_multiplier;

        // Apply congestion penalty if enabled
        if self.congestion_weight > 0.0 {
            if let Some(tm) = traffic {
                let occupancy = tm.get_edge_occupancy(edge_id);
                return base_cost * (1.0 + self.congestion_weight * occupancy as f64);
            }
        }
        base_cost
    }

    fn dijkstra(
        &self,
        map: &WarehouseMap,
        from: NodeId,
        to: NodeId,
        traffic: Option<&TrafficManager>,
    ) -> Option<Route> {
        #[derive(Clone, PartialEq)]
        struct State {
            cost: f64,
            node: NodeId,
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other
                    .cost
                    .partial_cmp(&self.cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut dist: HashMap<NodeId, f64> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(from, 0.0);
        heap.push(State {
            cost: 0.0,
            node: from,
        });

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

            for (neighbor, edge_id, length) in map.neighbors(node) {
                let edge_cost = self.edge_cost(map, length, edge_id, traffic);
                let next_cost = cost + edge_cost;

                if dist.get(&neighbor).is_none_or(|&d| next_cost < d) {
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

    /// v1: Dijkstra avoiding specific edges
    fn dijkstra_avoiding(
        &self,
        map: &WarehouseMap,
        from: NodeId,
        to: NodeId,
        avoid_edges: &HashSet<EdgeId>,
        traffic: Option<&TrafficManager>,
    ) -> Option<Route> {
        #[derive(Clone, PartialEq)]
        struct State {
            cost: f64,
            node: NodeId,
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other
                    .cost
                    .partial_cmp(&self.cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut dist: HashMap<NodeId, f64> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(from, 0.0);
        heap.push(State {
            cost: 0.0,
            node: from,
        });

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

            for (neighbor, edge_id, length) in map.neighbors(node) {
                // Skip avoided edges
                if avoid_edges.contains(&edge_id) {
                    continue;
                }

                let edge_cost = self.edge_cost(map, length, edge_id, traffic);
                let next_cost = cost + edge_cost;

                if dist.get(&neighbor).is_none_or(|&d| next_cost < d) {
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

    /// v1: A* algorithm with euclidean heuristic
    fn astar(
        &self,
        map: &WarehouseMap,
        from: NodeId,
        to: NodeId,
        traffic: Option<&TrafficManager>,
    ) -> Option<Route> {
        #[derive(Clone, PartialEq)]
        struct State {
            f_cost: f64, // f = g + h
            g_cost: f64, // actual cost from start
            node: NodeId,
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other
                    .f_cost
                    .partial_cmp(&self.f_cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut g_score: HashMap<NodeId, f64> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        g_score.insert(from, 0.0);
        let h = map.euclidean_distance(from, to);
        heap.push(State {
            f_cost: h,
            g_cost: 0.0,
            node: from,
        });

        while let Some(State { g_cost, node, .. }) = heap.pop() {
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
                    total_distance: g_cost,
                });
            }

            if let Some(&g) = g_score.get(&node) {
                if g_cost > g {
                    continue;
                }
            }

            for (neighbor, edge_id, length) in map.neighbors(node) {
                let edge_cost = self.edge_cost(map, length, edge_id, traffic);
                let tentative_g = g_cost + edge_cost;

                if g_score.get(&neighbor).is_none_or(|&g| tentative_g < g) {
                    g_score.insert(neighbor, tentative_g);
                    prev.insert(neighbor, node);

                    let h = map.euclidean_distance(neighbor, to);
                    heap.push(State {
                        f_cost: tentative_g + h,
                        g_cost: tentative_g,
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
