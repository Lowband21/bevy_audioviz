@group(2) @binding(0)
var<uniform> normalized_data: array<vec4<f32>, 16>;

@group(2) @binding(1)
var<uniform> viewport_width: f32;

@group(2) @binding(2)
var<uniform> viewport_height: f32;

@group(2) @binding(3)
var<uniform> monochrome: u32;

@group(2) @binding(4)
var<uniform> colors: array<vec4<f32>, 4>;

// Removed globals - not used
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

fn value_to_monochrome(value: f32) -> vec4<f32> {
    // Define a grayscale value by setting all color components to the value
    let grayscale = value; // Value between 0.0 (black) and 1.0 (white)

    // Create a color vector using the grayscale value for all components
    let color = vec4<f32>(grayscale, grayscale, grayscale, 1.0);

    // Return the color with full opacity
    return color;
}

fn value_to_color(value: f32) -> vec4<f32> {
    // Define start, middle, and end colors for the gradient
    let start_color = vec4<f32>(colors[0].x, colors[0].y, colors[0].z, colors[0].w); // Blue
    let middle_color = vec4<f32>(colors[1].x, colors[1].y, colors[1].z, colors[1].w); // Green
    let end_color = vec4<f32>(colors[2].x, colors[2].y, colors[2].z, colors[2].w); // Red

    // Declare a variable for the color
    var color: vec4<f32>;

    // Use an if statement to determine which gradient range to use
    if (value < 0.5) {
        color = mix(start_color, middle_color, value * 2.0);
    } else {
        color = mix(middle_color, end_color, (value - 0.5) * 2.0);
    }

    // Return the color with full opacity
    return color;
}

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> @location(0) vec4<f32> {

    // Calculate the aspect ratio
    let aspect_ratio = viewport_height / viewport_width;

    // Set the center of the circle to be the middle of the UV space
    let center = vec2<f32>(0.5 * aspect_ratio, 0.5);// * aspect_ratio);

    let uv_corrected = vec2<f32>(uv.x * aspect_ratio, uv.y);// * aspect_ratio);

    // Calculate the angle from the current UV coordinate to the center
    let angle_uv = atan2(uv_corrected.y - center.y, uv_corrected.x - center.x);
    var angle_uv_positive = angle_uv;
    if (angle_uv < 0.0) {
        angle_uv_positive += 2.0 * 3.14159;
    }

    // Determine the index based on the angle (64 sections)
    let index = i32(angle_uv_positive / (2.0 * 3.14159) * 64.0);

    // Calculate which component of vec4<f32> to use
    let component_index = index % 4;
    let array_index = index / 4;

    // Extract the correct audio value from the normalized_data array
    let audio_value = normalized_data[array_index][component_index];

    // Define a radius based on the audio_value
    let radius = 0.1 + audio_value * 0.2;

    // Determine if the current UV coordinate is within the defined shape
    let next_index = (index + 1) % 64;
    let next_component_index = next_index % 4;
    let next_array_index = next_index / 4;
    let next_audio_value = normalized_data[next_array_index][next_component_index];
    let next_radius = 0.1 + next_audio_value * 0.2;

    let angle_next = (2.0 * 3.14159 * f32(next_index) / 64.0) + 0.001; // small offset to ensure inclusion of boundary
    let uv_next = center + vec2<f32>(cos(angle_next), sin(angle_next)) * next_radius * aspect_ratio;

    let angle_prev = (2.0 * 3.14159 * f32(index) / 64.0) - 0.001; // small offset to ensure inclusion of boundary
    let uv_prev = center + vec2<f32>(cos(angle_prev), sin(angle_prev)) * radius * aspect_ratio;

    // Calculate barycentric coordinates to check if the point lies within the triangle
    let v0 = uv_next - uv_prev;
    let v1 = center - uv_prev;
    let v2 = uv_corrected - uv_prev;

    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);

    let denom = d00 * d11 - d01 * d01;
    let a = (d11 * d20 - d01 * d21) / denom;
    let b = (d00 * d21 - d01 * d20) / denom;
    let c = 1.0 - a - b;

    // Check if the point is inside the triangle
    if (a >= 0.0 && a <= 1.0 && b >= 0.0 && b <= 1.0 && c >= 0.0 && c <= 1.0) {
        // Get the color based on the audio value
        if (monochrome == 1u){
            return value_to_monochrome(audio_value);
        }else{
            return value_to_color(audio_value * ((-(uv.y * 0.8) + 1.0) + 0.2));
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color
    }
}

