use crate::trips_arena::TripsArena;
use gtfs_structure_2::gtfs_wrapper::{Gtfs0, Gtfs1, LibraryGTFS};
use id_arena::Id;
use lazy_static::lazy_static;
use log::info;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::gtfs_processing::StopsWithTrips;
use crate::in_progress_trip::InProgressTrip;
use std::sync::Mutex;
lazy_static! {
    static ref AGENCY_MAP: Mutex<FxHashMap<String, u8>> = Mutex::new(FxHashMap::default());
}

#[inline(never)]
pub fn generate_stops_trips(gtfs: &Gtfs1) -> StopsWithTrips {
    let mut result = StopsWithTrips::default();
    for trip in gtfs.trips.values() {
        for st in &trip.stop_times {
            result.add_stop(st, trip);
        }
    }
    result
}

pub fn get_agency_id_from_short_name(short_name: &str) -> Option<u8> {
    let map = AGENCY_MAP.lock().unwrap();
    map.get(&short_name.to_ascii_uppercase()).copied()
}
pub fn initialize_gtfs_as_bson(path: &str, short_name: &str) -> Gtfs1 {
    info!("Loading schedules for {path}");
    let file = File::create_new(format!("{path}-1.rkyv"));

    let result: Gtfs1 = if let Ok(mut file) = file {
        if cfg!(feature = "prod") {
            panic!("Prod deployment -- not allowed to parse GTFS txt files. Not found {path}-1.rkyv file");
        }
        info!("GTFS not detected! Creating new");
        let gtfs = Gtfs1::from(Gtfs0::from(LibraryGTFS::from_path(path).unwrap()));
        let bytes = rkyv::to_bytes::<_, 1024>(&gtfs).unwrap();
        file.write_all(&bytes).unwrap();
        info!("GTFS created");
        gtfs
    } else {
        let mut file = File::open(format!("{path}-1.rkyv")).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        if cfg!(feature = "prod") {
            rkyv::from_bytes::<Gtfs1>(&bytes).unwrap()
        } else {
            unsafe { rkyv::from_bytes_unchecked(&bytes) }.unwrap()
        }
    };
    let sample_id = result.stops.keys().next().unwrap();
    let mut map = AGENCY_MAP.lock().unwrap();
    map.insert(short_name.to_string(), sample_id.0);
    result
}

pub fn get_trip_transfers(
    arena: &TripsArena,
    mut trip: Id<InProgressTrip>,
) -> Vec<&InProgressTrip> {
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
