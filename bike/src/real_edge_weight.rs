use crate::rate_bicycle_friendliness;
use petgraph::graph::EdgeReference;
use petgraph::prelude::EdgeRef;
use crate::graph::{AGraph, Edge};

const DO_NOT_EXPLORE: f64 = std::f64::MAX;

pub fn real_edge_weight<'a>(graph: &'a AGraph, edgeref: EdgeReference<'a, Edge>) -> f64 {
    // Calculate the real edge weight including elevation difference
    let edge = edgeref.weight();

    let bicyle_friendly = rate_bicycle_friendliness(&edge.kvs);

    let bicycle_penalty = match bicyle_friendly {
        0 => {
            return DO_NOT_EXPLORE;
        },
        1 => 1.90,
        2 => 1.40,
        3 => 1.0,
        4 => 0.90,
        5 => 0.85,
        _ => panic!("Invalid bicycle rating"),
    };

    let source = graph.node_weight(edgeref.source()).unwrap();
    let target = graph.node_weight(edgeref.target()).unwrap();
    let elevation_diff = target.ele - source.ele;
    let slope = elevation_diff / edge.dist;

    let elevation_penalty = (slope * 10.0).powf(2.0).abs() * elevation_diff.signum() * edge.dist;
    let elevation_penalty = if elevation_penalty.is_nan() { 0.0 } else { elevation_penalty };

    (edge.dist + elevation_penalty * 1.0).max(edge.dist * 0.85) * bicycle_penalty
}
