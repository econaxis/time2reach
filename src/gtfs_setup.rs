use crate::trips_arena::TripsArena;
use crate::{Gtfs0, Gtfs1, LibraryGTFS};
use id_arena::Id;
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::gtfs_processing::StopsWithTrips;
use crate::in_progress_trip::InProgressTrip;
use std::sync::Mutex;
lazy_static! {
    static ref AGENCY_MAP: Mutex<HashMap<String, u8>> = Mutex::new(HashMap::new());
}

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

#[test]
fn test_1() {
    initialize_gtfs_as_bson("/Users/henry.nguyen@snapcommerce.com/Downloads/gtfs-test1");
}

pub fn get_agency_id_from_short_name(short_name: &str) -> u8 {
    let map = AGENCY_MAP.lock().unwrap();
    println!("{:?} {:?}", map, short_name);
    *map.get(&short_name.to_ascii_uppercase()).unwrap()
}
pub fn initialize_gtfs_as_bson(path: &str, short_name: &str) -> Gtfs1 {
    info!("For file {path}");
    let file = File::create_new(format!("{path}-1.rkyv"));
    let result: Gtfs1 = if let Ok(mut file) = file {
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
        unsafe { rkyv::from_bytes_unchecked::<Gtfs0>(&bytes) }
            .unwrap()
            .into()
    };
    let sample_id = result.stops.keys().next().unwrap();
    let mut map = AGENCY_MAP.lock().unwrap();
    map.insert(short_name.to_string(), sample_id.0);
    result
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
