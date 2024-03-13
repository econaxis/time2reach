use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use core::cell::RefCell;
use std::fmt::Write;
use std::io::Read;

use lazy_static::lazy_static;
use serde::{Deserialize};
use sqlite::{Connection, State};

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::prelude::EdgeRef;

use crate::bicycle_rating::{filter_by_tag, rate_bicycle_friendliness};
use crate::parse_graph::PointSnap;

pub type AGraph = UnGraph<Node, Edge>;

pub struct Graph {
    pub graph: AGraph,
    pub location_index: LocationIndex,
    pub db: Mutex<Connection>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    // 32 bytes per node
    pub id: usize,
    #[serde(rename = "lat")]
    pub lat: f64,
    #[serde(rename = "lon")]
    pub lon: f64,
    pub ele: f32,
}

#[derive(Clone, Debug)]
pub struct Edge {
    // 32 bytes per edge
    pub id: usize,
    pub source: usize,
    pub target: usize,
    pub dist: f32,
    pub bike_friendly: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
    pub ele: f32,
}

impl Graph {
    pub fn new() -> Self {
        let conn = Connection::open("elevation-big.db").unwrap();
        let mut graph = AGraph::new_undirected();
        let mut node_indices: HashMap<usize, NodeIndex> = HashMap::new();


        // Query and insert nodes
        let mut statement = conn.prepare("SELECT node_id, lat, lon, ele FROM nodes").unwrap();
        while let State::Row = statement.next().unwrap() {
            let node = Node {
                id: statement.read::<i64, _>(0).unwrap() as usize,
                lat: statement.read::<f64, _>(1).unwrap(),
                lon: statement.read::<f64, _>(2).unwrap(),
                ele: statement.read::<f64, _>(3).unwrap() as f32,
            };
            let node_index = graph.add_node(node.clone());
            node_indices.insert(node.id, node_index);
        }

        // Query and insert edges
        let mut statement1 = conn.prepare("SELECT id, nodeA, nodeB, dist, kvs FROM edges").unwrap();
        while let State::Row = statement1.next().unwrap() {
            let kvs_json: String = statement1.read(4).unwrap();
            let kvs: HashMap<String, String> = serde_json::from_str(&kvs_json).unwrap();
            if !filter_by_tag(&kvs) {
                continue;
            }
            let bike_friendly = rate_bicycle_friendliness(&kvs); // Implement this function based on your criteria

            if bike_friendly == 0 {
                continue;
            }

            let edge = Edge {
                id: statement1.read::<i64, _>(0).unwrap() as usize,
                source: statement1.read::<i64, _>(1).unwrap() as usize,
                target: statement1.read::<i64, _>(2).unwrap() as usize,
                dist: statement1.read::<f64, _>(3).unwrap() as f32,
                bike_friendly,
            };

            if let (Some(source_index), Some(target_index)) = (node_indices.get(&edge.source), node_indices.get(&edge.target)) {
                graph.add_edge(*source_index, *target_index, edge);
            }
        }

        // TODO: remove orphaned nodes

        std::mem::drop(statement1);
        std::mem::drop(statement);

        let db =  Mutex::new(conn);
        let location_index = LocationIndex::new(&graph, &db);

        Self { graph, location_index, db }
    }
}
impl Point {
    pub fn new(lat: f64, lon: f64) -> Point {
        Point { lat, lon, ele: 0.0 }
    }
    fn eq(&self, b: &Point) -> bool {
        self.haversine_distance(b) <= 0.001
    }
    pub(crate) fn haversine_distance_ele(&self, other: &Point) -> f32 {
        const EARTH_RADIUS: f32 = 6371.0; // Earth radius in kilometers

        let d_lat = (other.lat - self.lat).to_radians();
        let d_lon = (other.lon - self.lon).to_radians();

        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + self.lat.to_radians().cos() * other.lat.to_radians().cos()
            * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt()) as f32;

        let distance = EARTH_RADIUS * c * 1000.0;

        // Add elevation difference to the distance
        let ele_diff = other.ele - self.ele;
        let ele_adjustment = ele_diff.abs(); // You can modify this based on your use case

        distance + ele_adjustment
    }
    pub(crate) fn haversine_distance(&self, other: &Point) -> f64 {
        const EARTH_RADIUS: f64 = 6371.0; // Earth radius in kilometers

        let d_lat = (other.lat - self.lat).to_radians();
        let d_lon = (other.lon - self.lon).to_radians();

        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + self.lat.to_radians().cos() * other.lat.to_radians().cos()
            * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c * 1000.0
    }
}

pub struct LocationIndex {
    points: Vec<PointSnap>,
}

impl LocationIndex {
    pub fn new(graph: &AGraph, conn: &Mutex<Connection>) -> LocationIndex {
        let points = graph.edge_indices().flat_map(|edge_index| {
            let (node_a, node_b) = graph.edge_endpoints(edge_index).unwrap();
            // todo: don't use edge_id anymore
            let node_a: &Node = &graph[node_a];
            let node_b: &Node = &graph[node_b];
            [PointSnap { point: node_a.point(), edge_id: edge_index.index() },
             PointSnap { point: node_b.point(), edge_id: edge_index.index() }]
        }).collect();
        LocationIndex {
            points,
        }
    }
    pub(crate) fn snap_closest(&self, point: &Point) -> &PointSnap {
        // TODO: Use a spatial index to speed this up
        let mut best = None;
        let mut best_dist = std::f64::MAX;
        for point_snap in &self.points {
            let dist = (point_snap.point.lat - point.lat).powi(2)
                + (point_snap.point.lon - point.lon).powi(2);
            if dist < best_dist {
                best_dist = dist;
                best = Some(point_snap);
            }
        }
        best.unwrap()
    }
}

impl Node {
    pub(crate) fn debug_coords(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    pub fn point(&self) -> Point {
        Point {
            lat: self.lat,
            lon: self.lon,
            ele: self.ele,
        }
    }
}

lazy_static! {
    static ref POINTS_CACHE: Mutex<HashMap<usize, Arc<Vec<Point>>>> = Mutex::new(HashMap::new());
}

impl Edge {
    pub fn points(&self, conn: &Mutex<Connection>) -> Arc<Vec<Point>> {
        let conn = conn.lock();
        let cache_lock = POINTS_CACHE.lock();
        if let Some(points) = cache_lock.get(&self.id) {
            return Arc::clone(points);
        }
        drop(cache_lock); // Explicitly drop the lock to avoid holding it while querying the database

        // Query the points from the database as they're not found in the cache
        let mut points: Vec<Point> = Vec::new();
        let mut statement = conn.prepare("SELECT lat, lon, ele FROM edge_points WHERE edge_id = ? ORDER BY point_id").unwrap();
        statement.bind((1, self.id as i64)).unwrap();

        while let sqlite::State::Row = statement.next().unwrap() {
            points.push(Point {
                lat: statement.read::<f64, _>(0).unwrap(),
                lon: statement.read::<f64, _>(1).unwrap(),
                ele: statement.read::<f64, _>(2).unwrap() as f32,
            });
        }

        let points_arc = Arc::new(points);
        let mut cache_lock = POINTS_CACHE.lock();
        // Insert the points into the cache and return. This pattern avoids the issue where the points
        // could be inserted into the cache while it was unlocked and queried.
        cache_lock.entry(self.id).or_insert_with(|| Arc::clone(&points_arc));

        points_arc
    }
}
