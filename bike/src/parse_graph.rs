use std::borrow::Cow;
use std::cell::Cell;
use std::f64::INFINITY;
use std::ops::AddAssign;
use crate::calories;
use geojson::Value as GeoJsonValue;
use crate::real_edge_weight;
use geojson::{Feature, GeoJson, Geometry};
use petgraph::algo::astar;
use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::Serialize;
use petgraph::prelude::EdgeRef;
use crate::graph::{Graph, Point};
use crate::location_index::PointSnap;
use crate::real_edge_weight::RouteOptions;

pub fn main() {
    let graph = Graph::new();

    // let gj = point_list_geojson(&graph.graph);
    // println!("{}", serde_json::to_string(&gj).unwrap());

    // Replace these coordinates with your actual coordinates
    let start = Point { lat: 37.7791612, lon: -122.4351754, ele: 51.0 };
    let end = Point { lat: 37.758454, lon: -122.446772, ele: 149.0 };

    let result = route(&graph, start, end, RouteOptions::default());
    println!("{:?}", result);
}

#[derive(Serialize, Debug, Default)]
pub struct Energy {
    pub calories: f64,
    pub uphill_meters: f64,
    pub downhill_meters: f64,
    pub total_meters: f64,
}

impl AddAssign for Energy {
    fn add_assign(&mut self, rhs: Self) {
        self.calories += rhs.calories;
        self.uphill_meters += rhs.uphill_meters;
        self.downhill_meters += rhs.downhill_meters;
        self.total_meters += rhs.total_meters;
    }
}

#[derive(Serialize, Debug)]
pub struct RouteResponse {
    pub route: GeoJson,
    pub route_metadata: Vec<(f32, f32, u8)>,
    pub elevation_index: Vec<usize>,
    pub energy: Option<Energy>,
}

impl RouteResponse {
    fn with_energy(mut self, energy: Energy) -> Self {
        self.energy = Some(energy);
        self
    }
}

type Position = Vec<f64>;

fn comp_positions(a: &Position, b: &Position) -> bool {
    (a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001
}

fn render_route(graph: &Graph, mut nodes: Cow<[NodeIndex]>, start_snap: PointSnap, end_snap: PointSnap) -> RouteResponse {
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
    // let start_snap_edge_endpoints = align_snap_endpoints(&nodes, start_snap.edge_id, false);
    // let end_snap_edge_endpoints = align_snap_endpoints(&nodes, end_snap.edge_id, true);

    // if start_snap_edge_endpoints.1 != nodes[1] {
    //     // We have to start from the other end of the edge to "cross" the start_snap node
    //     nodes.to_mut().insert(0, start_snap_edge_endpoints.1);
    // }
    //
    // if end_snap_edge_endpoints.1 != nodes[nodes.len() - 2] {
    //     // We have to end all the way the other end of the edge to "cross" the start_snap node
    //     nodes.to_mut().push(end_snap_edge_endpoints.1);
    // }


    let mut last_end: Option<Point> = None;

    let mut has_started = false;
    let mut has_ended = false;

    let mut route_metadata = Vec::new();

    // Maps each linestring point to an elevation index
    // Since we're rendering curved routes, one node has multiple linestring points
    let mut elevation_index_map: Vec<usize> = Vec::new();

    let mut cumdist = 0.0;
    let coordinates: Vec<Position> = nodes.windows(2).flat_map(|indices| {
        let prev = indices[0];
        let prevnode = &graph.graph[prev];
        let cur = indices[1];

        let edge = graph.graph.find_edge(prev, cur).unwrap();
        let edge = &graph.graph[edge];

        let edge_points = edge.points(&graph.db).to_vec();
        let points_iterator: Box<dyn Iterator<Item=&Point>> = if edge.source == prevnode.id {
            Box::new(edge_points.iter())
        } else {
            Box::new(edge_points.iter().rev())
        };

        let mut pointlist = points_iterator.filter_map(|point| {
            if !has_started && start_snap.point.eq(point) {
                has_started = true;
            }

            if has_started && !has_ended && end_snap.point.eq(point) {
                has_ended = true;
            }
            if has_started && !has_ended {
                Some(point.clone())
            } else {
                None
            }
        }).collect::<Vec<Point>>();

        if !pointlist.is_empty() {
            let last_pointlist = pointlist.last().unwrap().clone();

            if last_end.as_ref().map(|end| end.haversine_distance(&last_pointlist) <= 0.001).unwrap_or(false) {
                pointlist.truncate(pointlist.len() - 1);
            };

            last_end = Some(last_pointlist);
        }

        let mut prev: Option<Point> = None;
        let mut current_cumdist = cumdist;
        pointlist.iter().for_each(|x| {
            elevation_index_map.push(route_metadata.len());

            if let Some(prev) = &prev {
                current_cumdist += prev.haversine_distance_ele(&x);
            }
            prev = Some(x.clone());

            let bike_friendly = edge.bike_friendly;

            route_metadata.push((current_cumdist, x.ele, bike_friendly));
        });

        if !pointlist.is_empty() {
            // assert!((current_cumdist - cumdist + edge.dist).abs() < 0.0001, "Distance mismatch: {} {}", current_cumdist - cumdist, edge.dist);
            // cumdist += edge.dist;
            cumdist = current_cumdist;
        }

        pointlist.into_iter().map(|point| vec![point.lon, point.lat])
    }).collect();

    if nodes.len() >= 2 {
        let last = nodes[nodes.len() - 1];
        let secondlast = nodes[nodes.len() - 2];
        let node = &graph.graph[last];
        let edge = graph.graph.find_edge(secondlast, last).unwrap();
        let edge = &graph.graph[edge];
        route_metadata.push((cumdist, node.ele, edge.bike_friendly));
    }

    assert!(has_started);
    assert!(has_ended);
    assert_eq!(coordinates.len(), elevation_index_map.len());
    // assert!(*elevation_index_map.last().unwrap() == &elevations.len() - 1);

    let line_string = GeoJsonValue::LineString(coordinates);

    let geometry = Geometry::new(line_string);

    let feature = Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: None,
        foreign_members: None,
    };

    RouteResponse {
        route: GeoJson::Feature(feature),
        route_metadata,
        elevation_index: elevation_index_map,
        energy: None
    }
}

fn get_finish_condition<'a>(graph: &'a Graph, end_nodes: &'a [NodeIndex]) -> impl Fn(NodeIndex) -> bool + 'a {
    |node: NodeIndex| {
        let node_pos = graph.graph[node].point();
        end_nodes.iter().any(|en| {
            if node == *en {
                true
            } else {
                false
                // let end_pos = graph.graph[*en].point();
                //
                // // Also allow anything within 50 meters of any end nodes
                // node_pos.haversine_distance(&end_pos) < 50.0
            }
        })
    }
}

pub fn route(graph: &Graph, start: Point, end: Point, options: RouteOptions) -> anyhow::Result<RouteResponse> {
    // Find the nearest nodes in the graph to the specified coordinates
    let (start_snap, start_nodes) = find_nearest_nodes(&graph, &start);
    let (end_snap, end_nodes) = find_nearest_nodes(&graph, &end);

    for start_node in &start_nodes {
        let mut points_checked = Vec::new();
        let mut edges_checked = Vec::new();

        let mut access_friendly_limit = Cell::new(5);
        let finish_condition = get_finish_condition(&graph, &end_nodes);
        let result = if let Some((cost, path)) = astar(
            &graph.graph,
            start_node.clone(),
            |x| {
                finish_condition(x)
            },
            |e| {
                edges_checked.push(e);
                real_edge_weight::real_edge_weight(&graph.graph, e, &options)
            }, // Cost function, using edge distance
            |node: NodeIndex| {
                points_checked.push(node);

                let node = &graph.graph[node];
                let point = node.point();

                let cost = end.haversine_distance(&point);
                // Might overestimate...what's the impact?
                cost * 1.0
            },
        ) {
            if path.len() < 2 {
                println!("Path found but is too short!");
                return Err(anyhow::Error::msg("No path found"));
            }

            println!("Found path with cost {}", cost);
            let response = render_route(&graph, Cow::from(&path), start_snap.clone(), end_snap.clone());
            let energy = calories::calculate_energy(&path, &graph.graph);


            Ok(response.with_energy(energy))
        } else {
            println!("No path found");
            let p = GeoJsonValue::MultiPoint(points_checked.iter().take(1000).map(|&node_index| {
                let node = &graph.graph[node_index];
                vec![(node.lon * 100000.0).round() / 100000.0, (node.lat * 100000.0).round() / 100000.0]
            }).collect());
            let feature = Feature {
                bbox: None,
                geometry: Some(Geometry::new(p)),
                id: None,
                properties: None,
                foreign_members: None,
            };

            let geojson = GeoJson::Feature(feature);
            println!("GEOJSON {}", serde_json::to_string(&geojson).unwrap());


            let mut line_strings: Vec<Vec<Vec<f64>>> = Vec::new();

            for edge in &edges_checked {
                let start_node = edge.source();
                let end_node = edge.target();
                let start_point = &graph.graph[start_node];
                let end_point = &graph.graph[end_node];
                let start_coords = vec![
                    (start_point.lon * 100000.0).round() / 100000.0,
                    (start_point.lat * 100000.0).round() / 100000.0,
                ];
                let end_coords = vec![
                    (end_point.lon * 100000.0).round() / 100000.0,
                    (end_point.lat * 100000.0).round() / 100000.0,
                ];
                line_strings.push(vec![start_coords, end_coords]);
            }

            let multi_line_string = geojson::Value::MultiLineString(line_strings);
            let feature = geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::new(multi_line_string)),
                id: None,
                properties: None,
                foreign_members: None,
            };

            let geojson = geojson::GeoJson::Feature(feature);
            println!("GEOJSON1 {}", serde_json::to_string(&geojson).unwrap());
            println!("Edges: {:?}", edges_checked);


            Err(anyhow::Error::msg("No path found"))
        };

        if result.is_ok() {
            return result;
        }
    }

    return Err(anyhow::Error::msg("No path found"));
}

fn find_nearest_nodes(graph: &Graph, point: &Point) -> (PointSnap, Vec<NodeIndex>) {
    let pgraph = &graph.graph;
    let closest = graph.location_index.snap_closest(point);

    let node = &pgraph[NodeIndex::new(closest.point_snap.point_id)];

    println!("Closest node: {:?} {:?}", closest, node);
    (closest.point_snap.clone(), vec![NodeIndex::new(closest.point_snap.point_id)])
}
