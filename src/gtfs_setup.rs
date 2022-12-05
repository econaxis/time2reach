use id_arena::Id;
use std::fs::File;
use crate::{Gtfs0, Gtfs1, InProgressTrip, LibraryGTFS, StopsWithTrips};
use crate::trips_arena::TripsArena;

pub fn generate_stops_trips(gtfs: &Gtfs1) -> StopsWithTrips {
    let mut result = StopsWithTrips::default();
    for (_trip_id, trip) in &gtfs.trips {
        for st in &trip.stop_times {
            result.add_stop(st, trip);
        }
    }
    result
}

pub fn initialize_gtfs_as_bson(path: &str) -> Gtfs0 {
    if let Ok(file) = File::create_new(format!("{path}.bson")) {
        println!("GTFS not detected! Creating new");
        let gtfs = Gtfs0::from(LibraryGTFS::from_path(path).unwrap());
        let document = bson::to_document(&gtfs).unwrap();
        document.to_writer(file).unwrap();
        println!("GTFS created");
        gtfs
    } else {
        let file = File::open(format!("{path}.bson")).unwrap();
        bson::from_reader(file).unwrap()
    }
}

pub fn get_trip_transfers<'a>(
    arena: &'a TripsArena,
    mut trip: Id<InProgressTrip>,
) -> Vec<&'a InProgressTrip> {
    let mut history = Vec::new();

    loop {
        let in_progress_trip = arena.get_by_id(trip);
        history.push(in_progress_trip);

        if let Some(parent) = in_progress_trip.previous_transfer {
            trip = parent;
        } else {
            break;
        }
    }
    history
}
