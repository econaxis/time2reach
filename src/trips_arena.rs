use crate::InProgressTrip;
use id_arena::{Arena, Id};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct TripsArena {
    explore_queue: VecDeque<id_arena::Id<InProgressTrip>>,
    arena: Arena<InProgressTrip>,
}

impl TripsArena {
    pub(crate) fn add_to_explore(&mut self, item: InProgressTrip) {
        let id = self.arena.alloc(item);
        self.explore_queue.push_back(id);
    }

    pub(crate) fn get_by_id(&self, id: id_arena::Id<InProgressTrip>) -> &InProgressTrip {
        &self.arena[id]
    }
    pub(crate) fn pop_front(&mut self) -> Option<(InProgressTrip, Id<InProgressTrip>)> {
        let id = self.explore_queue.pop_front()?;
        Some((self.get_by_id(id).clone(), id))
    }
}
