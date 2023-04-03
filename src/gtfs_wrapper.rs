use crate::IdType;
use gtfs_structures::{
    Availability, BikesAllowedType, Frequency, RawTrip,
};
use rkyv::{Serialize, Deserialize, Archive};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

pub type LibraryGTFS = gtfs_structures::RawGtfs;

static AGENCY_COUNT: AtomicU8 = AtomicU8::new(0);

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum RouteType {
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area
    Tramway,
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area
    Subway,
    /// Used for intercity or long-distance travel
    Rail,
    /// Used for short- and long-distance bus routes
    #[default]
    Bus,
    /// Used for short- and long-distance boat service
    Ferry,
    /// Used for street-level rail cars where the cable runs beneath the vehicle, e.g., cable car in San Francisco
    CableCar,
    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway). Cable transport where cabins, cars, gondolas or open chairs are suspended by means of one or more cables
    Gondola,
    /// Any rail system designed for steep inclines
    Funicular,
    /// (extended) Used for intercity bus services
    Coach,
    /// (extended) Airplanes
    Air,
    /// (extended) Taxi, Cab
    Taxi,
    /// (extended) any other value
    Other(i32),
}

impl From<gtfs_structures::RouteType> for RouteType {
    fn from(value: gtfs_structures::RouteType) -> Self {
        match value {
            gtfs_structures::RouteType::Tramway => RouteType::Tramway,
            gtfs_structures::RouteType::Subway => RouteType::Subway,
            gtfs_structures::RouteType::Rail => RouteType::Rail,
            gtfs_structures::RouteType::Bus => RouteType::Bus,
            gtfs_structures::RouteType::Ferry => RouteType::Ferry,
            gtfs_structures::RouteType::CableCar => RouteType::CableCar,
            gtfs_structures::RouteType::Gondola => RouteType::Gondola,
            gtfs_structures::RouteType::Funicular => RouteType::Funicular,
            gtfs_structures::RouteType::Coach => RouteType::Coach,
            gtfs_structures::RouteType::Air => RouteType::Air,
            gtfs_structures::RouteType::Taxi => RouteType::Taxi,
            gtfs_structures::RouteType::Other(i32) => RouteType::Other(i32),
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Default, Copy)]
pub enum DirectionType {
    /// Travel in one direction (e.g. outbound travel).
    #[default]
    Outbound,
    /// Travel in the opposite direction (e.g. inbound travel).
    Inbound,
}

impl From<gtfs_structures::DirectionType> for DirectionType {
    fn from(value: gtfs_structures::DirectionType) -> Self {
        match value {
            gtfs_structures::DirectionType::Outbound => DirectionType::Outbound,
            gtfs_structures::DirectionType::Inbound => DirectionType::Inbound,
        }
    }
}
#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum LocationType {
    /// Stop (or Platform). A location where passengers board or disembark from a transit vehicle. Is called a platform when defined within a parent_station
    #[default]
    StopPoint,
    /// Station. A physical structure or area that contains one or more platform
    StopArea,
    /// A location where passengers can enter or exit a station from the street. If an entrance/exit belongs to multiple stations, it can be linked by pathways to both, but the data provider must pick one of them as parent
    StationEntrance,
    /// A location within a station, not matching any other [Stop::location_type], which can be used to link together pathways define in pathways.txt.
    GenericNode,
    /// A specific location on a platform, where passengers can board and/or alight vehicles
    BoardingArea,
    /// An unknown value
    Unknown(i32),
}

impl From<gtfs_structures::LocationType> for LocationType {
    fn from(value: gtfs_structures::LocationType) -> Self {
        match value {
            gtfs_structures::LocationType::StopPoint => LocationType::StopPoint,
            gtfs_structures::LocationType::StopArea => LocationType::StopArea,
            gtfs_structures::LocationType::StationEntrance => LocationType::StationEntrance,
            gtfs_structures::LocationType::GenericNode => LocationType::GenericNode,
            gtfs_structures::LocationType::BoardingArea => LocationType::BoardingArea,
            gtfs_structures::LocationType::Unknown(i32) => LocationType::Unknown(i32),
        }
    }
}

trait FromWithAgencyId<From> {
    fn from_with_agency_id(agency_id: u8, f: From) -> Self
    where
        Self: Sized;
}

thread_local! {
    static ID_MAP: RefCell<HashMap<String, u64>> = {
        RefCell::new(HashMap::new())
    };
}
fn try_parse_id(a: &str) -> u64 {
    ID_MAP.with(|idmap| {
        let mut idmap = idmap.borrow_mut();
        if let Some(id) = idmap.get(a) {
            *id
        } else {
            let id = idmap.len() as u64;
            idmap.insert(a.to_string(), id);
            id
        }
    })
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Archive)]
pub struct StopTime {
    /// Arrival time of the stop time.
    /// It's an option since the intermediate stops can have have no arrival
    /// and this arrival needs to be interpolated
    pub arrival_time: Option<u32>,
    /// Order of stops for a particular trip. The values must increase along the trip but do not need to be consecutive
    pub stop_sequence: u16,
    /// Text that appears on signage identifying the trip's destination to riders
    pub stop_id: IdType,

    pub trip_id: IdType,

    pub index_of_stop_time: usize,
}

/// A physical stop, station or area. See <https://gtfs.org/reference/static/#stopstxt>
#[derive(Debug, Serialize, Deserialize, Clone, Default, Archive)]
pub struct Stop {
    /// Unique technical identifier (not for the traveller) of the stop
    pub id: IdType,
    /// Short text or a number that identifies the location for riders
    pub code: Option<String>,
    ///Name of the location. Use a name that people will understand in the local and tourist vernacular
    pub name: String,
    /// Type of the location
    pub longitude: Option<f64>,
    /// Latitude of the stop
    pub latitude: Option<f64>,
    pub location_type: LocationType,
    pub parent_station: Option<String>,
}

impl FromWithAgencyId<gtfs_structures::Stop> for Stop {
    fn from_with_agency_id(agency_id: u8, f: gtfs_structures::Stop) -> Self
    where
        Self: Sized,
    {
        Self {
            id: (agency_id, try_parse_id(&f.id)),
            code: f.code,
            name: f.name,
            longitude: f.longitude,
            latitude: f.latitude,
            location_type: f.location_type.into(),
            parent_station: f.parent_station,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Archive)]
pub enum Availability1 {
    /// No information if the service is available
    #[default]
    InformationNotAvailable,
    /// The service is available
    Available,
    /// The service is not available
    NotAvailable,
    /// An unknown value not in the specification
    Unknown(i32),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Archive)]
pub struct Trip {
    /// Unique technical identifier (not for the traveller) for the Trip
    pub id: IdType,
    /// References the [Calendar] on which this trip runs
    pub service_id: IdType,
    /// References along which [Route] this trip runs
    pub route_id: IdType,
    /// All the [StopTime] that define the trip
    pub stop_times: Vec<StopTime>,
    /// Text that appears on signage identifying the trip's destination to riders
    pub shape_id: Option<IdType>,
    /// Text that appears on signage identifying the trip's destination to riders
    pub trip_headsign: Option<String>,
    /// Public facing text used to identify the trip to riders, for instance, to identify train numbers for commuter rail trips
    pub trip_short_name: Option<String>,
    /// Indicates the direction of travel for a trip. This field is not used in routing; it provides a way to separate trips by direction when publishing time tables
    pub direction_id: Option<DirectionType>,
    /// Identifies the block to which the trip belongs. A block consists of a single trip or many sequential trips made using the same vehicle, defined by shared service days and block_id. A block_id can have trips with different service days, making distinct blocks
    pub block_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Archive)]
pub struct Route {
    /// Unique technical (not for the traveller) identifier for the route
    pub id: IdType,
    /// Short name of a route. This will often be a short, abstract identifier like "32", "100X", or "Green" that riders use to identify a route, but which doesn't give any indication of what places the route serves
    pub short_name: String,
    /// Full name of a route. This name is generally more descriptive than the [Route::short_name]] and often includes the route's destination or stop
    pub long_name: String,
    /// Description of a route that provides useful, quality information
    pub desc: Option<String>,
    /// Indicates the type of transportation used on a route
    pub route_type: RouteType,
    /// URL of a web page about the particular route
    pub url: Option<String>,
    /// Orders the routes in a way which is ideal for presentation to customers. Routes with smaller route_sort_order values should be displayed first.
    pub order: Option<u32>,
}

impl FromWithAgencyId<gtfs_structures::Route> for Route {
    fn from_with_agency_id(agency_id: u8, a: gtfs_structures::Route) -> Self {
        Self {
            id: (agency_id, try_parse_id(&a.id)),
            short_name: a.short_name,
            long_name: a.long_name,
            desc: a.desc,
            route_type: a.route_type.into(),
            url: a.url,
            order: a.order,
        }
    }
}

impl FromWithAgencyId<RawTrip> for Trip {
    fn from_with_agency_id(agency_id: u8, a: RawTrip) -> Self {
        Self {
            id: (agency_id, try_parse_id(&a.id)),
            service_id: (agency_id, try_parse_id(&a.service_id)),
            route_id: (agency_id, try_parse_id(&a.route_id)),
            stop_times: Default::default(),
            shape_id: a.shape_id.map(|a| (agency_id, try_parse_id(&a))),
            trip_headsign: a.trip_headsign,
            trip_short_name: a.trip_short_name,
            direction_id: a.direction_id.map(Into::into),
            block_id: a.block_id,
        }
    }
}
#[derive(Default, Serialize, Deserialize, Archive)]
pub struct Gtfs0 {
    /// All stop by `stop_id`. Stops are in an [Arc] because they are also referenced by each [StopTime]
    pub stops: Vec<Stop>,
    /// All routes by `route_id`
    pub routes: Vec<Route>,
    /// All trips by `trip_id`
    pub trips: Vec<Trip>,
    pub stop_times: Vec<StopTime>,
    pub agency_id: u8,
}

#[derive(Debug)]
pub struct Gtfs1 {
    /// All stop by `stop_id`. Stops are in an [Arc] because they are also referenced by each [StopTime]
    pub stops: HashMap<IdType, Stop>,
    /// All routes by `route_id`
    pub routes: HashMap<IdType, Route>,
    /// All trips by `trip_id`
    pub trips: HashMap<IdType, Trip>,
}

fn vec_to_hashmap<T, F: Fn(&T) -> IdType>(vec: Vec<T>, accessor: F) -> HashMap<IdType, T> {
    let mut hashmap = HashMap::new();
    for v in vec {
        let id = accessor(&v);
        hashmap.insert(id, v);
    }
    hashmap
}

impl From<Gtfs0> for Gtfs1 {
    fn from(mut a: Gtfs0) -> Self {
        AGENCY_COUNT.fetch_add(1, Ordering::SeqCst);

        let stops = vec_to_hashmap(a.stops, |stop| stop.id);
        let mut trips: HashMap<IdType, Trip> = a
            .trips
            .into_iter()
            .map(|a| {
                (
                    a.id,
                    Trip {
                        id: a.id,
                        service_id: a.service_id,
                        route_id: a.route_id,
                        shape_id: a.shape_id,
                        trip_headsign: a.trip_headsign,
                        trip_short_name: a.trip_short_name,
                        direction_id: a.direction_id,
                        block_id: a.block_id,
                        stop_times: Default::default(),
                    },
                )
            })
            .collect();

        a.stop_times
            .sort_by(|a, b| a.stop_sequence.cmp(&b.stop_sequence));

        for mut st in a.stop_times {
            let stop_time_vec = &mut trips.get_mut(&st.trip_id).unwrap().stop_times;
            st.index_of_stop_time = stop_time_vec.len();
            stop_time_vec.push(st);
        }

        Self {
            stops,
            routes: vec_to_hashmap(a.routes, |route| route.id),
            trips,
        }
    }
}

impl From<LibraryGTFS> for Gtfs0 {
    fn from(a: LibraryGTFS) -> Self {
        let agency_id = AGENCY_COUNT.fetch_add(1, Ordering::SeqCst);
        Self {
            stops: a
                .stops
                .unwrap()
                .into_iter()
                .map(|a| Stop::from_with_agency_id(agency_id, a))
                .collect(),
            routes: a
                .routes
                .unwrap()
                .into_iter()
                .map(|a| Route::from_with_agency_id(agency_id, a))
                .collect(),
            trips: a
                .trips
                .unwrap()
                .into_iter()
                .map(|a| Trip::from_with_agency_id(agency_id, a))
                .collect(),
            stop_times: a
                .stop_times
                .unwrap()
                .into_iter()
                .map(|st| StopTime {
                    arrival_time: st.arrival_time,
                    stop_sequence: st.stop_sequence,
                    stop_id: (agency_id, try_parse_id(&st.stop_id)),
                    trip_id: (agency_id, try_parse_id(&st.trip_id)),
                    index_of_stop_time: 0,
                })
                .collect(),
            agency_id,
        }
    }
}

impl Gtfs1 {
    pub fn merge(&mut self, other: Gtfs1) {
        self.stops.extend(other.stops);
        self.routes.extend(other.routes);
        self.trips.extend(other.trips);
    }
}
