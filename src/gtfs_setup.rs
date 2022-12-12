use crate::trips_arena::TripsArena;
use crate::{Gtfs0, Gtfs1, InProgressTrip, LibraryGTFS, StopsWithTrips};
use id_arena::Id;
use std::fs::File;

#[inline(never)]
pub fn generate_stops_trips(gtfs: &Gtfs1) -> StopsWithTrips {
    let mut result = StopsWithTrips::default();
    for (_trip_id, trip) in &gtfs.trips {
        for st in &trip.stop_times {
            result.add_stop(st, trip);
        }
    }
    result
}

#[inline(never)]
pub fn initialize_gtfs_as_bson(path: &str) -> Gtfs1 {
    if let Ok(file) = File::create_new(format!("{path}.bson")) {
        println!("GTFS not detected! Creating new");
        let gtfs = Gtfs0::from(LibraryGTFS::from_path(path).unwrap());
        let document = bson::to_document(&gtfs).unwrap();
        document.to_writer(file).unwrap();
        println!("GTFS created");
        gtfs.into()
    } else {
        let file = File::open(format!("{path}.bson")).unwrap();
        bson::from_reader::<File, Gtfs0>(file).unwrap().into()
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
