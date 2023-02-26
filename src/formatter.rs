use crate::trips_arena::TripsArena;
use crate::{Gtfs1, gtfs_setup, InProgressTrip, NULL_ID, RoadStructure, TimeToReachRTree, WALKING_SPEED};
use rstar::PointDistance;
use std::fmt::{Display, Formatter};
use crate::time::Time;

pub struct InProgressTripsFormatter<'a, 'b> {
    trips: Vec<&'a InProgressTrip>,
    gtfs: &'b Gtfs1,
}

struct TimeFormatter {
    secs: Time,
}

impl Display for TimeFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let hours = self.secs.as_u32() / 3600;
        let minutes = (self.secs.as_u32() % 3600) / 60;

        f.write_fmt(format_args!("{:02}:{:02}", hours, minutes))
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
        println!("Trip {:?}", trip);
        println!("Route id {:?}", trip.current_route);
        let bus_number = &gtfs.routes[&trip.current_route.route_id].short_name;
        let stop_name = &gtfs.stops[&trip.boarding_stop_id].name;
        fmt.write_fmt(format_args!(
            "Boarding {} at {}, {}\n",
            bus_number,
            stop_name,
            TimeFormatter {
                secs: trip.boarding_time
            }
        ))
    }

    fn format_in_progress_trip_disembark(
        gtfs: &Gtfs1,
        trip: &InProgressTrip,
        fmt: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        // For disembarking part
        let bus_number = &gtfs.routes[&trip.current_route.route_id].short_name;
        let stop_name = &gtfs.stops[&trip.get_off_stop_id].name;
        fmt.write_fmt(format_args!(
            "Get off {} at {}, {}\n",
            bus_number,
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
        Ok(())
    }
}

pub fn time_to_point(
    data: &RoadStructure,
    arena: &TripsArena,
    gtfs: &Gtfs1,
    point: [f64; 2],
    is_lat_lng: bool,
) {
    let point = if is_lat_lng {
        crate::projection::project_lng_lat(point[1], point[0])
    } else {
        crate::projection::project_lng_lat(point[0], point[1])
    };

    let (best_time, obs) = data.nearest_times_to_point(&point).map(|obs| {
        let time_to_reach =
            obs.data.timestamp + obs.distance_2(&point).sqrt() / WALKING_SPEED;
        (time_to_reach, obs)
    })
        .min_by_key(|(time, obs)| {
            // Penalize time for every transfer performed
            *time + obs.data.transfers as f64 * 45.0
        })
        .unwrap();

    println!(
        "Best Time to reach is {:02.0}:{:02.0}",
        (best_time.0 / 3600.0).round(),
        ((best_time.0 % 3600.0) / 60.0).round()
    );

    println!(
        "{}",
        InProgressTripsFormatter {
            trips: gtfs_setup::get_trip_transfers(arena, obs.data.progress_trip_id.unwrap()),
            gtfs
        }
    );
}
