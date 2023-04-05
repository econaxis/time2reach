use std::collections::{BTreeSet, HashMap};
use rstar::RTree;
use rstar::primitives::GeomWithData;
use crate::{BusPickupInfo, IdType, NULL_ID, projection};
use crate::gtfs_wrapper::{Gtfs1, StopTime, Trip};
use crate::time::Time;

#[derive(Default, Debug)]
pub struct RoutePickupTimes(pub HashMap<RouteStopSequence, BTreeSet<BusPickupInfo>>);

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct RouteStopSequence {
    pub route_id: IdType,
    pub direction: bool,
}

impl Default for RouteStopSequence {
    fn default() -> Self {
        Self {
            route_id: NULL_ID,
            direction: false,
        }
    }
}

impl RoutePickupTimes {
    fn add_route_pickup_time(&mut self, trip: &Trip, stop_time: &StopTime) {
        let route_stop_sequence = RouteStopSequence {
            route_id: trip.route_id,
            direction: crate::direction_to_bool(&trip.direction_id.unwrap()),
        };

        let bus_pickup = BusPickupInfo {
            timestamp: Time(stop_time.arrival_time.unwrap() as f64),
            stop_sequence_no: stop_time.stop_sequence as u16,
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
pub struct StopsWithTrips(pub HashMap<IdType, RoutePickupTimes>);

#[derive(Debug)]
pub struct StopsData {
    pub trips_with_time: RoutePickupTimes,
    pub stop_id: IdType
}

#[derive(Debug)]
pub struct SpatialStopsWithTrips(pub RTree<GeomWithData<[f64; 2], StopsData>>);

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
    pub fn to_spatial(self, gtfs: &Gtfs1) -> SpatialStopsWithTrips {
        let mut points_data = Vec::new();

        for (stop_id, trips_with_time) in self.0 {
            let stop = &gtfs.stops[&stop_id];
            let stop_coords = projection::project_stop(stop);

            let stops_data = StopsData { trips_with_time, stop_id };
            points_data.push(GeomWithData::new(stop_coords, stops_data));
        }

        SpatialStopsWithTrips(RTree::bulk_load(points_data))
    }
}
