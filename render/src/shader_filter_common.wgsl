/// This is prepended to many shaders at runtime before compiling them.
/// The `filter__` identifier prefix serves as pseudo-namespacing.

struct filter__VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct filter__VertexInput {
    /// The position of the vertex in texture space (topleft 0,0, bottomright 1,1)
    @location(0) position: vec2<f32>,

    /// The coordinate of the texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(1) uv: vec2<f32>,
};

fn filter__main_vertex(in: filter__VertexInput) -> filter__VertexOutput {
    // Convert texture space (topleft 0,0 to bottomright 1,1) to render space (topleft -1,1 to bottomright 1,-1)
    let pos = vec4<f32>((in.position.x * 2.0 - 1.0), (1.0 - in.position.y * 2.0), 0.0, 1.0);
    return filter__VertexOutput(pos, in.uv);
}

// Delegating because the one above is called from other shaders, so it can't be an entry point.
@vertex
fn filter__vertex_entry_point(in: filter__VertexInput) -> filter__VertexOutput {
    return filter__main_vertex(in);
}
