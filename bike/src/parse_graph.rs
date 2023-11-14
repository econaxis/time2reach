use geojson::{FeatureCollection, Value as GeoJsonValue};
use geojson::{Feature, GeoJson, Geometry};
use petgraph::algo::astar;
use petgraph::graph::{EdgeIndex, EdgeReference, NodeIndex, UnGraph};
use petgraph::prelude::EdgeRef;
use petgraph::{Undirected};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use petgraph::matrix_graph::Zero;
use petgraph::visit::IntoEdgeReferences;

type AGraph = UnGraph<Node, Edge>;

pub struct Graph {
    graph: AGraph,
    location_index: LocationIndex,
}

#[derive(Debug, Deserialize, Clone)]
struct Node {
    id: usize,
    #[serde(rename = "lat")]
    lat: f64,
    #[serde(rename = "lon")]
    lon: f64,
    ele: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
}


impl Point {
    fn haversine_distance(&self, other: &Point) -> f64 {
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

struct PointSnap {
    point: Point,
    edge_id: usize,
}

struct LocationIndex {
    points: Vec<PointSnap>,
}

impl LocationIndex {
    fn snap_closest(&self, point: &Point) -> &PointSnap {
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
    fn debug_coords(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    fn point(&self) -> Point {
        Point {
            lat: self.lat,
            lon: self.lon,
            ele: self.ele,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Edge {
    id: usize,
    #[serde(rename = "nodeA")]
    source: usize,
    #[serde(rename = "nodeB")]
    target: usize,
    dist: f64,
    #[serde(rename = "kvs")]
    kvs: HashMap<String, String>,
    points: Vec<Point>,
}

fn real_edge_weight<'a>(graph: &'a Graph, edgeref: EdgeReference<'a, Edge>) -> f64 {
    // Calculate the real edge weight including elevation difference
    let edge = edgeref.weight();
    let source = graph.graph.node_weight(edgeref.source()).unwrap();
    let target = graph.graph.node_weight(edgeref.target()).unwrap();
    let elevation_diff = target.ele - source.ele;
    let slope = elevation_diff / edge.dist;

    let elevation_penalty = (slope * 200.0).powf(2.0).abs() * elevation_diff.signum() * edge.dist;
    let elevation_penalty = if elevation_penalty.is_nan() { 0.0 } else { elevation_penalty };


    println!("{} {}", edge.dist, elevation_penalty);

    (edge.dist + elevation_penalty * 1.0).max(edge.dist * 0.85)
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
    for edge in &edges {
        let source_index = node_indices[edge.source];
        let target_index = node_indices[edge.target];

        let edge_index = graph.add_edge(source_index, target_index, edge.clone());
        edge_indices.insert(edge.id, edge_index);
        // let edge_index1 = graph.add_edge(target_index, source_index, edge.reverse());
    }

    let location_index = LocationIndex {
        points: edges.iter().map(|edge| PointSnap {
            point: Point {
                lat: edge.points[0].lat,
                lon: edge.points[0].lon,
                ele: edge.points[0].ele,
            },
            edge_id: edge_indices[&edge.id].index(),
        }).collect(),
    };

    Graph {
        graph,
        location_index,
    }
}

fn point_list_geojson(g: &AGraph) -> GeoJson {
    let mut features = Vec::new();

    for edge in g.edge_references() {
        let edge_data = edge.weight();

        // Convert each point_list in each edge to a GeoJSON LineString
        let line_coordinates: Vec<Vec<f64>> = edge_data.points
            .iter()
            .map(|point| vec![point.lon, point.lat])
            .collect();

        let line_string = GeoJsonValue::LineString(line_coordinates);

        // Create a GeoJSON feature for each edge
        let feature_properties = json!({
            "id": edge.id().index(),
            "source": edge_data.source,
            "dest": edge_data.target
        });


        let feature = Feature {
            bbox: None,
            geometry: Some(Geometry::new(line_string)),
            id: None,
            properties: feature_properties.as_object().cloned(),
            foreign_members: None,
        };

        features.push(feature);
    }

    GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features: features,
        foreign_members: None,
    })
}

pub fn main() {
    let graph = parse_graph();

    // let gj = point_list_geojson(&graph.graph);
    // println!("{}", serde_json::to_string(&gj).unwrap());

    // Replace these coordinates with your actual coordinates
    const START: (f64, f64) = (37.73167959059285, -122.44497075710883);
    const END: (f64, f64) = (37.766034778541155, -122.45105332934128);
    let start = Point {
        lat: START.0,
        lon: START.1,
        ele: 0.0,
    };
    let end = Point {
        lat: END.0,
        lon: END.1,
        ele: 0.0,
    };

    route(&graph, start, end);
}
pub fn route(graph: &Graph, start: Point, end: Point) -> anyhow::Result<GeoJson> {
    // Find the nearest nodes in the graph to the specified coordinates
    let start_nodes = find_nearest_nodes(&graph, &start);
    let end_nodes = find_nearest_nodes(&graph, &end);

    let start_node = start_nodes[0];

    println!("Start node: {} {:?}", start_node.index(), graph.graph[start_node].debug_coords());

    for end_node in &end_nodes {
        println!("End node: {} {:?}", end_node.index(), graph.graph[*end_node].debug_coords());
    }

    println!("DIST {}", start.haversine_distance(&end));
    // Perform A* routing
    let mut points_checked = Vec::new();

    if let Some((cost, path)) = astar(
        &graph.graph,
        start_node,
        |finish: NodeIndex| {
            end_nodes.contains(&finish)
        },
        |e| {
            real_edge_weight(&graph, e)
        }, // Cost function, using edge distance
        |node: NodeIndex| {
            points_checked.push(node);

            let node = &graph.graph[node];
            let point = node.point();

            let cost = end.haversine_distance(&point);
            cost
        },
    ) {
        println!("Total cost: {}", cost);

        let mut dist_accum = 0.0;
        let heights = path.as_slice().windows(2).map(|node_index| {
            let prev = node_index[0];
            let cur = node_index[1];

            let edge = graph.graph.find_edge(prev, cur).unwrap();
            let node = &graph.graph[cur];
            let dist_accum_prev = dist_accum;
            dist_accum += graph.graph[edge].dist;
            (dist_accum_prev, node.ele)
        }).collect::<Vec<(f64, f64)>>();

        // Convert the path to GeoJSON LineString
        let coordinates: Vec<Vec<f64>> = path
            .iter()
            .map(|&node_index| {
                let node = &graph.graph[node_index];
                vec![node.lon, node.lat]
            })
            .collect();

        let line_string = GeoJsonValue::LineString(coordinates);
        let geometry = Geometry::new(line_string);

        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        };

        let geojson = GeoJson::Feature(feature);
        println!("{}", serde_json::to_string(&geojson).unwrap());
        println!("{:?}", heights);

        Ok(geojson)
    } else {
        println!("No path found");
        Err(anyhow::Error::msg("No path found"))
    }

    // Convert points_checked to geojson
    // let p = GeoJsonValue::MultiPoint(points_checked.iter().map(|&node_index| {
    //     let node = &graph.graph[node_index];
    //     vec![node.lon, node.lat]
    // }).collect());
    // let feature = Feature {
    //     bbox: None,
    //     geometry: Some(Geometry::new(p)),
    //     id: None,
    //     properties: None,
    //     foreign_members: None,
    // };
    //
    // let geojson = GeoJson::Feature(feature);
    // println!("{}", serde_json::to_string(&geojson).unwrap());
}

fn find_nearest_nodes(graph: &Graph, point: &Point) -> Vec<NodeIndex> {
    let pgraph = &graph.graph;
    let closest = graph.location_index.snap_closest(point);

    let endpoints = pgraph.edge_endpoints(EdgeIndex::new(closest.edge_id)).unwrap();
    vec![endpoints.0, endpoints.1]
}
