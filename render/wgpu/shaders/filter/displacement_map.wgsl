struct Filter {
    color: vec4<f32>,
    components: u32,  // 00000000 00000000 XXXXXXXX YYYYYYYY
    mode: u32,        // 0 wrap, 1 clamp, 2 ignore, 3 color
    scale_x: f32,
    scale_y: f32,
    source_width: f32,
    source_height: f32,
    map_width: f32,
    map_height: f32,
    offset_x: f32,
    offset_y: f32,
    viewscale_x: f32,
    viewscale_y: f32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var map_texture: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var map_sampler: sampler;
@group(0) @binding(4) var<uniform> filter_args: Filter;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VertexInput {
    /// The position of the vertex in texture space (topleft 0,0, bottomright 1,1)
    @location(0) position: vec2<f32>,

    /// The coordinate of the source texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(1) uv: vec2<f32>,
};

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    // Convert texture space (topleft 0,0 to bottomright 1,1) to render space (topleft -1,1 to bottomright 1,-1)
    let pos = vec4<f32>((in.position.x * 2.0 - 1.0), (1.0 - in.position.y * 2.0), 0.0, 1.0);
    return VertexOutput(pos, in.uv);
}

fn unpack_components(packed_components: u32) -> vec2<u32> {
    return vec2<u32>(packed_components >> 8u, packed_components & 15u);
}

fn get_component(map: vec4<f32>, component: u32) -> f32 {
    switch (component) {
        case 1u: {
            return map.r * 255.0;
        }
        case 2u: {
            return map.g * 255.0;
        }
        case 4u: {
            return map.b * 255.0;
        }
        case 8u: {
            return map.a * 255.0;
        }
        default: {
            return 128.0; // here this means zero displacement
        }
    }
}

fn displace_coordinates(original: vec2<f32>, map: vec4<f32>, components: vec2<u32>, scale: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        original.x + (get_component(map, components.x) - 128.0) * scale.x / 256.0,
        original.y + (get_component(map, components.y) - 128.0) * scale.y / 256.0,
    );
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let source_size = vec2<f32>(filter_args.source_width, filter_args.source_height);
    let map_size = vec2<f32>(filter_args.map_width, filter_args.map_height);

    var source_pos = vec2<f32>(
        in.uv.x * filter_args.source_width,
        in.uv.y * filter_args.source_height,
    );
    var map_uv = vec2<f32>(
        (source_pos.x - filter_args.offset_x) / filter_args.viewscale_x / filter_args.map_width,
        (source_pos.y - filter_args.offset_y) / filter_args.viewscale_y / filter_args.map_height,
    );

    let viewscale = vec2<f32>(filter_args.viewscale_x, filter_args.viewscale_y);

    var map = textureSample(map_texture, map_sampler, map_uv);
    if (map_uv.x < 0.0 || map_uv.x > 1.0 || map_uv.y < 0.0 || map_uv.y > 1.0) {
        map = vec4<f32>(0.5, 0.5, 0.5, 0.5);
    }
    let components = unpack_components(filter_args.components);
    let displaced = displace_coordinates(source_pos, map, components, viewscale * (vec2<f32>(filter_args.scale_x, filter_args.scale_y)));
    var displaced_uv = vec2<f32>(
        displaced.x / filter_args.source_width,
        displaced.y / filter_args.source_height,
    );
    let out_of_bounds = displaced_uv.x < 0.0 || displaced_uv.x > 1.0 || displaced_uv.y < 0.0 || displaced_uv.y > 1.0;

    // 0 wrap is already taken care of by the sampler, we can ignore it here
    if (filter_args.mode == 1u) { // clamp
        displaced_uv = saturate(displaced_uv);
    } else if (filter_args.mode == 2u && out_of_bounds) { // ignore
        displaced_uv = in.uv;
    }
    var result = textureSample(source_texture, source_sampler, displaced_uv);
    if (filter_args.mode == 3u && out_of_bounds) { // color
        // the textureSample can't be conditional, so we need to execute it and throw it away in this case
        result = vec4<f32>(filter_args.color.rgb, 1.0) * filter_args.color.a;
    }
    return result;
}
