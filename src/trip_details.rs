use crate::formatter::get_route_mode;
use crate::web::RequestId;
use crate::web_app_data::AllAppData;
use crate::{time_to_point, LatLng, NULL_ID, WALKING_SPEED};
use geo_types::Coord;
use geojson::PointType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use warp::Reply;

#[derive(Deserialize)]
pub struct CalculateRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub agencies: Vec<String>,
    pub modes: Vec<String>,

    #[serde(rename = "startTime")]
    pub start_time: u64,

    #[serde(rename = "previousRequestId")]
    pub previous_request_id: Option<RequestId>,
}

#[derive(Serialize, Deserialize)]
struct TripDetailsInner {
    time: f64,
    line: String,
    stop: String,
}

#[derive(Serialize)]
struct TripDetailsTransit {
    background_color: String,
    text_color: String,
    mode: &'static str,
    boarding: TripDetailsInner,
    exit: TripDetailsInner,
}

#[derive(Serialize)]
struct TripDetailsWalking {
    time: f64,
    length: f32,
}

#[derive(Serialize)]
#[serde(tag = "method")]
enum TripDetails {
    Transit(TripDetailsTransit),
    Walking(TripDetailsWalking),
}

#[derive(Deserialize)]
pub struct GetDetailsRequest {
    pub latlng: LatLng,
    pub request_id: RequestId,
}

fn to_point_type(c: &Coord) -> PointType {
    vec![c.x, c.y]
}

pub fn get_trip_details(ad: Arc<AllAppData>, req: GetDetailsRequest) -> impl Reply {
    let latlng = req.latlng;
    let city = req.request_id.city;
    let ad = ad.ads.get(&city).unwrap();

    ad.rs_list.write().unwrap().pre_get(req.request_id.rs_list_index);

    let rs_list = ad.rs_list.read().unwrap();
    let rs_option = rs_list.get(req.request_id.rs_list_index);

    if rs_option.is_none() {
        return warp::reply::json(&"Invalid -- request ID not found");
    }

    let rs = rs_option.unwrap();
    let formatter = time_to_point(
        rs,
        &rs.trips_arena,
        &ad.gtfs,
        [latlng.latitude, latlng.longitude],
        true,
    );

    if formatter.is_none() {
        return warp::reply::json(&"None");
    }

    let formatter = formatter.unwrap();

    let mut details_list = Vec::new();

    let final_walking_time = formatter.final_walking_length as f64 / WALKING_SPEED;
    if final_walking_time >= 40.0 {
        details_list.push(TripDetails::Walking(TripDetailsWalking {
            time: final_walking_time,
            length: formatter.final_walking_length,
        }))
    }

    // Automatically skips
    let mut has_free_transfer_from_prev = false;

    let path_shape = formatter.construct_shape();

    let mut features: Vec<_> = path_shape
        .0
        .iter()
        .map(|a| geojson::Feature::from(geojson::Geometry::from(a)))
        .collect();

    let mut features_points: Vec<_> = path_shape
        .0
        .iter()
        .map(|a| {
            let first_point = a.0.first().unwrap();
            let last_point = a.0.last().unwrap();

            let geom_multipoint = geojson::Value::MultiPoint(vec![
                to_point_type(first_point),
                to_point_type(last_point),
            ]);

            geojson::Feature::from(geom_multipoint)
        })
        .collect();

    let features_zip_iter = features.iter_mut().zip(&mut features_points);
    for ((feature, feature_point), trip) in features_zip_iter.zip(&formatter.trips) {
        if trip.current_route.route_id == NULL_ID {
            // Begin of trip. Skip here.
            continue;
        }

        let route = &ad.gtfs.routes[&trip.current_route.route_id];
        let boarding_stop = &ad.gtfs.stops[&trip.boarding_stop_id];
        let exit_stop = &ad.gtfs.stops[&trip.get_off_stop_id];

        let mode = get_route_mode(&ad.gtfs, trip);

        // Vary line-width based on how advanced the mode is
        let line_width = match mode {
            "rail" => 4.9,
            "subway" => 4.3,
            "tram" => 3.4,
            _ => 2.6
        };

        feature.set_property("color", route.color.clone());
        feature.set_property("line_width", line_width);
        feature_point.set_property("color", route.color.clone());

        let exit_stop_msg = if has_free_transfer_from_prev {
            format!("{} (stay on vehicle)", exit_stop.name)
        } else {
            exit_stop.name.clone()
        };
        details_list.push(TripDetails::Transit(TripDetailsTransit {
            mode,
            background_color: route.color.clone(),
            text_color: route.text_color.clone(),
            boarding: TripDetailsInner {
                time: trip.boarding_time.0,
                line: route.short_name.clone(),
                stop: boarding_stop.name.clone(),
            },
            exit: TripDetailsInner {
                time: trip.exit_time.0,
                line: route.short_name.clone(),
                stop: exit_stop_msg,
            },
        }));

        if trip.walking_time.0 >= 30.0 {
            assert!(!trip.is_free_transfer);
            details_list.push(TripDetails::Walking(TripDetailsWalking {
                time: trip.walking_time.0,
                length: trip.walking_length_m,
            }))
        }

        has_free_transfer_from_prev = trip.is_free_transfer;
    }

    details_list.reverse();

    features.extend(features_points);
    let geojson = geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let response = json!({
        "details": details_list,
        "path": geojson
    });
    warp::reply::json(&response)
}
