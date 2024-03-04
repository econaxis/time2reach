use crate::Energy;
use petgraph::graph::NodeIndex;
use crate::AGraph;

/// Calculates the calories used for a route.

const CALORIES_PER_WATT: f64 = 69.78;
const JOULES_PER_CALORIE: f64 = 4186.0;

fn calculate_watts(
    // m / s
    speed: f64,
    // m / s
    headwind: f64,
    // Mass of cyclist + bicycle
    mass_kg: f64,
    // fraction
    slope: f64,
    // m/s^2
    acceleration: f64,
) -> f64 {
    // https://www.cptips.com/formula.htm
    let cv_cw = speed + headwind;
    let k1 = 7.0;
    let k2 = 0.3153;

    let watts = speed * (k1 + k2 * cv_cw.powi(2) + (10.50 * mass_kg * (slope + 1.01 * acceleration / 9.806)));

    watts
}



fn calculate_calories_for_edge(from: NodeIndex, to: NodeIndex, graph: &AGraph) -> Energy {
    let source = graph.node_weight(from).unwrap();
    let target = graph.node_weight(to).unwrap();

    let edge = graph.find_edge(from, to).unwrap();
    let edge_dist = graph.edge_weight(edge).unwrap().dist;

    let elevation_diff = target.ele - source.ele;
    let slope = elevation_diff / edge_dist;

    let average_speed = match slope {
        (0.02..=0.1) => 6.0,
        (0.1..=0.2) => 4.5,
        (0.2..) => 3.5,
        (-0.02..=0.02) => 6.26,
        (-0.1..=0.0) => 6.6,
        (..=-0.1) => 7.0,
        _ => 6.0,
    };

    let watts = calculate_watts(
        average_speed, // m/s
        0.0, // m/s
        86.0, // kg
        slope, // percent
        0.0, // m/s^2
    ).max(5.0);

    let seconds_per_edge = edge_dist / average_speed;
    let calories = watts * seconds_per_edge / JOULES_PER_CALORIE / CALORIE_INEFFICIENCY;
    let uphill_meters = elevation_diff.max(0.0);
    let downhill_meters = elevation_diff.min(0.0);

    Energy {
        calories,
        uphill_meters,
        downhill_meters,
        total_meters: edge_dist,
    }
}

const CALORIE_INEFFICIENCY: f64 = 0.25;

pub fn calculate_energy(points: &[NodeIndex], graph: &AGraph) -> Energy {
    let mut total_energy = Energy::default();

    points.windows(2).for_each(|indices| {
        let prev = indices[0];
        let cur = indices[1];

        let energy = calculate_calories_for_edge(prev, cur, graph);
        total_energy += energy;
    });

    total_energy
}