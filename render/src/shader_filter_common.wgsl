#define_import_path filter

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VertexInput {
    /// The position of the vertex in texture space (topleft 0,0, bottomright 1,1)
    @location(0) position: vec2<f32>,

    /// The coordinate of the texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(1) uv: vec2<f32>,
};

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    // Convert texture space (topleft 0,0 to bottomright 1,1) to render space (topleft -1,1 to bottomright 1,-1)
    let pos = vec4<f32>((in.position.x * 2.0 - 1.0), (1.0 - in.position.y * 2.0), 0.0, 1.0);
    return VertexOutput(pos, in.uv);
}
