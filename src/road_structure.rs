use crate::{project_lng_lat, PROJSTRING, ReachData, WALKING_SPEED};
use gdal::vector::{FieldDefn, Layer, LayerAccess};
use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
use geo_types::LineString;
use proj::Proj;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use std::collections::{HashMap, VecDeque};


use gdal::vector::OGRFieldType;
use gdal::vector::sql::Dialect;
use crate::time::Time;

type EdgeId = u64;

#[derive(Clone)]
struct EdgeData {
    from_node: NodeId,
    to_node: NodeId,
    length: f64,
}

impl EdgeData {
    fn get_other_node(&self, this_node: NodeId) -> NodeId {
        if self.from_node == this_node {
            self.to_node
        } else if self.to_node == this_node {
            self.from_node
        } else {
            panic!("this_node not connected to current edge");
        }
    }
}

struct NodeEdges(
    // Terminal([EdgeId; 1]),
    // Straight([EdgeId; 2]),
    Vec<EdgeId>,
);

struct NodeEdgesIteratorMut<'a> {
    n: &'a NodeEdges,
    idx: usize,
}

impl<'a> Iterator for NodeEdgesIteratorMut<'a> {
    type Item = &'a EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.idx < self.n.0.len() {
            Some(&self.n.0[self.idx])
        } else {
            None
        };
        self.idx += 1;
        result
    }
}

impl Default for NodeEdges {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl NodeEdges {
    fn iter(&self) -> NodeEdgesIteratorMut<'_> {
        NodeEdgesIteratorMut { n: self, idx: 0 }
    }
    fn add_edge(&mut self, edge: EdgeId) {
        self.0.push(edge);
    }
}

type NodeId = u64;

struct RoadStructureInner {
    dataset: Dataset,
    edges_rtree: RTree<GeomWithData<rstar::primitives::Line<[f64; 2]>, EdgeId>>,
    nodes: HashMap<NodeId, NodeEdges>,
    edges: HashMap<EdgeId, EdgeData>,
}

pub type NodeBestTimes = HashMap<NodeId, Time>;
pub struct RoadStructure {
    rs: RoadStructureInner,
    nb: NodeBestTimes,
}


fn is_first_reacher(rs: &RoadStructureInner, nb: &NodeBestTimes, point: &[f64; 2], time: Time) -> bool {
    let edge = &rs.edges[&rs.nearest_edge_to_point(point)];

    for nodeid in [edge.to_node, edge.from_node] {
        if nb.get(&nodeid).unwrap_or(&Time::MAX) <= &time {
            return false;
        }
    }
    true
}
impl RoadStructure {
    pub fn is_first_reacher(&self, point: &[f64; 2], time: Time) -> bool {
        is_first_reacher(&self.rs, &self.nb, point, time)
    }

    pub fn add_observation(&mut self, point: &[f64; 2], data: ReachData) {
        self.rs
            .explore_from_point(point, data.timestamp, &mut self.nb);
    }
    pub fn new() -> Self {
        Self {
            rs: RoadStructureInner::new(),
            nb: NodeBestTimes::new(),
        }
    }

    pub fn save(&self, null_value: u32) {
        self.rs.calculate_best_times(&self.nb, null_value);
    }
}

#[test]
fn one() {
    let mut r = RoadStructureInner::new();
    let nb = r.test();
    r.calculate_best_times(&nb);
    // r.test();
}

fn geo_line_to_rstar_line(l: geo_types::Line) -> rstar::primitives::Line<[f64; 2]> {
    rstar::primitives::Line::new(l.start.into(), l.end.into())
}

fn feature_fids(feat: &mut Layer) -> Vec<u64> {
    let feature_fids: Vec<u64> = feat.features().map(|f| f.fid().unwrap()).collect();
    feature_fids
}
impl RoadStructureInner {
    fn all_edges_from_node(&self, id: NodeId) -> NodeEdgesIteratorMut<'_> {
        self.nodes[&id].iter()
    }

    fn set_best_time(node_best_times: &mut NodeBestTimes, node: NodeId, time: Time) -> bool {
        let _l = node_best_times.len();
        match node_best_times.get_mut(&node) {
            Some(x) if *x > time => {
                // Only return true if the difference is sufficient enough
                if *x - time > Time(60.0) {
                    *x = time;
                    true
                } else {
                    *x = time;
                    false
                }
            }
            None => {
                node_best_times.insert(node, time);
                true
            }
            _ => false,
        }
    }
    fn explore_from_node(
        &self,
        node: NodeId,
        base_time: Time,
        to_explore: &mut VecDeque<(NodeId, Time)>,
        node_best_times: &mut NodeBestTimes,
    ) {
        for edge_ in self.all_edges_from_node(node) {
            let edge = &self.edges[edge_];

            let other_node = edge.get_other_node(node);
            let time_to_other_node = base_time + edge.length / WALKING_SPEED;
            if Self::set_best_time(node_best_times, other_node, time_to_other_node) {
                // This node has it's best time beat.
                to_explore.push_back((other_node, time_to_other_node));
            }
        }
    }

    fn nearest_edge_to_point(&self, point: &[f64; 2]) -> EdgeId {
        let starting_edge_geom = self.edges_rtree.nearest_neighbor(point).unwrap();
        starting_edge_geom.data
    }
    pub fn explore_from_point(
        &self,
        point: &[f64; 2],
        base_time: Time,
        node_best_times: &mut NodeBestTimes,
    ) {
        // Explore all reachable roads from a particular point
        let starting_edge = &self.edges[&self.nearest_edge_to_point(point)];

        let mut queue = VecDeque::new();
        // TODO: assuming that time to edge == time to the node.
        // TODO: fix this, base_time + time to the edge
        self.explore_from_node(
            starting_edge.from_node,
            base_time,
            &mut queue,
            node_best_times,
        );
        self.explore_from_node(
            starting_edge.to_node,
            base_time,
            &mut queue,
            node_best_times,
        );

        while let Some((item, set_time)) = queue.pop_back() {
            let time = node_best_times[&item];
            if time != set_time {
                assert!(time < set_time);
                continue;
            }

            if time - base_time >= Time(3600.0 * 0.75) {
                continue;
            }

            self.explore_from_node(item, time, &mut queue, node_best_times);
        }
    }

    pub fn new() -> Self {
        let options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = gdal::Dataset::open_ex("web/public/toronto2.gpkg", options).unwrap();


        let mut s = Self {
            dataset,
            edges_rtree: Default::default(),
            nodes: Default::default(),
            edges: Default::default(),
        };

        let mut edges_layer = s.dataset.layer_by_name("edges").unwrap();
        let mut nodes_layer = s.dataset.layer_by_name("nodes").unwrap();

        let field = FieldDefn::new("test_field1", OGRFieldType::OFTInteger).unwrap();
        field.add_to_layer(&edges_layer);
        s.dataset.execute_sql("UPDATE edges SET test_field1=-1", None, Dialect::SQLITE).unwrap();

        let spatialref = edges_layer.spatial_ref().unwrap();

        let proj = Proj::new_known_crs(&spatialref.to_proj4().unwrap(), PROJSTRING, None).unwrap();

        for feature in nodes_layer.features() {
            s.nodes.insert(
                feature
                    .field("osmid")
                    .unwrap()
                    .unwrap()
                    .into_int64()
                    .unwrap() as u64,
                NodeEdges::default(),
            );
        }



        for feature in edges_layer.features() {
            let geo = feature.geometry().unwrap().to_geo().unwrap();
            let from_node = feature
                .field("from")
                .unwrap()
                .unwrap()
                .into_int64()
                .unwrap() as u64;
            let to_node = feature.field("to").unwrap().unwrap().into_int64().unwrap() as u64;
            let id = feature.fid().unwrap();
            let length = feature
                .field("length")
                .unwrap()
                .unwrap()
                .into_real()
                .unwrap();

            let edge_data = EdgeData {
                from_node,
                to_node,
                length,
            };
            s.edges.insert(id, edge_data);

            s.nodes.get_mut(&from_node).unwrap().add_edge(id);
            s.nodes.get_mut(&to_node).unwrap().add_edge(id);
            let line_string: LineString = geo.try_into().unwrap();
            for line in line_string.lines() {
                let start = proj.project(line.start, false).unwrap();
                let end = proj.project(line.end, false).unwrap();
                let line1 = rstar::primitives::Line::new(start.into(), end.into());
                s.edges_rtree.insert(GeomWithData::new(line1, id));
            }
        }
        // dbg!(max_len.sqrt());
        s.dataset.flush_cache();
        // dbg!(spatialref.to_proj4().unwrap());
        // dbg!(proj.project((-79.5799408_f64, 43.6233548_f64), false).unwrap());
        // dbg!(project_lng_lat(-79.5799408, 43.6233548));

        s
    }
    pub fn calculate_best_times(&self, b: &NodeBestTimes, null_value: u32) {
        let mut edges = self.dataset.layer_by_name("edges").unwrap();
        let mut max_time = Time(0.0);
        for feature_fid in feature_fids(&mut edges) {
            let feature = edges.feature(feature_fid).unwrap();

            let from_node = feature
                .field("from")
                .unwrap()
                .unwrap()
                .into_int64()
                .unwrap() as u64;
            let to_node = feature.field("to").unwrap().unwrap().into_int64().unwrap() as u64;
            let _length = feature
                .field("length")
                .unwrap()
                .unwrap()
                .into_real()
                .unwrap();

            let from_time = b.get(&from_node).copied();
            let to_time = b.get(&to_node).copied();

            if from_time.and(to_time).is_some() {
                let average_time = (from_time.unwrap() + to_time.unwrap()) / 2.0;
                max_time = max_time.max(average_time);
                feature
                    .set_field_integer("test_field1", average_time.as_u32() as i32)
                    .unwrap();
                edges.set_feature(feature);
            }
            // } else {
            //     feature
            //         .set_field_integer("test_field1", null_value as i32)
            //         .unwrap();
            // }
        }
        self.dataset.execute_sql(format!("UPDATE edges SET test_field1=NULL WHERE test_field1=-1"), None, Dialect::SQLITE).unwrap();
        // edges
    }
    pub fn test(&mut self) -> NodeBestTimes {
        let point = [43.7762291, -79.4889104];
        let point = project_lng_lat(point[1], point[0]);

        let mut nb = HashMap::new();
        self.explore_from_point(&point, Time(0.0), &mut nb);
        nb
    }
}
