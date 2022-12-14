use std::cmp::min;
use crate::{project_lng_lat, TimeToReachRTree, WALKING_SPEED};
use serde::Serialize;
use serde_bytes::ByteBuf;
use crate::projection::inverse_project_lng_lat;
use crate::time_to_reach::calculate_score;

#[derive(Serialize)]
pub struct MapSerialize {
    pub(crate) map: ByteBuf,
    pub(crate) x: usize,
    pub(crate) y: usize,
}

pub unsafe fn to_bytebuf(v: Vec<i32>) -> ByteBuf {
    let (ptr, len, cap) = v.into_raw_parts();
    let len = len * 4;
    let cap = cap * 4;
    let ptr = ptr as *mut u8;
    let v1 = Vec::from_raw_parts(ptr, len, cap);
    ByteBuf::from(v1)
}

pub struct TimeGrid {
    pub start_coord: [f64; 2],
    pub end_coord: [f64; 2],
    pub(crate) x_samples: usize,
    pub(crate) y_samples: usize,
    pub(crate) map: Vec<i32>,
}

impl TimeGrid {
    pub fn test() -> Self {
        let mut m = Vec::new();

        for i in 0..1000 * 1000 {
            m.push(15 as i32);
        }

        let clarkson = project_lng_lat(-79.65209, 43.56631);
        let markham = project_lng_lat(-79.95375, 43.85612);
        dbg!(clarkson, markham);
        Self {
            start_coord: clarkson,
            end_coord: markham,
            x_samples: 1000,
            y_samples: 1000,
            map: m
        }
    }
    pub fn calculate_x_scale(&self) -> f64 {
        (self.end_coord[0] - self.start_coord[0]) / self.x_samples as f64
    }
    pub fn calculate_y_scale(&self) -> f64 {
        (self.end_coord[1] - self.start_coord[1]) / self.y_samples as f64
    }
    fn at(&mut self, x: usize, y: usize) -> &mut i32 {
        if y * self.x_samples + x >= self.map.len() {
            return &mut self.map[0];
        }
        &mut self.map[y * self.x_samples + x]
    }

    fn set_if_lower<F: Fn() -> i32>(&mut self, x: usize, y: usize, minimum_bound: i32, value: F) {
        let old_value = self.at(x, y);
        if *old_value == -1 || *old_value > minimum_bound {
            let value = value();
            if *old_value > value || *old_value == -1 {
                *old_value = value;
            }
        }
    }

    fn get_bounding_box(data: &TimeToReachRTree) -> ([f64; 2], [f64; 2]) {
        let mut start = [f64::MAX, f64::MAX];
        let mut end = [f64::MIN, f64::MIN];
        for point in data.tree.iter() {
            let [x, y] = *point.geom();
            start[0] = start[0].min(x);
            start[1] = start[1].min(y);
            end[0] = end[0].max(x);
            end[1] = end[1].max(y);
        }
        (start, end)
    }
    pub(crate) fn new(data: &TimeToReachRTree, x_samples: usize, y_samples: usize) -> Self {
        let (start_coord, end_coord) = Self::get_bounding_box(data);

        let total_cells = x_samples * y_samples;
        let mut map = Vec::with_capacity(total_cells);
        map.resize(total_cells, -1);

        Self {
            start_coord,
            end_coord,
            x_samples,
            y_samples,
            map,
        }
    }

    fn mark_circle<F: Fn(usize, usize) -> i32>(&mut self, point: [usize; 2], square_size: usize, value_function: F) {
        let minimum_bound = value_function(point[0], point[1]);
        for x in point[0]..point[0] + 2 * square_size {
            if let Some(real_x) = x.checked_sub(square_size) {
                for y in point[1]..point[1] + 2 * square_size {
                    if let Some(real_y) = y.checked_sub(square_size) {
                        self.set_if_lower(real_x, real_y,minimum_bound, || value_function(real_x, real_y));
                    }
                }
            }
        }
    }

    #[inline(never)]
    pub(crate) fn process(&mut self, data: &TimeToReachRTree) {
        // Size of each x or y coordinate in meters
        let start_coord = self.start_coord;

        let x_iter = (self.end_coord[0] - self.start_coord[0]) / self.x_samples as f64;
        let y_iter = (self.end_coord[1] - self.start_coord[1]) / self.y_samples as f64;

        for elem in data.tree.iter() {
            let point = elem.geom();
            let xindex = ((point[0] - self.start_coord[0]) / x_iter).floor() as usize;
            let yindex = ((point[1] - self.start_coord[1]) / y_iter).floor() as usize;

            self.mark_circle([xindex, yindex], 40, |x, y| {
                let evaluated_point = [(x as f64 + 0.5) * x_iter + start_coord[0], (y as f64 + 0.5) * y_iter + start_coord[1]];
                // let xdiff = x.abs_diff(xindex)  as f64 * x_iter;
                // let ydiff = y.abs_diff(y_iter)  as f64 * y_iter;
                let score = calculate_score(&evaluated_point, elem);
                score as i32

                // ((xdiff * xdiff + ydiff * ydiff).sqrt() / WALKING_SPEED) as i32 + elem.data.timestamp as i32
            })
            // *self.at(xindex, yindex) = elem.data.timestamp as i32;
        }

        // for x in 0..self.x_samples {
        //     for y in 0..self.y_samples {
        //         let xcoord = x as f64 * x_iter + self.start_coord[0];
        //         let ycoord = y as f64 * y_iter + self.start_coord[1];
        //
        //         let time = data.sample_fastest_time([xcoord, ycoord]);
        //         *self.at(x as usize, y as usize) = time.map(|a|{
        //             255 - ((255 * (a - time_range[0])) / (time_range[1] - time_range[0])) as i32
        //         }).unwrap_or(-1);
        //     }
        // }
    }
}
