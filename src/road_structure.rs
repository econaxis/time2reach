use crate::{TripsArena, STRAIGHT_WALKING_SPEED, WALKING_SPEED};
use gdal::vector::LayerAccess;
use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
use geo_types::Point;
use gtfs_structure_2::IdType;
use proj::Proj;
use rstar::primitives::GeomWithData;
use rstar::{PointDistance, RTree};
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer};
use std::collections::VecDeque;

use log::info;
use std::sync::{Arc, Mutex};

use crate::agencies::City;
use crate::best_times::BestTimes;
use crate::projection::get_proj_defn;
use crate::time::Time;
use serde::ser::SerializeTuple;

use crate::reach_data::ReachData;

pub type EdgeId = u64;
const MAX_WALKING_HOURS: f64 = 0.40;

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

#[derive(Default)]
struct NodeEdges(Vec<EdgeId>);

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

impl NodeEdges {
    fn iter(&self) -> NodeEdgesIteratorMut<'_> {
        NodeEdgesIteratorMut { n: self, idx: 0 }
    }
    fn add_edge(&mut self, edge: EdgeId) {
        self.0.push(edge);
    }
}

type NodeId = u64;

pub struct RoadStructureInner {
    nodes_rtree: RTree<GeomWithData<[f64; 2], NodeId>>,
    nodes_rtree_cache: Mutex<FxHashMap<IdType, NodeId>>,
    nodes: FxHashMap<NodeId, NodeEdges>,
    edges: FxHashMap<EdgeId, EdgeData>,
    city: City,
}

pub struct RoadStructure {
    pub rs: Arc<RoadStructureInner>,
    pub nb: BestTimes<NodeId>,
    pub trips_arena: TripsArena,
}

impl RoadStructure {
    pub fn city(&self) -> &City {
        &self.rs.city
    }
    pub fn clear_data(&mut self) {
        self.nb.clear();
        self.trips_arena = TripsArena::default();
    }

    pub fn is_first_reacher_to_stop(&self, stop_id: IdType, point: &[f64; 2], time: Time) -> bool {
        let nodeid = &self.rs.nearest_node_to_point(point, Some(stop_id));

        self.nb
            .get(nodeid)
            .map(|a| a.timestamp)
            .unwrap_or(Time::MAX)
            > time
    }

    pub fn add_observation(&mut self, point: &[f64; 2], data: ReachData) {
        self.rs.explore_from_point(point, data, &mut self.nb);
    }
    pub fn new_city(city: City) -> Self {
        Self {
            rs: Arc::new(RoadStructureInner::new(city)),
            nb: BestTimes::new(),
            trips_arena: TripsArena::default(),
        }
    }

    pub fn new_from_road_structure(rs: Arc<RoadStructureInner>) -> Self {
        Self {
            rs,
            nb: BestTimes::new(),
            trips_arena: TripsArena::default(),
        }
    }

    pub fn nearest_times_to_point(
        &self,
        point: &[f64; 2],
    ) -> impl Iterator<Item = GeomWithData<[f64; 2], &ReachData>> + '_ {
        self.rs
            .n_nearest_nodes_to_point(point, 5)
            .filter_map(|geom| {
                self.nb
                    .get(&geom.data)
                    .map(|reachdata| GeomWithData::new(*geom.geom(), reachdata))
            })
    }

    pub fn save(&self) -> Vec<EdgeTime> {
        self.rs.calculate_best_times(&self.nb)
    }
}

#[derive(Debug)]
pub struct EdgeTime {
    pub edge_id: EdgeId,
    pub time: f64,
}

#[derive(Debug)]
pub struct NodeTime {
    pub node_id: NodeId,
    pub time: f64,
}

impl Serialize for NodeTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2).unwrap();
        tuple.serialize_element(&self.node_id).unwrap();
        tuple.serialize_element(&self.time).unwrap();
        tuple.end()
    }
}

impl RoadStructureInner {
    fn all_edges_from_node(&self, id: NodeId) -> NodeEdgesIteratorMut<'_> {
        self.nodes[&id].iter()
    }

    fn explore_from_node(
        &self,
        node: NodeId,
        base_time: &ReachData,
        to_explore: &mut VecDeque<(NodeId, ReachData)>,
        node_best_times: &mut BestTimes<NodeId>,
        do_edge_based_search: bool,
    ) {
        if node_best_times
            .get(&node)
            .map(|a| a.timestamp < base_time.timestamp)
            .unwrap_or(false)
        {
            return;
        }

        if do_edge_based_search {
            for edge_ in self.all_edges_from_node(node) {
                let edge = &self.edges[edge_];

                let other_node = edge.get_other_node(node);
                let time_to_other_node = base_time.with_time_and_dist(
                    base_time.timestamp + edge.length / WALKING_SPEED,
                    edge.length,
                );
                if node_best_times.set_best_time(other_node, time_to_other_node.clone()) {
                    // This node has it's best time beat.
                    to_explore.push_back((other_node, time_to_other_node));
                }
            }
        } else {
            node_best_times.set_best_time(node, base_time.clone());
        }
    }

    fn nearest_node_to_point(&self, point: &[f64; 2], cache_key: Option<IdType>) -> NodeId {
        if let Some(cache_key) = cache_key {
            let mut cache = self.nodes_rtree_cache.lock().unwrap();

            let nodeid = cache
                .entry(cache_key)
                .or_insert_with(|| self.nodes_rtree.nearest_neighbor(point).unwrap().data);
            *nodeid
        } else {
            let starting_edge_geom = self.nodes_rtree.nearest_neighbor(point).unwrap();
            starting_edge_geom.data
        }
    }

    pub fn n_nearest_nodes_to_point(
        &self,
        point: &[f64; 2],
        number: usize,
    ) -> impl Iterator<Item = &GeomWithData<[f64; 2], NodeId>> + '_ {
        self.nodes_rtree.nearest_neighbor_iter(point).take(number)
    }

    pub fn distance_nearest_nodes_to_point(
        &self,
        point: [f64; 2],
        distance_squared: f64,
    ) -> impl Iterator<Item = &GeomWithData<[f64; 2], NodeId>> + '_ {
        self.nodes_rtree
            .locate_within_distance(point, distance_squared)
    }
    pub fn explore_from_point(
        &self,
        point: &[f64; 2],
        base_time: ReachData,
        node_best_times: &mut BestTimes<NodeId>,
    ) {
        const EDGE_BASED_SEARCH: bool = true;
        const WALKING_DISTANCE: f64 = if EDGE_BASED_SEARCH { 100.0 } else { 1100.0 };
        // Explore all reachable roads from a particular point
        let mut queue = VecDeque::new();

        for closest_node in
            self.distance_nearest_nodes_to_point(*point, WALKING_DISTANCE * WALKING_DISTANCE)
        {
            let distance_to_closest_node = closest_node.distance_2(point).sqrt();
            let time_to_closest_node = distance_to_closest_node / STRAIGHT_WALKING_SPEED;

            self.explore_from_node(
                closest_node.data,
                &base_time.with_time_and_dist(
                    base_time.timestamp + time_to_closest_node,
                    distance_to_closest_node,
                ),
                &mut queue,
                node_best_times,
                // Don't do edge based search, only distance search
                EDGE_BASED_SEARCH,
            );
        }

        if EDGE_BASED_SEARCH {
            // Unused because we don't want to explore based on road positions anymore
            while let Some((item, rd)) = queue.pop_back() {
                let set_time = rd.timestamp;
                let time = node_best_times.get(&item).unwrap().timestamp;
                if time != set_time {
                    debug_assert!(time < set_time);
                    continue;
                }

                if time - base_time.timestamp >= Time(3600.0 * MAX_WALKING_HOURS) {
                    continue;
                }

                self.explore_from_node(item, &rd, &mut queue, node_best_times, EDGE_BASED_SEARCH);
            }
        }
    }

    pub fn new(city: City) -> Self {
        let options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_READONLY,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        info!("Loading {} road network...", city.get_gpkg_path());
        let dataset =
            Dataset::open_ex(format!("web/public/{}.gpkg", city.get_gpkg_path()), options).unwrap();

        let mut s = Self {
            nodes_rtree: Default::default(),
            nodes_rtree_cache: Mutex::new(FxHashMap::default()),
            nodes: Default::default(),
            edges: Default::default(),
            city,
        };

        let mut edges_layer = dataset.layer_by_name("edges").unwrap();
        let mut nodes_layer = dataset.layer_by_name("nodes").unwrap();

        let spatialref = edges_layer.spatial_ref().unwrap();

        let proj_instance = get_proj_defn(&city);

        let proj =
            Proj::new_known_crs(&spatialref.to_proj4().unwrap(), &proj_instance, None).unwrap();

        let mut nodes_rtree_vec = Vec::with_capacity(nodes_layer.feature_count() as usize);
        for feature in nodes_layer.features() {
            let osmid = feature
                .field("osmid")
                .unwrap()
                .unwrap()
                .into_int64()
                .unwrap() as u64;
            s.nodes.insert(osmid, NodeEdges::default());

            let geo = feature.geometry().unwrap().to_geo().unwrap();
            let point: Point = geo.try_into().unwrap();

            let point = proj.project(point, false).unwrap();
            nodes_rtree_vec.push(GeomWithData::new([point.x(), point.y()], osmid));
        }

        s.nodes_rtree = RTree::bulk_load(nodes_rtree_vec);

        for feature in edges_layer.features() {
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
        }
        s
    }

    pub fn calculate_best_times(&self, b: &BestTimes<NodeId>) -> Vec<EdgeTime> {
        let mut max_time = Time(0.0);
        let mut edge_times = Vec::new();

        for (edge_id, edge_data) in &self.edges {
            let from_node = edge_data.from_node;
            let to_node = edge_data.to_node;

            let from_time = b.get(&from_node);
            let to_time = b.get(&to_node);

            if from_time.and(to_time).is_some() {
                let average_time =
                    (from_time.unwrap().timestamp + to_time.unwrap().timestamp) / 2.0;
                max_time = max_time.max(average_time);
                edge_times.push(EdgeTime {
                    edge_id: *edge_id,
                    time: average_time.0,
                });
            }
        }
        edge_times
    }
}
