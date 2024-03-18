use crate::{AGraph, Node, Point};
use parking_lot::Mutex;
use rusqlite::Connection;
use petgraph::visit::{IntoEdges, IntoNeighbors, IntoNodeReferences};

pub struct LocationIndex {
    points: Vec<PointSnap>,
}

impl LocationIndex {
    pub fn new(graph: &AGraph, conn: &Mutex<Connection>) -> LocationIndex {
        let points = graph.node_references().filter_map(|(node_index, node_weight)| {
            let has_edges = graph.neighbors(node_index).next().is_some();
            if has_edges {
                Some(PointSnap { point: node_weight.point(), point_id: node_index.index() })
            } else {
                None
            }
        }).collect();
        LocationIndex {
            points,
        }
    }
    pub(crate) fn snap_closest<'a>(&'a self, point: &Point) -> PointSnapResult<'a> {
        // TODO: Use a spatial index to speed this up
        let mut best = None;
        let mut best_dist = std::f32::MAX;
        for point_snap in &self.points {
            let dist =  point_snap.point.haversine_distance(point);
            if dist < best_dist {
                best_dist = dist;
                best = Some(point_snap);
            }
        }
        PointSnapResult {
            point_snap: best.unwrap(),
            dist: best_dist,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PointSnap {
    pub point: Point,
    pub(crate) point_id: usize,
}

#[derive(Debug)]
pub struct PointSnapResult<'a> {
    pub point_snap: &'a PointSnap,
    pub dist: f32,
}
