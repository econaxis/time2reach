use crate::projection::project_lng_lat;
use crate::time::Time;
use crate::web::LatLng;
use crate::{projection, BusPickupInfo, NULL_ID};
use gtfs_structure_2::gtfs_wrapper::{Gtfs1, StopTime, Trip};
use gtfs_structure_2::IdType;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use rustc_hash::FxHashMap;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

#[derive(Default, Debug)]
pub struct RoutePickupTimes(pub FxHashMap<RouteStopSequence, BTreeSet<BusPickupInfo>>);

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct RouteStopSequence {
    pub route_id: IdType,
    pub shape_id: Option<IdType>,
    pub headsign_hash: u64,
    pub direction: bool,
}

impl Default for RouteStopSequence {
    fn default() -> Self {
        Self {
            shape_id: None,
            route_id: NULL_ID,
            headsign_hash: 0,
            direction: false,
        }
    }
}

impl RoutePickupTimes {
    fn add_route_pickup_time(&mut self, trip: &Trip, stop_time: &StopTime) {
        let route_stop_sequence = RouteStopSequence {
            route_id: trip.route_id,
            shape_id: trip.shape_id,
            headsign_hash: trip
                .trip_headsign
                .as_ref()
                .map(|x| {
                    let mut s = DefaultHasher::new();
                    x.hash(&mut s);
                    s.finish()
                })
                .unwrap_or(0),
            direction: crate::direction_to_bool(&trip.direction_id.unwrap()),
        };

        let bus_pickup = BusPickupInfo {
            timestamp: Time(stop_time.arrival_time.unwrap() as f64),
            stop_sequence_no: stop_time.stop_sequence,
            trip_id: trip.id,
        };
        if let Some(times) = self.0.get_mut(&route_stop_sequence) {
            times.insert(bus_pickup);
        } else {
            self.0
                .insert(route_stop_sequence, BTreeSet::from([bus_pickup]));
        }
    }
}

#[derive(Default)]
pub struct StopsWithTrips(pub FxHashMap<IdType, RoutePickupTimes>);

#[derive(Debug)]
pub struct StopsData {
    pub trips_with_time: RoutePickupTimes,
    pub stop_id: IdType,
}

#[derive(Debug)]
pub struct SpatialStopsWithTrips(pub RTree<GeomWithData<[f64; 2], StopsData>>);

impl SpatialStopsWithTrips {
    pub fn is_near_point(&self, point: LatLng) -> bool {
        let xy = project_lng_lat(point.longitude, point.latitude);
        self.0
            .locate_within_distance(xy, 1000.0 * 1000.0)
            .next()
            .is_some()
    }
}

impl StopsWithTrips {
    pub fn add_stop(&mut self, stop_time: &StopTime, trip: &Trip) {
        if let Some(trips) = self.0.get_mut(&stop_time.stop_id) {
            trips.add_route_pickup_time(trip, stop_time);
        } else {
            let mut rp = RoutePickupTimes::default();
            rp.add_route_pickup_time(trip, stop_time);
            self.0.insert(stop_time.stop_id, rp);
        }
    }
    pub fn into_spatial(self, gtfs: &Gtfs1) -> SpatialStopsWithTrips {
        let mut points_data = Vec::new();

        for (stop_id, trips_with_time) in self.0 {
            let stop = &gtfs.stops[&stop_id];
            let stop_coords = projection::project_stop(stop);

            let stops_data = StopsData {
                trips_with_time,
                stop_id,
            };
            points_data.push(GeomWithData::new(stop_coords, stops_data));
        }

        SpatialStopsWithTrips(RTree::bulk_load(points_data))
    }
}
