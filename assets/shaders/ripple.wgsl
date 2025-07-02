struct RippleSettings {
    time: f32,
    intensity: f32,
};

@group(1) @binding(0)
var color_texture: texture_2d<f32>;
@group(1) @binding(1)
var color_sampler: sampler;
@group(1) @binding(2)
var<uniform> settings: RippleSettings;

@fragment
fn fragment(in: Material2dFragmentInput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let center = vec2(0.5, 0.5);
    let dir = uv - center;
    let dist = length(dir);
    let ripple = sin(dist * 40.0 - settings.time * 6.0) * settings.intensity;
    uv += normalize(dir) * ripple * 0.05;
    return textureSample(color_texture, color_sampler, uv);
}
