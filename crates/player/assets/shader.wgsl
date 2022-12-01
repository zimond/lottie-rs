// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

struct GradientStop {
    offset: f32,
    color: vec4<f32>,
};

struct GradientInfo {
    start_pos: vec2<f32>,
    end_pos: vec2<f32>,
    use_gradient: u32,
    stops: array<GradientStop, 2>
};

struct MaskInfo {
    masks: array<vec4<u32>, 4>,
    mask_count: u32,
    mask_total_count: u32
}

@group(1) @binding(0)
var mask: texture_2d<f32>;

@group(1) @binding(1)
var mask_sampler: sampler;

@group(1) @binding(2)
var<uniform> scene_size: vec4<f32>;

@group(1) @binding(3)
var<uniform> mask_info: MaskInfo;

@group(1) @binding(4)
var<uniform> gradient: GradientInfo;

@group(2) @binding(0)
var<uniform> mesh: Mesh2d;

// NOTE: Bindings must come before functions that use them!
#import bevy_sprite::mesh2d_functions

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: u32,
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    @builtin(position) clip_position: vec4<f32>,
    // We pass the vertex color to the fragment shader in location 0
    @location(0) color: vec4<f32>,
};


/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    // out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 0.0, 1.0);
    out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(vertex.position.xy, 0.0, 1.0));
    // Unpack the `u32` from the vertex buffer into the `vec4<f32>` used by the fragment shader
    out.color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    return out;
}

// https://dawn.googlesource.com/tint/+/refs/heads/chromium/4846/test/benchmark/skinned-shadowed-pbr-fragment.wgsl.expected.wgsl
let GAMMA = 2.200000048;

fn linearTosRGB(color: vec3<f32>) -> vec3<f32> {
    let INV_GAMMA = (1.0 / GAMMA);
    return pow(color, vec3(INV_GAMMA));
}

fn sRGBToLinear(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(GAMMA));
}

// https://docs.rs/lyon_geom/latest/src/lyon_geom/line.rs.html#650-655
fn point_projection(pt: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    let v = end - start;
    var a = -v.y;
    var b = v.x;
    var c = -(a * start.x + b * start.y);
    let div = 1.0 / length(vec2(a, b));
    a = a * div;
    b = b * div;
    c = c * div;
    let x = b * (b * pt.x - a * pt.y) - a * c;
    let y = a * (a * pt.y - b * pt.x) - b * c;
    return vec2(x, y);
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    @location(0) color: vec4<f32>,
};

/// Entry point for the fragment shader
@fragment
fn fragment(@builtin(position) position: vec4<f32>, in: FragmentInput) -> @location(0) vec4<f32> {
    var out: vec4<f32>;
    let scale = scene_size.z;
    let pos = position.xy / scale;
    if gradient.use_gradient == 1u {
        var start = (mesh.model * vec4<f32>(gradient.start_pos, 0.0, 1.0)).xy;
        let end = (mesh.model * vec4<f32>(gradient.end_pos, 0.0, 1.0)).xy;
        let proj = point_projection(pos.xy, start, end);
        let inv = smoothstep(start, end, proj);
        var t = inv.x;
        if end.x == start.x {
            t = inv.y;
        }
        let color = mix(gradient.stops[0].color.xyz, gradient.stops[1].color.xyz, t);
        out = vec4(color, 1.0);
    } else {
        out = in.color;
    }
    let mask_size = vec2<f32>(textureDimensions(mask));
    let count = mask_info.mask_count;
    for (var i: u32 = 0u; i < count; i++) {
        let info = mask_info.masks[i];
        let mask_index = f32(info.x);
        let mask_count = f32(mask_info.mask_total_count);
        let stride = vec2(mask_size.x / mask_count, 0.0);
        let sample_pos = (pos.xy + stride * mask_index) / mask_size;
        var mask_pixel = textureSample(mask, mask_sampler, sample_pos);
        if info.y == 2u {
            out.a = 1.0 - mask_pixel.a;
        } else {
            out.a = mask_pixel.a;
        }
    }
    // if (mask_info.z != 0u) {
    //     out = mask_pixel;
    // }
    return out;
}