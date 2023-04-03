use crate::trips_arena::TripsArena;
use crate::{Gtfs0, Gtfs1, InProgressTrip, LibraryGTFS, StopsWithTrips};
use id_arena::Id;
use log::info;
use std::fs::File;
use std::io::{Read, Write};

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
    info!("For file {path}");
    let file = File::create_new(format!("{path}-1.rkyv"));
    if let Ok(mut file) = file {
        info!("GTFS not detected! Creating new");
        let gtfs = Gtfs0::from(LibraryGTFS::from_path(path).unwrap());
        let bytes = rkyv::to_bytes::<_, 1024>(&gtfs).unwrap();
        file.write_all(&bytes).unwrap();
        info!("GTFS created");
        gtfs.into()
    } else {
        let mut file = File::open(format!("{path}-1.rkyv")).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        unsafe { rkyv::from_bytes_unchecked::<Gtfs0>(&bytes) }.unwrap().into()
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
