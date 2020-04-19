pub fn in_between(f: f32, f_low: f32, f_hi: f32) -> bool {
    f_low <= f && f <= f_hi
}

pub fn slice_excess(f: f32) -> f32 {
    (f * 100.0).round() / 100.0
}

pub fn approx(f1: f32, f2: f32) -> bool {
    (f1 - f2).abs() < 0.001
}
