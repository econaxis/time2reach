use std::borrow::Cow;
use geojson::{FeatureCollection, Value as GeoJsonValue};
use crate::rate_bicycle_friendliness;
use geojson::{Feature, GeoJson, Geometry};
use petgraph::algo::astar;
use petgraph::graph::{EdgeIndex, EdgeReference, NodeIndex, UnGraph};
use petgraph::prelude::EdgeRef;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use petgraph::visit::IntoEdgeReferences;
use crate::bicycle_rating::filter_by_tag;

pub type AGraph = UnGraph<Node, Edge>;

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
    fn eq(&self, b: &Point) -> bool {
        self.haversine_distance(b) <= 0.001
    }
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

#[derive(Clone)]
struct PointSnap {
    pub point: Point,
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

const DO_NOT_EXPLORE: f64 = std::f64::MAX;

fn real_edge_weight<'a>(graph: &'a Graph, edgeref: EdgeReference<'a, Edge>) -> f64 {
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

    let source = graph.graph.node_weight(edgeref.source()).unwrap();
    let target = graph.graph.node_weight(edgeref.target()).unwrap();
    let elevation_diff = target.ele - source.ele;
    let slope = elevation_diff / edge.dist;

    let elevation_penalty = (slope * 10.0).powf(2.0).abs() * elevation_diff.signum() * edge.dist;
    let elevation_penalty = if elevation_penalty.is_nan() { 0.0 } else { elevation_penalty };

    (edge.dist + elevation_penalty * 1.0).max(edge.dist * 0.85) * bicycle_penalty
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
            },
            _ => {}
        }
        return true;
    }).collect::<Vec<Edge>>();

    (changed, new_edges)
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

fn render_route(graph: &Graph, mut nodes: Cow<[NodeIndex]>, start_snap: PointSnap, end_snap: PointSnap) -> GeoJson {
    let align_snap_endpoints = |nodes: &[NodeIndex], edge_id: usize, is_end: bool| -> (NodeIndex, NodeIndex) {
        let edge_endpoints = graph.graph.edge_endpoints(EdgeIndex::new(edge_id)).unwrap();

        let index = if is_end {
            nodes.len() - 1
        } else {
            0
        };

        if edge_endpoints.0 == nodes[index] {
            (edge_endpoints.0, edge_endpoints.1)
        } else if edge_endpoints.1 == nodes[index] {
            (edge_endpoints.1, edge_endpoints.0)
        } else {
            panic!("Edge does not contain start node")
        }
    };
    // For each node in the path, find the edge that connects it to the next node
    // Then add all the pointlist in the edge
    type Position = Vec<f64>;


    let start_snap_edge_endpoints = align_snap_endpoints(&nodes, start_snap.edge_id, false);
    let end_snap_edge_endpoints = align_snap_endpoints(&nodes, end_snap.edge_id, true);
    
    if start_snap_edge_endpoints.1 != nodes[1] {
        // We have to start from the other end of the edge to "cross" the start_snap node
        nodes.to_mut().insert(0, start_snap_edge_endpoints.1);
    }

    if end_snap_edge_endpoints.1 != nodes[nodes.len() - 2] {
        // We have to end all the way the other end of the edge to "cross" the start_snap node
        nodes.to_mut().push(end_snap_edge_endpoints.1);
    }
    println!("End snap edge endpoints {:?} {:?}", end_snap_edge_endpoints, nodes);

    fn comp_positions(a: &Position, b: &Position) -> bool {
        (a[0] - b[0]).abs() < 0.001 &&  (a[1] - b[1]).abs() < 0.001
    }

    let mut last_end: Option<Position> = None;

    let mut has_started = false;
    let mut has_ended = false;

    let coordinates: Vec<Position> = nodes.windows(2).flat_map(|indices| {
        let prev = indices[0];
        let prevnode = &graph.graph[prev];
        let cur = indices[1];

        println!("Processing edges {:?} {:?}", prev, cur);

        let edge = graph.graph.find_edge(prev, cur).unwrap();
        let edge = &graph.graph[edge];

        let points_iterator: Box<dyn Iterator<Item=&Point>> = if edge.source == prevnode.id {
            Box::new(edge.points.iter())
        } else {
            Box::new(edge.points.iter().rev())
        };

        let mut pointlist = points_iterator.filter_map(|point| {
            if !has_started && start_snap.point.eq(point) {
                has_started = true;
            }

            if has_started && !has_ended && end_snap.point.eq(point) {
                has_ended = true;
            }
            if has_started && !has_ended {
                Some(vec![point.lon, point.lat])
            } else {
                None
            }
        }).collect::<Vec<Position>>();

        if !pointlist.is_empty() {
            let last_pointlist = pointlist.last().unwrap().clone();

            if last_end.as_ref().map(|end| comp_positions(&end, &last_pointlist)).unwrap_or(false) {
                pointlist.truncate(pointlist.len() - 1);
            };

            last_end = Some(last_pointlist);
        }
        pointlist
    }).collect();

    assert!(has_started);
    assert!(has_ended);

    let line_string = GeoJsonValue::LineString(coordinates);
    let geometry = Geometry::new(line_string);

    let feature = Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: None,
        foreign_members: None,
    };

    GeoJson::Feature(feature)
}
pub fn route(graph: &Graph, start: Point, end: Point) -> anyhow::Result<GeoJson> {
    // Find the nearest nodes in the graph to the specified coordinates
    let (start_snap, start_nodes) = find_nearest_nodes(&graph, &start);
    let (end_snap, end_nodes) = find_nearest_nodes(&graph, &end);

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

        let geojson = render_route(&graph, Cow::from(&path), start_snap, end_snap);

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

fn find_nearest_nodes(graph: &Graph, point: &Point) -> (PointSnap, Vec<NodeIndex>) {
    let pgraph = &graph.graph;
    let closest = graph.location_index.snap_closest(point).clone();

    let endpoints = pgraph.edge_endpoints(EdgeIndex::new(closest.edge_id)).unwrap();

    // Check
    let edge = pgraph.find_edge(endpoints.0, endpoints.1).unwrap();
    let edge = &pgraph[edge];
    assert!(edge.points.iter().find(|x| closest.point.eq(x)).is_some());

    println!("Closest edge: {:?} (E{}) {:?}", endpoints, edge.id, edge.points);
    (closest, vec![endpoints.0, endpoints.1])
}
