fn find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    let uv = uv * 2.0 - 1.0;
    var d: vec2<f32> = vec2<f32>(focal_point, 0.0) - uv;
    let l = length(d);
    d = d / l;
    return l / (sqrt(1.0 - focal_point * focal_point * d.y * d.y) + focal_point * d.x);
}