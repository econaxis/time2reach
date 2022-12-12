use crate::{ReachData, WALKING_SPEED};
use rstar::primitives::GeomWithData;
use rstar::{PointDistance, RTree};

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
        for near in self.tree.drain_within_distance(point, 15.0 * 15.0) {
            let time_to_walk_here =
                (near.distance_2(&point) / WALKING_SPEED) as u32 + near.data.timestamp;
            if time_to_walk_here < data.timestamp {
                data.timestamp = time_to_walk_here;
            }
        }

        self.tree.insert(GeomWithData::new(point, data));
    }

    pub(crate) fn sample_fastest_time(&self, point: [f64; 2]) -> Option<u32> {
        self.sample_fastest_time_within_distance(point, 1000.0)
            .or_else(|| self.sample_fastest_time_within_distance(point, 1500.0))
    }

    fn sample_fastest_time_within_distance(&self, point: [f64; 2], distance: f64) -> Option<u32> {
        let best_time1 = self
            .tree
            .locate_within_distance(point, distance * distance)
            .map(|obs| {
                let distance = obs.distance_2(&point).sqrt();

                let time_to_reach = distance / WALKING_SPEED;
                let mut penalty = 0.0;
                if time_to_reach >= 60.0 {
                    penalty += 1.0 * time_to_reach;
                } else {
                    penalty += (time_to_reach - 60.0) * 5.0;
                }

                if time_to_reach >= 2.0 * 60.0 {
                    penalty += 0.8 * (time_to_reach - 2.0 * 60.0);
                }


                if obs.data.transfers >= 2 {
                    // Penalize time for every transfer performed
                    penalty += (obs.data.transfers as u32 - 1) as f64 * 20.0
                }

                let mut time_to_reach = time_to_reach
                    + obs.data.timestamp as f64
                    + penalty;
                let mut time_to_reach = time_to_reach as u32;
                if time_to_reach < obs.data.timestamp {
                    time_to_reach = obs.data.timestamp;
                }
                (time_to_reach as u32, obs)
            })
            .min_by_key(|(time, _obs)| *time);

        best_time1.map(|a| a.0)
    }
}
