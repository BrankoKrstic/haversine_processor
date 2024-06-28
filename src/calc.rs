use crate::CoordPair;

static EARTH_RADIUS: f64 = 6372.8;

pub fn naive_haversine(cp: CoordPair) -> f64 {
    let d_lat = (cp.lat1 - cp.lat0).to_radians();
    let d_lng = (cp.lng1 - cp.lng0).to_radians();
    let lat0 = cp.lat0.to_radians();
    let lat1 = cp.lat1.to_radians();

    let a = (d_lat / 2.0).sin().powf(2.0) + lat0.cos() * lat1.cos() * (d_lng / 2.0).sin().powf(2.0);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS * c
}
