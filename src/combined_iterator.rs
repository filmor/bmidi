use crate::types::Event;
use std::collections::BinaryHeap;

struct IterState<'a> {
    next_event: Event,
    iter: Box<dyn Iterator<Item = Event> + 'a>,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
struct HeapItem {
    time: usize,
    track: usize,
}

pub struct CombinedIterator<'a> {
    states: Vec<IterState<'a>>,
    heap: BinaryHeap<HeapItem>,
}

pub struct TrackEvent {
    pub track: usize,
    pub event: Event,
}

impl<'a> CombinedIterator<'a> {
    pub fn new(iters: Vec<Box<dyn Iterator<Item = Event> + 'a>>) -> Self {
        let mut states = Vec::with_capacity(iters.len());
        let mut heap = BinaryHeap::with_capacity(iters.len());

        let mut n = 0;

        for mut iter in iters {
            if let Some(event) = iter.next() {
                states.push(IterState {
                    iter,
                    next_event: event,
                });
                heap.push(HeapItem { track: n, time: 0 });
                n += 1;
            }
        }

        CombinedIterator { states, heap }
    }
}

impl Iterator for CombinedIterator<'_> {
    type Item = TrackEvent;

    fn next(&mut self) -> Option<TrackEvent> {
        if let Some(item) = self.heap.pop() {
            let state = &mut self.states[item.track];

            let result = state.next_event.clone();

            if let Some(event) = state.iter.next() {
                let new_time = event.delay + result.delay;
                state.next_event = event;

                let new_item = HeapItem {
                    time: new_time as usize,
                    track: item.track,
                };

                self.heap.push(new_item);
            }

            Some(TrackEvent {
                track: item.track,
                event: result,
            })
        } else {
            None
        }
    }
}
