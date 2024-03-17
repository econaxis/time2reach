extern crate core;
mod parse_graph;
mod location_index;
mod bicycle_rating;
mod virtual_graph;
mod graph;
mod real_edge_weight;
mod calories;


pub use parse_graph::*;
pub use bicycle_rating::rate_bicycle_friendliness;
pub use graph::*;
pub use real_edge_weight::RouteOptions;

fn main() {
    parse_graph::main();
}
