use crate::gtfs_wrapper::Stop;
use proj::Proj;

pub static PROJSTRING: &str =
    concat!("+proj=merc +lat_ts=43.765313 +lon_0=-79.649588 +lat_0=43.765313");
thread_local! {
    pub static PROJ: Proj = {
        Proj::new(PROJSTRING).unwrap()
    };
}

pub fn project_stop(stop: &Stop) -> [f64; 2] {
    let lng = stop.longitude.unwrap();
    let lat = stop.latitude.unwrap();
    project_lng_lat(lng, lat)
}

pub fn project_lng_lat(lng: f64, lat: f64) -> [f64; 2] {
    let coord = PROJ.with(|p| {
        p.project((lng.to_radians(), lat.to_radians()), false)
            .unwrap()
    });
    [coord.0, coord.1]
}
//
// pub fn inverse_project_lng_lat(x: f64, y: f64) -> [f64; 2] {
//     let coord = PROJ.with(|p| p.project((x, y), true).unwrap());
//     [coord.1.to_degrees(), coord.0.to_degrees()]
// }
