mod parse_graph;
mod bicycle_rating;
mod real_edge_weight;
mod graph;

pub use parse_graph::*;
pub use bicycle_rating::rate_bicycle_friendliness;


fn main() {
    parse_graph::main();
}
