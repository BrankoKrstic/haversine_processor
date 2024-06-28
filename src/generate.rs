use std::cell::RefCell;

use rand::Rng;
use serde::{ser::SerializeSeq, Serialize};

use crate::CoordPair;

fn gen_rand_lat_lng(
    min_lat: f64,
    max_lat: f64,
    min_lng: f64,
    max_lng: f64,
    rng: &mut impl Rng,
) -> (f64, f64) {
    let lat = rng.gen_range(min_lat..max_lat);
    let lng = rng.gen_range(min_lng..max_lng);
    (lat, lng)
}

struct CoordPairGen<T> {
    cur_item: usize,
    item_count: usize,
    min_lat: f64,
    max_lat: f64,
    min_lng: f64,
    max_lng: f64,
    should_cluster: bool,
    rng: T,
}

impl<T: Rng> CoordPairGen<T> {
    fn new(rng: T, should_cluster: bool, item_count: usize) -> Self {
        Self {
            cur_item: 0,
            item_count,
            rng,
            min_lat: -90.0,
            max_lat: 90.0,
            min_lng: -180.0,
            max_lng: 180.0,
            should_cluster,
        }
    }
}

impl<T: Rng> Iterator for CoordPairGen<T> {
    type Item = CoordPair;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item_count < self.cur_item {
            return None;
        }
        if self.should_cluster && self.cur_item % 1000 == 0 {
            todo!();
        }
        self.cur_item += 1;
        Some(
            (
                gen_rand_lat_lng(
                    self.min_lat,
                    self.max_lat,
                    self.min_lng,
                    self.max_lng,
                    &mut self.rng,
                ),
                gen_rand_lat_lng(
                    self.min_lat,
                    self.max_lat,
                    self.min_lng,
                    self.max_lng,
                    &mut self.rng,
                ),
            )
                .into(),
        )
    }
}

pub struct CoordSerializer<T> {
    inner: RefCell<CoordPairGen<T>>,
}

impl<T: Rng> CoordSerializer<T> {
    pub fn new(rng: T, should_cluster: bool, item_count: usize) -> Self {
        Self {
            inner: RefCell::new(CoordPairGen::new(rng, should_cluster, item_count)),
        }
    }
}

impl<T: Rng> Serialize for CoordSerializer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut iter = self.inner.borrow_mut();
        let mut seq = serializer.serialize_seq(Some(iter.item_count - iter.cur_item))?;
        for item in iter.by_ref() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}
