use std::sync::Arc;

use crate::agencies::City;
use crate::cache_function::CacheDecorator;
use gtfs_structure_2::gtfs_wrapper::Stop;
use proj::Proj;

const PRECISION: f64 = 100000.0;

fn get_proj_instance_inner(lat_u: i64, lon_u: i64) -> Proj {
    let lat = lat_u as f64 / PRECISION;
    let lon = lon_u as f64 / PRECISION;
    log::warn!(
        "Thread {:?} created new PROJ instance! {} {}",
        std::thread::current().id(),
        lat_u,
        lon_u
    );
    Proj::new(&format!("+proj=aeqd +lon_0={} +lat_0={}", lon, lat)).unwrap()
}
thread_local! {
    static PROJ_CACHE: CacheDecorator<fn(i64, i64) -> Proj, Proj> = CacheDecorator::new(get_proj_instance_inner, 20);
}

fn get_proj_instance(c: &CacheDecorator<fn(i64, i64) -> Proj, Proj>, city: &City) -> Arc<Proj> {
    let [lat, lon] = city.get_city_center();
    c.call((lat * PRECISION) as i64, (lon * PRECISION) as i64)
}

pub fn get_proj_defn(city: &City) -> String {
    PROJ_CACHE.with(|c| get_proj_instance(c, city).def().unwrap())
}

pub fn project_stop(city: &City, stop: &Stop) -> [f64; 2] {
    let lng = stop.longitude.unwrap();
    let lat = stop.latitude.unwrap();
    project_lng_lat(city, lng, lat)
}

pub fn project_lng_lat(city: &City, lng: f64, lat: f64) -> [f64; 2] {
    let coord = PROJ_CACHE.with(|c| {
        get_proj_instance(c, city)
            .project((lng.to_radians(), lat.to_radians()), false)
            .unwrap()
    });
    [coord.0, coord.1]
}
