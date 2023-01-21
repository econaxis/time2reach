use crate::objects::*;
use crate::Gtfs;
use crate::RawGtfs;
use chrono::NaiveDate;
use rgb::RGB8;

#[test]
fn serialization_deserialization() {
    // route, trip, stop, stop_times
    let gtfs = RawGtfs::from_path("fixtures/basic").expect("impossible to read gtfs");

    let string = serde_json::to_string(&gtfs.routes.unwrap()).unwrap();
    let _parsed: Vec<Route> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.trips.unwrap()).unwrap();
    let _parsed: Vec<RawTrip> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.stops.unwrap()).unwrap();
    let _parsed: Vec<Stop> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.stop_times.unwrap()).unwrap();
    let _parsed: Vec<RawStopTime> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.frequencies.unwrap().unwrap()).unwrap();
    let _parsed: Vec<RawFrequency> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.pathways.unwrap().unwrap()).unwrap();
    let _parsed: Vec<RawPathway> = serde_json::from_str(&string).unwrap();

    let string = serde_json::to_string(&gtfs.transfers.unwrap().unwrap()).unwrap();
    let _parsed: Vec<RawTransfer> = serde_json::from_str(&string).unwrap();
}
#[test]
fn read_calendar() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.calendar.len());
    assert!(!gtfs.calendar["service1"].monday);
    assert!(gtfs.calendar["service1"].saturday);
}

#[test]
fn read_calendar_dates() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(2, gtfs.calendar_dates.len());
    assert_eq!(2, gtfs.calendar_dates["service1"].len());
    assert_eq!(
        Exception::Deleted,
        gtfs.calendar_dates["service1"][0].exception_type
    );
    assert_eq!(
        Exception::Added,
        gtfs.calendar_dates["service2"][0].exception_type
    );
}

#[test]
fn read_stop() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(6, gtfs.stops.len());
    assert_eq!(
        LocationType::StopArea,
        gtfs.get_stop("stop1").unwrap().location_type
    );
    assert_eq!(
        LocationType::StopPoint,
        gtfs.get_stop("stop2").unwrap().location_type
    );
    assert_eq!(Some(48.796_058), gtfs.get_stop("stop2").unwrap().latitude);
    assert_eq!(
        Some("1".to_owned()),
        gtfs.get_stop("stop3").unwrap().parent_station
    );
    assert_eq!(
        LocationType::GenericNode,
        gtfs.get_stop("stop6").unwrap().location_type
    );
    assert_eq!(None, gtfs.get_stop("stop6").unwrap().latitude);
}

#[test]
fn read_routes() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(3, gtfs.routes.len());
    assert_eq!(RouteType::Bus, gtfs.get_route("1").unwrap().route_type);
    assert_eq!(RGB8::new(0, 0, 0), gtfs.get_route("1").unwrap().color);
    assert_eq!(
        RGB8::new(255, 255, 255),
        gtfs.get_route("1").unwrap().text_color
    );
    assert_eq!(RGB8::new(0, 0, 0), gtfs.get_route("1").unwrap().color);
    assert_eq!(
        RGB8::new(0, 0, 0),
        gtfs.get_route("default_colors").unwrap().text_color
    );
    assert_eq!(
        RGB8::new(255, 255, 255),
        gtfs.get_route("default_colors").unwrap().color
    );
    assert_eq!(Some(1), gtfs.get_route("1").unwrap().order);
    assert_eq!(
        RouteType::Other(42),
        gtfs.get_route("invalid_type").unwrap().route_type
    );
}

#[test]
fn read_trips() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.trips.len());
}

#[test]
fn read_stop_times() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let stop_times = &gtfs.trips.get("trip1").unwrap().stop_times;
    assert_eq!(3, stop_times.len());
    assert_eq!(PickupDropOffType::Regular, stop_times[0].pickup_type);
    assert_eq!(PickupDropOffType::NotAvailable, stop_times[0].drop_off_type);
    assert_eq!(PickupDropOffType::ArrangeByPhone, stop_times[1].pickup_type);
    assert_eq!(PickupDropOffType::Regular, stop_times[1].drop_off_type);
    assert_eq!(
        PickupDropOffType::Unknown(-999),
        stop_times[2].drop_off_type
    );
    assert_eq!(TimepointType::Exact, stop_times[0].timepoint);
    assert_eq!(TimepointType::Approximate, stop_times[1].timepoint);
}

#[test]
fn read_frequencies() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let trip = &gtfs.trips.get("trip1").unwrap();
    assert_eq!(1, trip.frequencies.len());
    let frequency = &trip.frequencies[0];
    assert_eq!(19800, frequency.start_time);
}

#[test]
fn read_agencies() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let agencies = &gtfs.agencies;
    assert_eq!("BIBUS", agencies[0].name);
    assert_eq!("http://www.bibus.fr", agencies[0].url);
    assert_eq!("Europe/Paris", agencies[0].timezone);
}

#[test]
fn read_shapes() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let shapes = &gtfs.shapes;
    assert_eq!(37.61956, shapes["A_shp"][0].latitude);
    assert_eq!(-122.48161, shapes["A_shp"][0].longitude);
}

#[test]
fn read_fare_attributes() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.fare_attributes.len());
    assert_eq!("1.50", gtfs.get_fare_attributes("50").unwrap().price);
    assert_eq!("EUR", gtfs.get_fare_attributes("50").unwrap().currency);
    assert_eq!(
        PaymentMethod::Aboard,
        gtfs.get_fare_attributes("50").unwrap().payment_method
    );
    assert_eq!(
        Transfers::Unlimited,
        gtfs.get_fare_attributes("50").unwrap().transfers
    );
    assert_eq!(
        Some("1".to_string()),
        gtfs.get_fare_attributes("50").unwrap().agency_id
    );
    assert_eq!(
        Some(3600),
        gtfs.get_fare_attributes("50").unwrap().transfer_duration
    );
}

#[test]
fn read_transfers() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.get_stop("stop3").unwrap().transfers.len());
    assert_eq!(1, gtfs.get_stop("stop1").unwrap().transfers.len());

    assert_eq!(
        "stop5",
        gtfs.get_stop("stop3").unwrap().transfers[0].to_stop_id
    );
    assert_eq!(
        "stop2",
        gtfs.get_stop("stop1").unwrap().transfers[0].to_stop_id
    );
    assert_eq!(
        TransferType::Recommended,
        gtfs.get_stop("stop3").unwrap().transfers[0].transfer_type
    );
    assert_eq!(
        TransferType::Impossible,
        gtfs.get_stop("stop1").unwrap().transfers[0].transfer_type
    );
    assert_eq!(
        Some(60),
        gtfs.get_stop("stop3").unwrap().transfers[0].min_transfer_time
    );
    assert_eq!(
        None,
        gtfs.get_stop("stop1").unwrap().transfers[0].min_transfer_time
    );
}

#[test]
fn read_pathways() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");

    let pathways = &gtfs.get_stop("stop1").unwrap().pathways;

    assert_eq!(1, pathways.len());
    assert_eq!("stop3", pathways[0].to_stop_id);
    assert_eq!(PathwayMode::Walkway, pathways[0].mode);
    assert_eq!(
        PathwayDirectionType::Unidirectional,
        pathways[0].is_bidirectional
    );
    assert_eq!(None, pathways[0].min_width);
}

#[test]
fn read_feed_info() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let feed = &gtfs.feed_info;
    assert_eq!(1, feed.len());
    assert_eq!("SNCF", feed[0].name);
    assert_eq!("http://www.sncf.com", feed[0].url);
    assert_eq!("fr", feed[0].lang);
    assert_eq!(Some(NaiveDate::from_ymd(2018, 07, 09)), feed[0].start_date);
    assert_eq!(Some(NaiveDate::from_ymd(2018, 09, 27)), feed[0].end_date);
    assert_eq!(Some("0.3".to_string()), feed[0].version);
}

#[test]
fn trip_days() {
    let gtfs = Gtfs::from_path("fixtures/basic/").unwrap();
    let days = gtfs.trip_days(&"service1".to_owned(), NaiveDate::from_ymd(2017, 1, 1));
    assert_eq!(vec![6, 7, 13, 14], days);

    let days2 = gtfs.trip_days(&"service2".to_owned(), NaiveDate::from_ymd(2017, 1, 1));
    assert_eq!(vec![0], days2);
}

#[test]
fn trip_clone() {
    let gtfs = Gtfs::from_path("fixtures/basic/").unwrap();
    let _: Trip = gtfs.trips.get("trip1").unwrap().clone();
}

#[test]
fn read_from_gtfs() {
    let gtfs = Gtfs::from_path("fixtures/zips/gtfs.zip").unwrap();
    assert_eq!(1, gtfs.calendar.len());
    assert_eq!(2, gtfs.calendar_dates.len());
    assert_eq!(5, gtfs.stops.len());
    assert_eq!(1, gtfs.routes.len());
    assert_eq!(1, gtfs.trips.len());
    assert_eq!(1, gtfs.shapes.len());
    assert_eq!(1, gtfs.fare_attributes.len());
    assert_eq!(1, gtfs.feed_info.len());
    assert_eq!(2, gtfs.get_trip("trip1").unwrap().stop_times.len());

    assert!(gtfs.get_calendar("service1").is_ok());
    assert!(gtfs.get_calendar_date("service1").is_ok());
    assert!(gtfs.get_stop("stop1").is_ok());
    assert!(gtfs.get_route("1").is_ok());
    assert!(gtfs.get_trip("trip1").is_ok());
    assert!(gtfs.get_fare_attributes("50").is_ok());

    assert!(gtfs.get_stop("Utopia").is_err());
}

#[test]
fn read_from_subdirectory() {
    let gtfs = Gtfs::from_path("fixtures/zips/subdirectory.zip").unwrap();
    assert_eq!(1, gtfs.calendar.len());
    assert_eq!(2, gtfs.calendar_dates.len());
    assert_eq!(5, gtfs.stops.len());
    assert_eq!(1, gtfs.routes.len());
    assert_eq!(1, gtfs.trips.len());
    assert_eq!(1, gtfs.shapes.len());
    assert_eq!(1, gtfs.fare_attributes.len());
    assert_eq!(2, gtfs.get_trip("trip1").unwrap().stop_times.len());
}

#[test]
fn display() {
    assert_eq!(
        "Sorano".to_owned(),
        format!(
            "{}",
            Stop {
                name: "Sorano".to_owned(),
                ..Stop::default()
            }
        )
    );

    assert_eq!(
        "Long route name".to_owned(),
        format!(
            "{}",
            Route {
                long_name: "Long route name".to_owned(),
                ..Route::default()
            }
        )
    );

    assert_eq!(
        "Short route name".to_owned(),
        format!(
            "{}",
            Route {
                short_name: "Short route name".to_owned(),
                ..Route::default()
            }
        )
    );
}

#[test]
fn path_files() {
    let gtfs = RawGtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    assert_eq!(gtfs.files.len(), 13);
}

#[test]
fn zip_files() {
    let gtfs = RawGtfs::from_path("fixtures/zips/gtfs.zip").expect("impossible to read gtfs");
    assert_eq!(gtfs.files.len(), 10);
    assert!(gtfs.files.contains(&"agency.txt".to_owned()));
}

#[test]
fn zip_subdirectory_files() {
    let gtfs =
        RawGtfs::from_path("fixtures/zips/subdirectory.zip").expect("impossible to read gtfs");
    assert_eq!(gtfs.files.len(), 11);
    assert!(gtfs.files.contains(&"subdirectory/agency.txt".to_owned()));
}

#[test]
fn compute_sha256() {
    let gtfs = RawGtfs::from_path("fixtures/zips/gtfs.zip").expect("impossible to read gtfs");
    assert_eq!(
        gtfs.sha256,
        Some("4a262ae109101ffbd1629b67e080a2b074afdaa60d57684db0e1a31c0a1e75b0".to_owned())
    );
}

#[test]
fn test_bom() {
    let gtfs =
        RawGtfs::from_path("fixtures/zips/gtfs_with_bom.zip").expect("impossible to read gtfs");
    assert_eq!(gtfs.agencies.expect("agencies missing").len(), 2);
}

#[test]
fn test_macosx() {
    let gtfs = RawGtfs::from_path("fixtures/zips/macosx.zip").expect("impossible to read gtfs");
    assert_eq!(gtfs.agencies.expect("agencies missing").len(), 2);
    assert_eq!(gtfs.stops.expect("stops missing").len(), 5);
}

#[test]
fn read_missing_feed_dates() {
    let gtfs = Gtfs::from_path("fixtures/missing_feed_date").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.feed_info.len());
    assert!(gtfs.feed_info[0].start_date.is_none());
}

#[test]
fn read_interpolated_stops() {
    let gtfs =
        Gtfs::from_path("fixtures/interpolated_stop_times").expect("impossible to read gtfs");
    assert_eq!(1, gtfs.feed_info.len());
    // the second stop have no departure/arrival, it should not cause any problems
    assert_eq!(
        gtfs.trips["trip1"].stop_times[1].stop.name,
        "Stop Point child of 1"
    );
    assert!(gtfs.trips["trip1"].stop_times[1].arrival_time.is_none());
}

#[test]
fn read_only_required_fields() {
    let gtfs = Gtfs::from_path("fixtures/only_required_fields").expect("impossible to read gtfs");
    let route = gtfs.routes.get("1").unwrap();
    let fare_attribute = gtfs.fare_attributes.get("50").unwrap();
    let feed = &gtfs.feed_info[0];
    let shape = &gtfs.shapes.get("A_shp").unwrap()[0];
    assert_eq!(route.color, RGB8::new(255, 255, 255));
    assert_eq!(route.text_color, RGB8::new(0, 0, 0));
    assert_eq!(fare_attribute.transfer_duration, None);
    assert_eq!(feed.start_date, None);
    assert_eq!(feed.end_date, None);
    assert_eq!(shape.dist_traveled, None);
    assert_eq!(
        TimepointType::Exact,
        gtfs.trips["trip1"].stop_times[0].timepoint
    )
}

#[test]
fn sorted_shapes() {
    let gtfs = Gtfs::from_path("fixtures/basic").expect("impossible to read gtfs");
    let shape = &gtfs.shapes.get("Unordered_shp").unwrap();

    let points = shape
        .into_iter()
        .map(|s| (s.sequence, s.latitude, s.longitude))
        .collect::<Vec<_>>();

    assert_eq!(
        points,
        vec![
            (0, 37.61956, -122.48161),
            (6, 37.64430, -122.41070),
            (11, 37.65863, -122.30839),
        ]
    );
}
