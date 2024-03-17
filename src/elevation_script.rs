// Add to Cargo.toml:
// reqwest = { version = "0.11", features = ["json", "blocking"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// gdal = "0.9"
// rusqlite = "0.26"
// proj = "0.23.1"

use std::sync::Mutex;

use gdal::Dataset;
use proj::Proj;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::{error::Error, fs, io::Write, path::Path};
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroUsize;
use gdal::raster::{Buffer, RasterBand};
use lazy_static::lazy_static;
use lru::LruCache;
// Assuming you're okay with using global mutable state and you've ensured thread safety,
// you can wrap the LRU cache in a Mutex and use lazy_static! to initialize it.
lazy_static! {
    static ref CACHE: Mutex<LruCache<String, String>> = Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())); // Cache up to 100 files

}

const IMAGE_SIZE: i32 = 2048;
// Example value, adjust as needed
const PIXEL_SIZE: i32 = 10;   // Example value, adjust as needed

fn download_geo_tiff(bbox: &str) -> Result<String, Box<dyn Error>> {
    let formatted_bbox = bbox.replace(",", "_");
    let file_name = format!("target/geotiff/geotiff_{}_{}_{}.tif", formatted_bbox, IMAGE_SIZE, PIXEL_SIZE);

    // Check if the file already exists
    if Path::new(&file_name).exists() {
        return Ok(file_name);
    }

    let file_name = format!("geotiff_{}.tif", bbox.replace(",", "_"));

    {
        let mut cache = CACHE.lock().unwrap(); // Acquire lock on cache
        if let Some(cached_file_name) = cache.get(bbox) {
            if Path::new(cached_file_name).exists() {
                println!("Using cached file for bbox: {}", bbox);
                return Ok(cached_file_name.clone());
            }
        }
    } // Release lock to allow cache update after checking

    let api_url: &str = "https://elevation.nationalmap.gov/arcgis/rest/services/3DEPElevation/ImageServer/exportImage";
    let params = [
        ("bbox", bbox),
        ("size", &format!("{},{}", 2048, 2048)),
        ("format", "tiff"),
        ("pixelType", "F32"),
        ("noData", ""),
        ("noDataInterpretation", "esriNoDataMatchAny"),
        ("interpolation", "RSP_BilinearInterpolation"),
        ("adjustAspectRatio", "true"),
        ("lercVersion", "1"),
        ("f", "image"),
    ];

    let client = Client::new();
    let res = client.get(api_url).query(&params).send()?;
    fs::write(&file_name, res.bytes()?)?;
    println!("Downloaded GeoTIFF file for bbox: {}", bbox);

    // Update cache with new file name
    let mut cache = CACHE.lock().unwrap();
    cache.put(bbox.to_string(), file_name.clone());

    Ok(file_name)
}

use core::cell::RefCell;
thread_local! {
    static PROJ: RefCell<Proj> = RefCell::new(Proj::new_known_crs("EPSG:4326", "EPSG:3857", None).unwrap());
}
fn convert_lat_lon_to_epsg3857(lat: f64, lon: f64) -> Result<(f64, f64), Box<dyn Error>> {
    PROJ.with(|proj| {
        proj.borrow().convert((lon, lat)).map_err(Into::into)
    })
}

fn calculate_bounding_box(lat: f64, lon: f64, image_size: i32, pixel_size: i32) -> (f64, f64, f64, f64) {
    let (x, y) = convert_lat_lon_to_epsg3857(lat, lon).unwrap();
    let (rounded_x, rounded_y) = round_to_tile_coordinates(x, y, image_size, pixel_size);
    let full_width = (image_size * pixel_size) as f64;
    let min_x = rounded_x;
    let min_y = rounded_y;
    let max_x = rounded_x + full_width;
    let max_y = rounded_y + full_width;
    (min_x, min_y, max_x, max_y)
}

fn round_to_tile_coordinates(x: f64, y: f64, image_size: i32, pixel_size: i32) -> (f64, f64) {
    let round_factor = (image_size * pixel_size) as f64;
    let rounded_x = (x / round_factor).floor() * round_factor;
    let rounded_y = (y / round_factor).floor() * round_factor;
    (rounded_x, rounded_y)
}

fn open_dataset(file_name: &str) -> Result<Dataset, Box<dyn std::error::Error>> {
    Dataset::open(Path::new(file_name)).map_err(|e| e.into())
}

fn extract_elevation_from_geotiff(dataset: &Dataset, x_coord: f64, y_coord: f64) -> Result<f64, Box<dyn std::error::Error>> {
    let geo_transform = dataset.geo_transform()?;
    let (pixel_x, pixel_y) = (
        ((x_coord - geo_transform[0]) / geo_transform[1]).floor() as isize,
        ((y_coord - geo_transform[3]) / geo_transform[5]).floor() as isize,
    );

    let band: RasterBand = dataset.rasterband(1)?;
    let buffer: Buffer<f64> = band.read_as((pixel_x, pixel_y), (1, 1), (1, 1), None)?;

    // Assuming the buffer contains at least one element and returning it
    buffer.data.get(0).copied().ok_or_else(|| "Failed to read elevation data".into())
}

fn create_db_and_tables(db_name: &str) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(db_name).context("Connect to db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS nodes (
            node_id INTEGER PRIMARY KEY,
            lat REAL,
            lon REAL,
            ele REAL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS edges (
            id INTEGER PRIMARY KEY,
            nodeA INTEGER,
            nodeB INTEGER,
            dist REAL,
            kvs TEXT,
            FOREIGN KEY(nodeA) REFERENCES nodes(node_id),
            FOREIGN KEY(nodeB) REFERENCES nodes(node_id)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS edge_points (
            point_id INTEGER PRIMARY KEY AUTOINCREMENT,
            edge_id INTEGER,
            lat REAL,
            lon REAL,
            ele REAL,
            FOREIGN KEY(edge_id) REFERENCES edges(id)
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edge_points_on_edge_id_and_point_id ON edge_points (edge_id, point_id);",
        [],
    )?;
    Ok(())
}


struct DatasetCache {
    cache: HashMap<String, Dataset>,
}

impl DatasetCache {
    fn new() -> DatasetCache {
        DatasetCache {
            cache: HashMap::new(),
        }
    }

    fn open_dataset(&mut self, file_name: &str) -> Result<&Dataset, Box<dyn Error>> {
        if !self.cache.contains_key(file_name) {
            let dataset = Dataset::open(Path::new(file_name)).map_err(|e| e.to_string())?;
            self.cache.insert(file_name.to_string(), dataset);
        }

        self.cache.get(file_name).ok_or_else(|| "Failed to get dataset from cache".into())
    }
}

fn get_ele(lat: f64, lon: f64, default: Option<f64>, cache: &mut DatasetCache) -> Result<f64, Box<dyn Error>> {
    let (x, y) = convert_lat_lon_to_epsg3857(lat, lon)?;
    let bbox = calculate_bounding_box(lat, lon, 2048, 10);
    let file_name = download_geo_tiff(&format!("{},{},{},{}", bbox.0, bbox.1, bbox.2, bbox.3))?;
    let dataset = cache.open_dataset(&file_name)?;
    let elevation = extract_elevation_from_geotiff(dataset, x, y)?;

    if elevation <= 0.25 {
        return Ok(default.unwrap_or(0.0));
    }
    Ok(elevation)
}

const CITIES: &[(f64, f64)] = &[
    (34.0522, -118.2437), // Los Angeles
    (32.7157, -117.1611), // San Diego
    (37.3382, -121.8863), // San Jose
    (37.7749, -122.4194), // San Francisco
    (36.7378, -119.7871), // Fresno
    (38.5816, -121.4944), // Sacramento
    (33.7701, -118.1937), // Long Beach
    (37.8044, -122.2712), // Oakland
    (35.3733, -119.0187), // Bakersfield
    (33.8366, -117.9143), // Anaheim
];

fn is_within_city(lat: f64, lng: f64) -> bool {
    let miles_per_degree = 69.0;
    let threshold = 75.0; // 75 miles threshold

    for &(city_lat, city_lng) in CITIES.iter() {
        let delta_lat = (city_lat - lat).abs();
        let delta_lng = (city_lng - lng).abs();
        let distance_approx = ((delta_lat.powi(2) + delta_lng.powi(2)).sqrt()) * miles_per_degree;

        if distance_approx <= threshold {
            return true;
        }
    }

    false
}

fn add_elevation_to_db(dbname: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = Connection::open(dbname)?;
    let mut cache = DatasetCache::new();
    let tx = conn.transaction()?;

    let mut stmt_update_node = tx.prepare("UPDATE nodes SET ele = ? WHERE node_id = ?")?;
    let mut stmt_query_nodes = tx.prepare("SELECT node_id, lat, lon FROM nodes")?;
    let mut stmt_query_edge_points = tx.prepare("SELECT point_id, lat, lon FROM edge_points")?;
    let mut stmt_update_edge_point = tx.prepare("UPDATE edge_points SET ele = ? WHERE point_id = ?")?;

    let nodes_iter = stmt_query_nodes.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
    })?;

    for node_result in nodes_iter {
        let (node_id, lat, lon) = node_result?;
        if is_within_city(lat, lon) {
            let ele = get_ele(lat, lon, None, &mut cache)?;
            stmt_update_node.execute(params![ele, node_id])?;
        }
    }

    let edge_points_iter = stmt_query_edge_points.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
    })?;

    for edge_point_result in edge_points_iter {
        let (point_id, lat, lon) = edge_point_result?;
        if is_within_city(lat, lon) {
            let ele = get_ele(lat, lon, None, &mut cache)?;
            stmt_update_edge_point.execute(params![ele, point_id])?;
        }
    }

    std::mem::drop(stmt_update_node);
    std::mem::drop(stmt_query_nodes);
    std::mem::drop(stmt_query_edge_points);
    std::mem::drop(stmt_update_edge_point);

    tx.commit()?;
    Ok(())
}


pub fn main1() -> Result<(), Box<dyn Error>> {
    let db_name = "/Users/henry/timetoreach/california-big.db";
    create_db_and_tables(db_name)?;

    add_elevation_to_db(db_name)?;

    Ok(())
}


