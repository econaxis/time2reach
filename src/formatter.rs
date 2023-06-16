use crate::gtfs_wrapper::RouteType;
use crate::in_progress_trip::InProgressTrip;
use crate::shape::Shape;
use crate::time::Time;
use crate::trips_arena::TripsArena;
use crate::{gtfs_setup, Gtfs1, RoadStructure, NULL_ID, WALKING_SPEED};
use geo_types::{LineString, MultiLineString};
use rstar::PointDistance;
use std::fmt::{Display, Formatter};

pub struct InProgressTripsFormatter<'a, 'b> {
    pub(crate) trips: Vec<&'a InProgressTrip>,
    pub(crate) gtfs: &'b Gtfs1,
    pub(crate) final_walking_length: f32,
}

fn construct_shape_for_ip_trip(gtfs: &Gtfs1, trip: &InProgressTrip) -> LineString {
    let gtfs_trip = &gtfs.trips[&trip.trip_id];
    let shape = &gtfs.shapes[&gtfs_trip.shape_id.unwrap()];

    let boarding_stop_time = &gtfs_trip.stop_times[trip.boarding_stop_time_idx];
    let get_off_stop_time = &gtfs_trip.stop_times[trip.get_off_stop_time_idx];
    let start_index = boarding_stop_time.shape_index;
    let end_index = get_off_stop_time.shape_index;

    Shape::to_geo_types_interp(shape, start_index, end_index)
}

impl<'a, 'b> InProgressTripsFormatter<'a, 'b> {
    pub fn construct_shape(&self) -> MultiLineString {
        MultiLineString::new(
            self.trips
                .iter()
                .filter_map(|trip| {
                    if trip.trip_id == NULL_ID {
                        None
                    } else {
                        Some(construct_shape_for_ip_trip(self.gtfs, trip))
                    }
                })
                .collect(),
        )
    }
}

pub struct TimeFormatter {
    secs: Time,
}

impl TimeFormatter {
    fn new(s: Time) -> Self {
        TimeFormatter { secs: s }
    }
}

impl Display for TimeFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let secs = self.secs.as_u32();
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        f.write_fmt(format_args!("{:02}:{:02}:{:02}", hours, minutes, seconds))
    }
}

impl From<&str> for RouteType {
    fn from(value: &str) -> Self {
        match value {
            "bus" => RouteType::Bus,
            "tram" => RouteType::Tramway,
            "subway" => RouteType::Subway,
            "rail" => RouteType::Rail,
            _ => panic!("{}", value),
        }
    }
}

pub fn get_route_mode(gtfs: &Gtfs1, trip: &InProgressTrip) -> &'static str {
    let route_id = trip.current_route.route_id;
    let route = &gtfs.routes[&route_id];
    match route.route_type {
        RouteType::Bus => "bus",
        RouteType::Tramway => "tram",
        RouteType::Subway => "subway",
        RouteType::Rail => "rail",
        _ => "",
    }
}

impl<'a, 'b> InProgressTripsFormatter<'a, 'b> {
    pub fn format_in_progress_trip_boarding(
        gtfs: &Gtfs1,
        trip: &InProgressTrip,
        fmt: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        // For the boarding part
        // Boarding {number} at {stop name}, {time}
        // println!("Trip {:?}", trip);
        // println!("Route id {:?}", trip.current_route);
        let route_id = trip.current_route.route_id;
        let route = &gtfs.routes[&route_id];
        let route_type = match route.route_type {
            RouteType::Bus => "bus",
            RouteType::Tramway => "tram",
            RouteType::Subway | RouteType::Rail => "train",
            _ => "",
        };
        let bus_number = &route.short_name;
        let stop = &gtfs.stops[&trip.boarding_stop_id];
        fmt.write_fmt(format_args!(
            "Get on {} #{} at {}, {} {:?} {:?}\n",
            route_type,
            bus_number,
            &stop.name,
            TimeFormatter {
                secs: trip.boarding_time
            },
            stop.location_type,
            stop.parent_station,
        ))
    }

    fn format_in_progress_trip_disembark(
        gtfs: &Gtfs1,
        trip: &InProgressTrip,
        fmt: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        // For disembarking part
        let _bus_number = &gtfs.routes[&trip.current_route.route_id].short_name;
        let stop_name = &gtfs.stops[&trip.get_off_stop_id].name;
        fmt.write_fmt(format_args!(
            "Get off at {}, {}\n",
            stop_name,
            TimeFormatter {
                secs: trip.exit_time
            }
        ))
    }
}

impl<'a, 'b> Display for InProgressTripsFormatter<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for trip in self.trips.iter().rev() {
            if trip.current_route.route_id == NULL_ID {
                // Begin of trip. Skip here.
                continue;
            }
            InProgressTripsFormatter::format_in_progress_trip_boarding(self.gtfs, trip, f)?;
            InProgressTripsFormatter::format_in_progress_trip_disembark(self.gtfs, trip, f)?;
        }

        let shape = self.construct_shape();
        let geojson = geojson::Value::from(&shape);
        Display::fmt(&geojson, f)?;
        Ok(())
    }
}

pub fn time_to_point<'a, 'b>(
    data: &RoadStructure,
    arena: &'a TripsArena,
    gtfs: &'b Gtfs1,
    point: [f64; 2],
    is_lat_lng: bool,
) -> Option<InProgressTripsFormatter<'a, 'b>> {
    let point = if is_lat_lng {
        crate::projection::project_lng_lat(point[1], point[0])
    } else {
        crate::projection::project_lng_lat(point[0], point[1])
    };

    let (_best_time, obs) = data
        .nearest_times_to_point(&point)
        .map(|obs| {
            let distance = obs.distance_2(&point).sqrt();
            let time_to_reach = obs.data.timestamp + distance / WALKING_SPEED;
            (time_to_reach, obs)
        })
        .min_by_key(|(time, obs)| {
            // Penalize time for every transfer performed
            *time + obs.data.transfers as f64 * 120.0
        })?;

    Some(InProgressTripsFormatter {
        trips: gtfs_setup::get_trip_transfers(arena, obs.data.progress_trip_id.unwrap()),
        gtfs,
        final_walking_length: obs.data.walking_length as f32,
    })
}
