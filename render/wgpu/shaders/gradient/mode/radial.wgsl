fn find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    return length(uv * 2.0 - 1.0);
}