use serde::Deserialize;
use crate::rate_bicycle_friendliness;
use petgraph::graph::EdgeReference;
use petgraph::prelude::EdgeRef;
use crate::graph::{AGraph, Edge};
use serde;

const DO_NOT_EXPLORE: f64 = std::f64::MAX;

#[derive(Deserialize, Debug, Clone)]
pub struct RouteOptions {
    /// Each value between 0 to 1 depending on how important it is
    #[serde(rename="avoidHills")]
    pub avoid_steep_hills: f64,

    #[serde(rename="preferProtectedLanes")]
    pub prefer_protected_bike_lanes: f64,
}

impl Default for RouteOptions {
    fn default() -> Self {
        Self {
            avoid_steep_hills: 0.5,
            prefer_protected_bike_lanes: 0.5,
        }
    }
}
pub fn real_edge_weight<'a>(graph: &'a AGraph, edgeref: EdgeReference<'a, Edge>, options: &RouteOptions) -> f64 {
    // Calculate the real edge weight including elevation difference
    let edge = edgeref.weight();

    let bicyle_friendly = rate_bicycle_friendliness(&edge.kvs);

    // from 0 -> 1. The higher the number, the worse it is.
    let bicycle_penalty = match bicyle_friendly {
        0 => {
            return DO_NOT_EXPLORE;
        },
        1 => 1.0,
        2 => 0.7,
        3 => 0.5,
        4 => 0.2,
        5 => 0.1,
        _ => panic!("Invalid bicycle rating"),
    };

    let bicycle_penalty_scaled = (bicycle_penalty + 0.5f64).powf(options.prefer_protected_bike_lanes + 1.0);

    let source = graph.node_weight(edgeref.source()).unwrap();
    let target = graph.node_weight(edgeref.target()).unwrap();
    let elevation_diff = target.ele - source.ele;
    let slope = elevation_diff / edge.dist;

    let elevation_penalty = (slope * 15.0).powf(2.0).abs() * elevation_diff.signum() * edge.dist;
    let elevation_penalty = if elevation_penalty.is_nan() { 0.0 } else { elevation_penalty };

    (edge.dist + elevation_penalty * options.avoid_steep_hills).max(edge.dist * 0.85) * bicycle_penalty_scaled
}
