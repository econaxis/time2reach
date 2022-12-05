use rstar::primitives::GeomWithData;
use rstar::{PointDistance, RTree};
use crate::{ReachData, WALKING_SPEED};

#[derive(Debug, Default)]
pub struct TimeToReachRTree {
    pub(crate) tree: RTree<GeomWithData<[f64; 2], ReachData>>,
}

impl TimeToReachRTree {
    fn serialize_to_json(&self) -> Vec<serde_json::Value> {
        self.tree
            .iter()
            .map(|doc| {
                serde_json::json! ({
                    "point": doc.geom().as_slice(),
                    "data": bson::to_bson(&doc.data).unwrap()
                })
            })
            .collect()
    }
    pub(crate) fn add_observation(&mut self, point: [f64; 2], mut data: ReachData) {
        for near in self.tree.drain_within_distance(point, 10.0 * 10.0) {
            let time_to_walk_here =
                (near.distance_2(&point) / WALKING_SPEED) as u32 + near.data.timestamp;
            if time_to_walk_here < data.timestamp {
                data.timestamp = time_to_walk_here;
            }
        }

        self.tree.insert(GeomWithData::new(point, data));
    }

    pub(crate) fn sample_fastest_time(&self, point: [f64; 2]) -> Option<u32> {
        self.sample_fastest_time_within_distance(point, 650.0)
            .or_else(|| self.sample_fastest_time_within_distance(point, 1000.0))
    }

    fn sample_fastest_time_within_distance(&self, point: [f64; 2], distance: f64) -> Option<u32> {
        let best_time1 = self
            .tree
            .locate_within_distance(point, distance * distance)
            .map(|obs| {
                let distance = obs.distance_2(&point).sqrt();

                let mut penalty = 0;
                if distance > 90.0 {
                    penalty = (0.7 * (distance - 80.0).sqrt()) as u32;
                }

                let time_to_reach = (distance / WALKING_SPEED) as u32
                    + obs.data.timestamp
                    + penalty
                    // Penalize time for every transfer performed
                    + obs.data.transfers as u32 * 25;
                (time_to_reach, obs)
            })
            .min_by_key(|(time, _obs)| {
                *time
            });

        best_time1.map(|a| a.0)
    }
}
