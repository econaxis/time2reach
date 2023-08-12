use std::borrow::Cow;

use crate::calendar::{Calendar, CalendarException, Service};
use crate::shape::Shape;
use crate::IdType;
use gtfs_structures::{Agency, CalendarDate, RawGtfs, RawStopTime, RawTrip};
use rkyv::{Archive, Deserialize, Serialize};
use rstar::primitives::{GeomWithData, Line};
use rstar::PointDistance;
use rstar::RTree;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::{AtomicU16, Ordering};

pub type LibraryGTFS = gtfs_structures::RawGtfs;

pub type AgencyId = u16;

static AGENCY_COUNT: AtomicU16 = AtomicU16::new(0);

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[archive(check_bytes)]
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

impl TryFrom<&str> for RouteType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "bus" => Ok(RouteType::Bus),
            "tram" => Ok(RouteType::Tramway),
            "subway" => Ok(RouteType::Subway),
            "rail" => Ok(RouteType::Rail),
            "ferry" => Ok(RouteType::Ferry),
            _ => Err(format!("Unknown route type: {}", value)),
        }
    }
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

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default, Copy)]
#[archive(check_bytes)]
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

#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[archive(check_bytes)]
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

pub trait FromWithAgencyId<From> {
    fn from_with_agency_id(agency_id: u16, f: From) -> Self
    where
        Self: Sized;
}

thread_local! {
    static ID_MAP: RefCell<FxHashMap<String, u64>> = {
        RefCell::new(FxHashMap::default())
    };
}
pub fn try_parse_id(a: &str) -> u64 {
    match a.parse() {
        Ok(x) => x,
        Err(_) => ID_MAP.with(|idmap| {
            let mut idmap = idmap.borrow_mut();
            if let Some(id) = idmap.get(a) {
                *id
            } else {
                let id = idmap.len() as u64 + 10000000000;
                idmap.insert(a.to_string(), id);
                id
            }
        }),
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Archive)]
#[archive(check_bytes)]
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
    pub shape_index: f32,
}

/// A physical stop, station or area. See <https://gtfs.org/reference/static/#stopstxt>
#[derive(Debug, Serialize, Deserialize, Clone, Default, Archive)]
#[archive(check_bytes)]
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
    // pub shape_travelled_index: f64
}

impl FromWithAgencyId<gtfs_structures::Stop> for Stop {
    fn from_with_agency_id(agency_id: u16, f: gtfs_structures::Stop) -> Self
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
#[archive(check_bytes)]
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
#[archive(check_bytes)]
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
    pub block_id: Option<String>
}


impl Trip {
    fn generate_shape_for_trip(&self, gtfs: &Gtfs1) -> Vec<Shape> {
        assert_eq!(self.shape_id, None);

        self.stop_times.iter().enumerate().map(|(index, st)| {
            let stop = &gtfs.stops[&st.stop_id];
            Shape {
                id: (0, 0),
                latitude: stop.latitude.unwrap(),
                longitude: stop.longitude.unwrap(),
                sequence: index,
                dist_traveled: None,
            }
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Archive)]
#[archive(check_bytes)]
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
    pub agency_id: Option<String>,
    /// Orders the routes in a way which is ideal for presentation to customers. Routes with smaller route_sort_order values should be displayed first.
    pub order: Option<u32>,
    pub color: String,
    pub text_color: String,
}

impl FromWithAgencyId<gtfs_structures::Route> for Route {
    fn from_with_agency_id(agency_id: u16, a: gtfs_structures::Route) -> Self {
        Self {
            agency_id: a.agency_id,
            id: (agency_id, try_parse_id(&a.id)),
            short_name: a.short_name,
            long_name: a.long_name,
            desc: a.desc,
            route_type: a.route_type.into(),
            url: a.url,
            order: a.order,
            color: a.color.to_string(),
            text_color: a.text_color.to_string(),
        }
    }
}

impl FromWithAgencyId<RawTrip> for Trip {
    fn from_with_agency_id(agency_id: u16, a: RawTrip) -> Self {
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

#[derive(Default)]
pub struct Gtfs0 {
    /// All stop by `stop_id`. Stops are in an [Arc] because they are also referenced by each [StopTime]
    pub stops: Vec<Stop>,
    pub shapes: Vec<Shape>,
    /// All routes by `route_id`
    pub routes: Vec<Route>,
    /// All trips by `trip_id`
    pub trips: Vec<Trip>,
    pub stop_times: Vec<StopTime>,
    pub calendar: Vec<Service>,
    pub calendar_dates: Vec<CalendarException>,
    pub agency_id: u16,
    pub agency: Agency,
}


#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
pub struct Gtfs1 {
    /// All stop by `stop_id`. Stops are in an [Arc] because they are also referenced by each [StopTime]
    pub stops: FxHashMap<IdType, Stop>,
    /// All routes by `route_id`
    pub routes: FxHashMap<IdType, Route>,
    /// All trips by `trip_id`
    pub trips: FxHashMap<IdType, Trip>,
    pub shapes: FxHashMap<IdType, Vec<Shape>>,

    pub generated_shapes: FxHashMap<IdType, Vec<Shape>>,

    pub calendar: Calendar,
    pub agency_id: u16,
    pub agency_city: String,
    pub agency_name: String
}

impl Gtfs1 {
    fn generate_shapes(&mut self) {
        for trip in self.trips.values() {
            if trip.shape_id.is_none() {
                let shape = trip.generate_shape_for_trip(self);
                self.generated_shapes.insert(trip.id, shape);
            }
        }

        for trip in self.trips.values_mut() {
            if trip.shape_id.is_none() {
                trip.shape_id = Some(trip.id);
            }
        }
    }

    pub fn get_shape(&self, trip: &Trip) -> &Vec<Shape> {
        let shape_id = trip.shape_id.unwrap();
        &self.shapes.get(&shape_id).or_else(|| self.generated_shapes.get(&shape_id)).unwrap()
    }
}


pub struct Gtfs0WithCity {
    pub gtfs0: Gtfs0,
    pub agency_city: String,
}

pub fn vec_to_hashmap<T, F: Fn(&T) -> IdType>(vec: Vec<T>, accessor: F) -> FxHashMap<IdType, T> {
    let mut hashmap = FxHashMap::default();
    for v in vec {
        let id = accessor(&v);
        hashmap.insert(id, v);
    }
    hashmap
}

fn convert_shapes(mut shape: Vec<Shape>) -> FxHashMap<IdType, Vec<Shape>> {
    shape.sort_by(|a, b| a.id.cmp(&b.id));

    let mut answer = FxHashMap::default();
    for shape_seq in shape.group_by(|a, b| a.id == b.id) {
        let shape_id = shape_seq[0].id;
        let mut shape_vec = shape_seq.to_vec();
        shape_vec.sort_by(|a, b| a.sequence.cmp(&b.sequence));
        answer.insert(shape_id, shape_vec);
    }

    answer
}
impl From<Gtfs0WithCity> for Gtfs1 {
    fn from(mut b: Gtfs0WithCity) -> Self {
        let mut a = b.gtfs0;
        AGENCY_COUNT.fetch_add(1, Ordering::SeqCst);

        let stops = vec_to_hashmap(a.stops, |stop| stop.id);
        let shapes = convert_shapes(a.shapes);
        let mut trips: FxHashMap<IdType, Trip> = a
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

        let calendar = Calendar::parse(a.calendar, a.calendar_dates);
        let mut self_ = Self {
            stops,
            shapes,
            routes: vec_to_hashmap(a.routes, |route| route.id),
            trips,
            calendar,
            agency_id: a.agency_id,
            agency_city: b.agency_city,
            agency_name: a.agency.name,
            generated_shapes: Default::default(),
        };

        self_.generate_shapes();
        process_stop_times_with_shape_dist_travelled(&mut self_);

        self_
    }
}

fn process_stop_times_with_shape_dist_travelled(gtfs: &mut Gtfs1) {
    let geo_shape = generate_rtree_for_shapes(&gtfs.shapes);
    let geo_shape_generated = generate_rtree_for_shapes(&gtfs.generated_shapes);

    for trip in gtfs.trips.values_mut() {
        if geo_shape_generated.contains_key(&trip.shape_id.unwrap()) {
            place_stop_along_shape(&gtfs.stops, &geo_shape_generated, trip);
        } else if geo_shape.contains_key(&trip.shape_id.unwrap()) {
            place_stop_along_shape(&gtfs.stops, &geo_shape, trip);
        } else {
            panic!("Shape not found for trip {:?}", trip.id);
        }

        for st in &trip.stop_times {
            assert!((st.shape_index + 1.0).abs() > 1e-6);
        }
    }
}

fn place_stop_along_shape(stops_map: &FxHashMap<IdType, Stop>, geo_shape: &FxHashMap<IdType, RTree<GeomWithData<Line<[f64; 2]>, usize>>>, trip: &mut Trip) {
    for stop_time in &mut trip.stop_times {
        let stop = &stops_map[&stop_time.stop_id];
        let shape_rstar = &geo_shape[&trip.shape_id.unwrap()];

        let stop_query_point = [stop.longitude.unwrap(), stop.latitude.unwrap()];
        let nearest_shape_edge = shape_rstar.nearest_neighbor(&stop_query_point).unwrap();
        let nearest_point = nearest_shape_edge.geom().nearest_point(&stop_query_point);

        let length = nearest_shape_edge.geom().length_2();

        if length < 1e-6 {
            stop_time.shape_index = nearest_shape_edge.data as f32;
        } else {
            let fraction =
                (nearest_point.distance_2(&nearest_shape_edge.geom().from) / length).sqrt();

            assert!((0.0..=1.0).contains(&fraction));
            stop_time.shape_index = fraction as f32 + nearest_shape_edge.data as f32;
        }
    }
}

fn generate_rtree_for_shapes(shapes: &FxHashMap<IdType, Vec<Shape>>) -> FxHashMap<IdType, RTree<GeomWithData<Line<[f64; 2]>, usize>>> {
    let geo_shape: FxHashMap<_, _> = shapes
        .iter()
        .map(|(id, shape)| {
            let mut edge_index_iter = 0;

            let edges: Vec<GeomWithData<Line<[f64; 2]>, usize>> = shape
                .windows(2)
                .map(|edge| {
                    let geo_line_string = Shape::to_geo_types(edge).0;
                    match geo_line_string.as_slice() {
                        [a, b] => {
                            let line = Line {
                                from: [a.x, a.y],
                                to: [b.x, b.y],
                            };
                            edge_index_iter += 1;
                            GeomWithData::new(line, edge_index_iter - 1)
                        }
                        _ => {
                            panic!("incorrect geo length")
                        }
                    }
                })
                .collect();
            (
                *id,
                RTree::<GeomWithData<Line<[f64; 2]>, usize>>::bulk_load(edges),
            )
        })
        .collect();
    geo_shape
}

fn build_hashset<T, H: Hash + Eq, F: Fn(&T) -> H>(list: &[T], f: F) -> FxHashSet<H> {
    let mut hashset = FxHashSet::default();
    for item in list {
        hashset.insert(f(item));
    }
    hashset
}

pub fn split_by_agency(mut gtfs: LibraryGTFS) -> Vec<LibraryGTFS> {
    let mut answer = Vec::new();

    if gtfs.agencies.as_ref().unwrap().len() <= 1 {
        return vec![gtfs];
    }

    for agency in gtfs.agencies.as_mut().unwrap() {
        if agency.id.is_none() {
            agency.id = Some(agency.name.clone());
        }
    }
    'a: for agency in gtfs.agencies.as_ref().unwrap() {
        let agency_id = agency.id.clone().unwrap();


        println!("Found agency: {}", agency.name);

        // We care about shapes, calendar, calendar_dates, stops, routes, trips, stop_times
        // routes -> trips -> calendar -> calendar_dates -> shape -> stop_times -> stops

        let (routes, trips, calendar, calendar_dates, shape, stop_times, stops) = extract_objects_by_agency(&gtfs, &agency_id);

        for st in &stop_times {
            if st.arrival_time.is_none()  {
                eprintln!("Missing arrival time for stop time {:?} agency {}", st, agency.name);
                continue 'a;
            }
        }

        let raw = LibraryGTFS {
            read_duration: gtfs.read_duration,
            calendar: Some(Ok(calendar)),
            calendar_dates: Some(Ok(calendar_dates)),
            stops: Ok(stops),
            routes: Ok(routes),
            trips: Ok(trips),
            agencies: Ok(vec![agency.clone()]),
            shapes: Some(Ok(shape)),
            fare_attributes: None,
            frequencies: None,
            transfers: None,
            pathways: None,
            feed_info: None,
            stop_times: Ok(stop_times),
            files: vec![],
            sha256: None,
        };

        answer.push(raw);
    }

    answer
}

fn unwrap_or_default<T, E>(x: &Option<Result<T, E>>) -> Cow<T>
where
    T: Default + Clone,
{
    match x {
        Some(Ok(x)) => Cow::Borrowed(x),
        _ => Cow::Owned(Default::default()),
    }
}

fn extract_objects_by_agency(gtfs: &LibraryGTFS, agency_id: &str) -> (Vec<gtfs_structures::Route>, Vec<gtfs_structures::RawTrip>, Vec<gtfs_structures::Calendar>, Vec<gtfs_structures::CalendarDate>, Vec<gtfs_structures::Shape>, Vec<gtfs_structures::RawStopTime>, Vec<gtfs_structures::Stop>) {
    // Nasty code to extract all the objects depending on agency_id
    let routes = gtfs.routes.as_ref().unwrap().iter().filter(|x| (x.agency_id.as_ref().unwrap() == agency_id)).cloned().collect::<Vec<_>>();

    let routes_hash = build_hashset(&routes, |x| x.id.clone());
    let trips = gtfs.trips.as_ref().unwrap().iter().filter(|x| routes_hash.contains(&x.route_id)).cloned().collect::<Vec<_>>();
    let trips_service_id_hash: FxHashSet<String> = build_hashset(&trips, |x| x.service_id.clone());
    let trips_shape_hash: FxHashSet<String> = build_hashset(&trips, |x| x.shape_id.clone().unwrap_or("NOT FOUND".to_string()));
    let trips_id_hash = build_hashset(&trips, |x| x.id.clone());
    let calendar = unwrap_or_default(&gtfs.calendar).iter().filter(|x| trips_service_id_hash.contains(&x.id)).cloned().collect::<Vec<_>>();
    let calendar_dates = unwrap_or_default(&gtfs.calendar_dates).iter().filter(|x| trips_service_id_hash.contains(&x.service_id)).cloned().collect::<Vec<_>>();

    let shape = unwrap_or_default(&gtfs.shapes).iter().filter(|x| trips_shape_hash.contains(&x.id)).cloned().collect::<Vec<_>>();

    let stop_times = gtfs.stop_times.as_ref().unwrap().iter().filter(|x| trips_id_hash.contains(&x.trip_id)).cloned().collect::<Vec<_>>();

    let stop_id_hash = build_hashset(&stop_times, |x| x.stop_id.clone());
    let stops = gtfs.stops.as_ref().unwrap().iter().filter(|x| stop_id_hash.contains(&x.id)).cloned().collect::<Vec<_>>();
    (routes, trips, calendar, calendar_dates, shape, stop_times, stops)
}

impl From<LibraryGTFS> for Gtfs0 {
    fn from(a: LibraryGTFS) -> Self {
        let agency_id = AGENCY_COUNT.fetch_add(1, Ordering::SeqCst);
        assert_eq!(a.agencies.as_ref().unwrap().len(), 1);
        Self {
            agency: a.agencies.unwrap()[0].clone(),
            shapes: a
                .shapes
                .unwrap_or(Ok(vec![]))
                .unwrap_or_default()
                .into_iter()
                .map(|a| Shape::from_with_agency_id(agency_id, a))
                .collect(),
            calendar: a
                .calendar
                .unwrap_or(Ok(vec![]))
                .unwrap_or_default()
                .into_iter()
                .map(|a| Service::from_with_agency_id(agency_id, a))
                .collect(),
            calendar_dates: a
                .calendar_dates
                .unwrap_or(Ok(vec![]))
                .unwrap_or_default()
                .into_iter()
                .map(|a| CalendarException::from_with_agency_id(agency_id, a))
                .collect(),
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
                    shape_index: -1.0,
                })
                .collect(),
            agency_id,
        }
    }
}

impl Gtfs1 {
    pub fn merge(mut self, other: Gtfs1) -> Gtfs1 {
        assert_eq!(self.agency_city, other.agency_city);

        self.stops.extend(other.stops);

        self.routes.extend(other.routes);

        self.trips.extend(other.trips);

        self.shapes.extend(other.shapes);

        self.calendar.extend(other.calendar);

        self.generated_shapes.extend(other.generated_shapes);

        Gtfs1 {
            stops: self.stops,
            routes: self.routes,
            trips: self.trips,
            shapes: self.shapes,
            generated_shapes: self.generated_shapes,
            calendar: self.calendar,
            agency_id: self.agency_id,
            agency_city: self.agency_city,
            agency_name: self.agency_name,
        }
    }
}
