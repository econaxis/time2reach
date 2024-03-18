use std::collections::{HashMap, HashSet};
use rusqlite::params;

use std::sync::Arc;
use parking_lot::Mutex;
use std::fmt::Write;
use lazy_static::lazy_static;
use serde::Deserialize;
use rusqlite::Connection;

use petgraph::graph::{NodeIndex, UnGraph};

use crate::bicycle_rating::{filter_by_tag, rate_bicycle_friendliness};
use crate::location_index::LocationIndex;

pub type AGraph = UnGraph<Node, Edge>;

pub struct Graph {
    pub graph: AGraph,
    pub location_index: LocationIndex,
    pub db: Mutex<Connection>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "lat")]
    pub lat: f32,
    #[serde(rename = "lon")]
    pub lon: f32,
    pub ele: f32,
}

pub struct NodeWithId {
    pub id: usize,
    pub node: Node,
}

#[derive(Clone, Debug)]
pub struct EdgeWithSourceTarget {
    // 32 bytes per edge
    pub source: usize,
    pub target: usize,
    pub id: u32,
    pub dist: f32,
    pub bike_friendly: u8,
    pub access_friendly: bool,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub id: u32,
    pub dist: f32,
    pub bike_friendly: u8,
    pub access_friendly: bool,
}

impl From<EdgeWithSourceTarget> for Edge {
    fn from(value: EdgeWithSourceTarget) -> Self {
        Self {
            id: value.id,
            dist: value.dist,
            bike_friendly: value.bike_friendly,
            access_friendly: value.access_friendly,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Point {
    pub lat: f32,
    pub lon: f32,
    pub ele: f32,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.lat - other.lat).abs() < 0.0001 && (self.lon - other.lon).abs() < 0.0001
    }
}

fn parse_to_hashmap(input: &str) -> HashMap<String, String> {
    let trimmed_input = if input.starts_with('{') && input.ends_with('}') {
        &input[1..input.len() - 1]
    } else {
        input
    };


    trimmed_input.split(',')
        .map(|kv| kv.trim()) // Trim spaces around the key-value pairs
        .filter_map(|kv| {
            let mut parts = kv.splitn(2, '='); // Split into key and value
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>()
}

impl Graph {
    pub fn new() -> Self {
        let conn = Connection::open("california-big.db").unwrap();
        let mut graph = AGraph::new_undirected();
        let mut node_indices: HashMap<usize, NodeIndex> = HashMap::new();


        let mut statement = conn.prepare("SELECT node_id, lat, lon, ele FROM nodes").unwrap();
        let node_iter = statement.query_map((), |row| {
            Ok(NodeWithId {
                id: row.get(0)?,
                node: Node {
                    lat: row.get(1)?,
                    lon: row.get(2)?,
                    ele: row.get::<_, f64>(3)? as f32, // Cast to f32 as needed
                },
            })
        }).unwrap();

        for node in node_iter {
            let node = node.unwrap(); // Handle errors as needed
            let node_index = graph.add_node(node.node);
            node_indices.insert(node.id, node_index);
        }

        let mut filtered_edges: HashSet<usize> = HashSet::new();
        let mut statement1 = conn.prepare("SELECT id, nodeA, nodeB, dist, kvs FROM edges").unwrap();
        let edge_iter = statement1.query_map((), |row| {
            let kvs_json: String = row.get(4)?;
            let kvs = parse_to_hashmap(&kvs_json);
            // let kvs: HashMap<String, String> = serde_json::from_str(&kvs_json).unwrap();
            let mut access_friendly = filter_by_tag(&kvs);

            // if !access_friendly {
            //     return Ok(None);
            // }
            let bike_friendly = rate_bicycle_friendliness(&kvs);

            if bike_friendly == 0 {
                filtered_edges.insert(row.get(0)?);
                access_friendly = false;
                // return Ok(None)
            }

            Ok(Some(EdgeWithSourceTarget {
                source: row.get(1)?,
                target: row.get(2)?,
                id: row.get(0)?,
                dist: row.get::<_, f64>(3)? as f32, // Cast to f32 as needed
                bike_friendly,
                access_friendly,
            }))
        }).unwrap();

        for edge_option in edge_iter {
            if let Some(edge) = edge_option.unwrap() { // Handle errors as needed
                if let (Some(source_index), Some(target_index)) = (node_indices.get(&edge.source), node_indices.get(&edge.target)) {
                    graph.add_edge(*source_index, *target_index, edge.into());
                }
            }
        }

        // TODO: remove orphaned nodes

        std::mem::drop(statement1);
        std::mem::drop(statement);

        let db = Mutex::new(conn);
        let location_index = LocationIndex::new(&graph, &db);

        Self { graph, location_index, db }
    }
}

impl Point {
    pub fn new(lat: f32, lon: f32) -> Point {
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
    pub(crate) fn haversine_distance(&self, other: &Point) -> f32 {
        const EARTH_RADIUS: f32 = 6371.0; // Earth radius in kilometers

        let d_lat = (other.lat - self.lat).to_radians();
        let d_lon = (other.lon - self.lon).to_radians();

        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + self.lat.to_radians().cos() * other.lat.to_radians().cos()
            * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c * 1000.0
    }
}

impl Node {
    pub(crate) fn debug_coords(&self) -> (f32, f32) {
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
    static ref POINTS_CACHE: Mutex<HashMap<u32, Arc<Vec<Point>>>> = Mutex::new(HashMap::new());
}

impl Edge {
    pub fn points(&self, conn: &Mutex<Connection>) -> Arc<Vec<Point>> {
        let conn_guard = conn.lock();
        let cache_lock = POINTS_CACHE.lock();

        // Check if points are already in the cache
        if let Some(points) = cache_lock.get(&self.id) {
            return Arc::clone(points);
        }
        drop(cache_lock); // Explicitly drop the lock to avoid holding it while querying the database

        // Prepare the query for edge points
        let mut statement = conn_guard.prepare("SELECT lat, lon, ele FROM edge_points WHERE edge_id = ? ORDER BY point_id").unwrap();

        // Execute the query and map rows to Point instances
        let points_result = statement.query_map(params![self.id as i64], |row| {
            Ok(Point {
                lat: row.get(0)?,
                lon: row.get(1)?,
                ele: row.get(2)?,
            })
        }).unwrap();

        // Collect points and handle potential errors
        let mut points: Vec<Point> = Vec::new();
        for point_result in points_result {
            if let Ok(point) = point_result {
                points.push(point);
            }
        }
        let points_arc = Arc::new(points);
        let mut cache_lock = POINTS_CACHE.lock();
        // Insert the points into the cache and return. This pattern avoids the issue where the points
        // could be inserted into the cache while it was unlocked and queried.
        cache_lock.entry(self.id).or_insert_with(|| Arc::clone(&points_arc));

        points_arc
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let input = "{key1=value1, key2=value2}";
        let expected: HashMap<String, String> = [("key1", "value1"), ("key2", "value2")]
            .iter().map(|&(k, v)| (k.into(), v.into())).collect();
        let result = parse_to_hashmap(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_values_with_spaces() {
        let input = "{name=John Doe, occupation=Software Developer}";
        let expected: HashMap<String, String> = [("name", "John Doe"), ("occupation", "Software Developer")]
            .iter().map(|&(k, v)| (k.into(), v.into())).collect();
        let result = parse_to_hashmap(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numerical_values_and_mixed_characters() {
        let input = "{age=30, location=New York, score=9.5, special_char=!@#}";
        let expected: HashMap<String, String> = [("age", "30"), ("location", "New York"), ("score", "9.5"), ("special_char", "!@#")]
            .iter().map(|&(k, v)| (k.into(), v.into())).collect();
        let result = parse_to_hashmap(input);
        assert_eq!(result, expected);
    }
}