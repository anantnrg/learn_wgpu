struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

fn sdBox(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32
{
    let d = abs(p)-b+r;
    return length(max(d,vec2<f32>(0.0, 0.0))) + min(max(d.x,d.y),0.0)-r;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // Define the size and corner radius of the rounded rectangle
    let size = vec2<f32>(0.5, 0.3);
    let cornerRadius = 0.1;

    // Use tex_coords as the point to calculate the signed distance
    let d = sdBox(in.tex_coords - size * 0.5, size, cornerRadius);

    // Apply color based on the signed distance value
    let colorInside = vec4<f32>(1.0, 0.0, 0.0, 1.0); // Inner color of the rectangle
    let colorOutside = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Outside color (background)

    // Apply the color based on the signed distance
    let color = mix(colorOutside, colorInside, smoothstep(-0.05, 0.05, d));

    return color;
}
