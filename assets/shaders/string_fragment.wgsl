@group(2) @binding(0)
var<uniform> left_data: array<vec4<f32>, 16>;

@group(2) @binding(1)
var<uniform> right_data: array<vec4<f32>, 16>;

@group(2) @binding(2)
var<uniform> viewport_width: f32;

@group(2) @binding(3)
var<uniform> viewport_height: f32;

@group(2) @binding(4)
var<uniform> monochrome: u32;

@group(2) @binding(5)
var<uniform> colors: array<vec4<f32>, 4>;

// Removed globals struct and binding since it doesn't appear to be used in this shader
// struct Globals {
//     time: f32,
//     delta_time: f32,
//     frame_count: u32,
// #ifdef SIXTEEN_BYTE_ALIGNMENT
//     _wasm_padding: f32
// #endif
// }

// @group(2) @binding(6)
// var<uniform> globals: Globals;

// Corrected constants without division
const NUM_CIRCLES: i32 = 128;
const NUM_CIRCLES_F32: f32 = 128.0;
const SECTION_WIDTH: f32 = 0.0078125; // Precomputed value of 1.0 / 128.0
const NUM_CIRCLES_HALF: i32 = 64;     // Precomputed value of 128 / 2

fn value_to_monochrome(value: f32) -> vec4<f32> {
    return vec4<f32>(value, value, value, 1.0);
}

fn value_to_color(value: f32) -> vec4<f32> {
    let start_color = colors[0];
    let middle_color = colors[1];
    let end_color = colors[2];

    let t = value * 2.0;
    let color = mix(
        mix(start_color, middle_color, clamp(t, 0.0, 1.0)),
        mix(middle_color, end_color, clamp(t - 1.0, 0.0, 1.0)),
        step(1.0, t)
    );
    return color;
}

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let aspect_ratio = viewport_width / viewport_height;
    let inv_aspect_ratio = 1.0 / aspect_ratio;

    // Correct the UV coordinates for the aspect ratio
    let uv_corrected = vec2<f32>(uv.x, -uv.y * inv_aspect_ratio);

    // Calculate the center of the circle
    let section = floor(uv.x / SECTION_WIDTH);
    let section_center_x = (section + 0.5) * SECTION_WIDTH;

    // Calculate the index for the audio data based on the x-coordinate
    let x_offset = abs(uv_corrected.x - 0.5) * 2.0;
    let index = clamp(i32(x_offset * f32(NUM_CIRCLES_HALF)), 0, 63);

    let component_index = index % 4;
    let array_index = index / 4;

    // Retrieve the current audio value
    let is_right = step(0.5, uv_corrected.x);
    let audio_value = mix(
        left_data[array_index][component_index],
        right_data[array_index][component_index],
        is_right
    );

    // Define circle parameters
    let scaled_audio_value = -(audio_value / 5.0) + 0.6;
    let diameter = ((audio_value * 0.8) + 0.2) * SECTION_WIDTH;
    let radius = diameter * 0.5;

    let circle_center = vec2<f32>(section_center_x, scaled_audio_value * inv_aspect_ratio);

    // Calculate distance from the pixel to the circle's center
    let dist_to_center = distance(vec2<f32>(uv_corrected.x, -uv_corrected.y), circle_center);

    // Determine if the pixel is inside the circle
    let is_inside = select(0.0, 1.0, dist_to_center < radius);

    // Compute the color
    let color_mono = value_to_monochrome(audio_value);
    let color_color = value_to_color(audio_value);
    let is_monochrome = f32(monochrome);
    let color = mix(color_color, color_mono, is_monochrome);

    // Return the final color based on whether the pixel is inside the circle
    return mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), color, is_inside);
}
