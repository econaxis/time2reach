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

    let bicyle_friendly = edge.bike_friendly;

    // from 0 -> 1. The higher the number, the worse it is.
    let bicycle_penalty: f64 = match bicyle_friendly {
        0 => {
            return DO_NOT_EXPLORE;
        },
        1 => 1.0,
        2 => 0.7,
        3 => 0.65,
        4 => 0.5,
        5 => 0.5,
        _ => panic!("Invalid bicycle rating"),
    };

    let bicycle_penalty_scaled = (bicycle_penalty + 0.5) * (options.prefer_protected_bike_lanes) * (edge.dist as f64);

    let source = graph.node_weight(edgeref.source()).unwrap();
    let target = graph.node_weight(edgeref.target()).unwrap();
    let elevation_diff = (target.ele - source.ele) as f64;

    let avoid_steep_hills_scaled = 90000.0 * options.avoid_steep_hills.powi(30);
    let elevation_penalty = elevation_diff.max(0.0);

    edge.dist as f64 + elevation_penalty * avoid_steep_hills_scaled + bicycle_penalty_scaled
}
