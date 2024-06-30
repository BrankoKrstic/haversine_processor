use rand::Rng;

use crate::CoordPair;

fn gen_rand_lat_lon(
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
    rng: &mut impl Rng,
) -> (f64, f64) {
    let lat = rng.gen_range(min_lat..max_lat);
    let lon = rng.gen_range(min_lon..max_lon);
    (lat, lon)
}

pub struct CoordPairGen<T> {
    cur_item: usize,
    item_count: usize,
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
    should_cluster: bool,
    rng: T,
}

impl<T: Rng> CoordPairGen<T> {
    pub fn new(rng: T, should_cluster: bool, item_count: usize) -> Self {
        Self {
            cur_item: 0,
            item_count,
            rng,
            min_lat: -90.0,
            max_lat: 90.0,
            min_lon: -180.0,
            max_lon: 180.0,
            should_cluster,
        }
    }
    fn start_new_cluster(&mut self) {
        let lat_center: f64 = self.rng.gen_range(-90.0..90.0);
        let lon_center: f64 = self.rng.gen_range(-180.0..180.0);
        let lat_radius: f64 = self.rng.gen_range(0.0..90.0);
        let lon_radius: f64 = self.rng.gen_range(0.0..180.0);
        self.min_lat = (lat_center - lat_radius).clamp(-90.0, 90.0);
        self.max_lat = (lat_center + lat_radius).clamp(-90.0, 90.0);
        self.min_lon = (lon_center - lon_radius).clamp(-180.0, 180.0);
        self.max_lon = (lon_center + lon_radius).clamp(-180.0, 180.0);
    }
}

impl<T: Rng> Iterator for CoordPairGen<T> {
    type Item = CoordPair;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item_count < self.cur_item {
            return None;
        }
        if self.should_cluster && self.cur_item % 1000 == 0 {
            self.start_new_cluster();
        }
        self.cur_item += 1;
        Some(
            (
                gen_rand_lat_lon(
                    self.min_lat,
                    self.max_lat,
                    self.min_lon,
                    self.max_lon,
                    &mut self.rng,
                ),
                gen_rand_lat_lon(
                    self.min_lat,
                    self.max_lat,
                    self.min_lon,
                    self.max_lon,
                    &mut self.rng,
                ),
            )
                .into(),
        )
    }
}
