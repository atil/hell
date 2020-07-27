pub fn in_between(f: f32, f_low: f32, f_hi: f32) -> bool {
    f_low <= f && f <= f_hi
}

pub fn slice_excess(f: f32) -> f32 {
    (f * 100.0).round() / 100.0
}
