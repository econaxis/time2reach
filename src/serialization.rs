use crate::TimeToReachRTree;
use serde::Serialize;
use serde_bytes::ByteBuf;

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
    start_coord: [f64; 2],
    end_coord: [f64; 2],
    pub(crate) x_samples: usize,
    pub(crate) y_samples: usize,
    pub(crate) map: Vec<i32>,
}

impl TimeGrid {
    fn at(&mut self, x: usize, y: usize) -> &mut i32 {
        &mut self.map[y * self.x_samples + x]
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
        map.resize(total_cells, 0);

        Self {
            start_coord,
            end_coord,
            x_samples,
            y_samples,
            map,
        }
    }

    pub(crate) fn process(&mut self, data: &TimeToReachRTree) {
        let x_iter = (self.end_coord[0] - self.start_coord[0]) / self.x_samples as f64;
        let y_iter = (self.end_coord[1] - self.start_coord[1]) / self.y_samples as f64;

        for x in 0..self.x_samples {
            for y in 0..self.y_samples {
                let xcoord = x as f64 * x_iter + self.start_coord[0];
                let ycoord = y as f64 * y_iter + self.start_coord[1];

                let time = data.sample_fastest_time([xcoord, ycoord]);
                *self.at(x as usize, y as usize) = time.map(|a| a as i32).unwrap_or(-1);
            }
        }
    }
}
