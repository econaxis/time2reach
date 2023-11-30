use serde::Deserialize;
use std::collections::HashMap;
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::fs::File;
use serde_json::Value;
use std::io::Read;
use crate::bicycle_rating::rate_bicycle_friendliness;
use crate::bicycle_rating::filter_by_tag;
use crate::parse_graph::PointSnap;

pub type AGraph = UnGraph<Node, Edge>;

pub struct Graph {
    pub graph: AGraph,
    pub location_index: LocationIndex,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: usize,
    #[serde(rename = "lat")]
    pub lat: f64,
    #[serde(rename = "lon")]
    pub lon: f64,
    pub ele: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
}


impl Point {
    fn eq(&self, b: &Point) -> bool {
        self.haversine_distance(b) <= 0.001
    }
    pub(crate) fn haversine_distance_ele(&self, other: &Point) -> f64 {
        const EARTH_RADIUS: f64 = 6371.0; // Earth radius in kilometers

        let d_lat = (other.lat - self.lat).to_radians();
        let d_lon = (other.lon - self.lon).to_radians();

        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + self.lat.to_radians().cos() * other.lat.to_radians().cos()
            * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

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
    pub fn new(edge_indices: &HashMap<usize, EdgeIndex>, edges: Vec<Edge>) -> LocationIndex {
        LocationIndex {
            points: edges.iter().map(|edge| PointSnap {
                point: Point {
                    lat: edge.points[0].lat,
                    lon: edge.points[0].lon,
                    ele: edge.points[0].ele,
                },
                edge_id: edge_indices[&edge.id].index(),
            }).collect(),
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

#[derive(Debug, Deserialize, Clone)]
pub struct Edge {
    pub id: usize,
    #[serde(rename = "nodeA")]
    pub source: usize,
    #[serde(rename = "nodeB")]
    pub target: usize,
    pub dist: f64,
    #[serde(rename = "kvs")]
    pub kvs: HashMap<String, String>,
    pub points: Vec<Point>,
}

pub fn parse_graph() -> Graph {
    let mut file = File::open("/Users/henry/graphhopper/export.json").unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str);

    let result: Value = serde_json::from_str(&json_str).expect("Error parsing JSON");

    let nodes: Vec<Node> =
        serde_json::from_value(result["nodes"].clone()).expect("Error parsing nodes");
    let edges: Vec<Edge> =
        serde_json::from_value(result["edges"].clone()).expect("Error parsing edges");

    // Create a petgraph from the parsed nodes and edges
    let mut graph = AGraph::new_undirected();
    let node_indices: Vec<NodeIndex> = nodes
        .iter()
        .map(|node| graph.add_node(node.clone()))
        .collect();


    let mut edge_indices: HashMap<usize, EdgeIndex> = HashMap::new();
    let (_changed, edges) = filter_ways_by_bike_friendliness(edges);

    for edge in &edges {
        let source_index = node_indices[edge.source];
        let target_index = node_indices[edge.target];

        let edge_index = graph.add_edge(source_index, target_index, edge.clone());
        edge_indices.insert(edge.id, edge_index);
        // let edge_index1 = graph.add_edge(target_index, source_index, edge.reverse());
    }

    let location_index = LocationIndex::new(&edge_indices, edges);

    Graph {
        graph,
        location_index,
    }
}


fn filter_ways_by_bike_friendliness(edges: Vec<Edge>) -> (bool, Vec<Edge>) {
    let mut changed = false;
    let new_edges = edges.into_iter().filter(|edge| {
        if filter_by_tag(&edge.kvs) == false {
            changed = true;
            return false;
        }
        // Filter nodes
        match rate_bicycle_friendliness(&edge.kvs) {
            0 => {
                changed = true;
                return false;
            }
            _ => {}
        }
        return true;
    }).collect::<Vec<Edge>>();

    (changed, new_edges)
}
