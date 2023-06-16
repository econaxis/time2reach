use crate::gtfs_wrapper::FromWithAgencyId;
use crate::{gtfs_wrapper, IdType};
use geo_types::{Coord, LineString};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Archive, Clone)]
pub struct Shape {
    pub id: IdType,
    pub latitude: f64,
    pub longitude: f64,
    pub sequence: usize,
    pub dist_traveled: Option<f32>,
}

impl Shape {
    fn interpolate(start: &Coord, end: &Coord, fraction: f64) -> Coord {
        let x = start.x + (end.x - start.x) * fraction;
        let y = start.y + (end.y - start.y) * fraction;

        Coord { x, y }
    }

    pub fn to_geo_types(v: &[Shape]) -> LineString {
        let coords: Vec<Coord> = v
            .iter()
            .map(|node| Coord {
                x: node.longitude,
                y: node.latitude,
            })
            .collect();
        LineString::new(coords)
    }
    pub fn to_geo_types_interp(v: &[Shape], start: f32, end: f32) -> LineString {
        let start_idx = start.floor() as usize;
        let start_frac = start - start.floor();
        let end_idx = end.ceil() as usize;
        let end_idx = end_idx.max(start_idx);
        let end_frac = end - end.floor();

        let end_is_whole = end.ceil() == end.floor();

        let length = end_idx - start_idx + 1;

        let coords: Vec<Coord> = v[start_idx..=end_idx]
            .iter()
            .map(|node| Coord {
                x: node.longitude,
                y: node.latitude,
            })
            .collect();

        if end_idx - start_idx <= 1 {
            return LineString::new(coords);
        }
        // Handle start case
        let mut linestring = Vec::new();

        let start_coord = Shape::interpolate(&coords[0], &coords[1], start_frac as f64);
        linestring.push(start_coord);

        for i in 1..length - 1 {
            linestring.push(coords[i].clone());
        }

        if end_frac > 0.0 || !end_is_whole {
            let end_coord =
                Shape::interpolate(&coords[length - 2], &coords[length - 1], end_frac as f64);
            linestring.push(end_coord);
        } else {
            linestring.push(coords[length - 1].clone());
        }

        LineString::new(linestring)
    }
}

impl FromWithAgencyId<gtfs_structures::Shape> for Shape {
    fn from_with_agency_id(agency_id: u8, f: gtfs_structures::Shape) -> Self
    where
        Self: Sized,
    {
        Shape {
            id: (agency_id, gtfs_wrapper::try_parse_id(&f.id)),
            latitude: f.latitude,
            longitude: f.longitude,
            sequence: f.sequence,
            dist_traveled: f.dist_traveled,
        }
    }
}
