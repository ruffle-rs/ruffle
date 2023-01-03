fn normalize_t(t: f32) -> f32 {
    var result: f32 = t;
    if( result < 0.0 )
    {
        result = -t;
    }
    if( (i32(result) & 1) == 0 ) {
        result = fract(result);
    } else {
        result = 1.0 - fract(result);
    }
    return result;
}