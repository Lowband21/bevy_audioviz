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

//@group(2) @binding(5)
//var<uniform> color_start: vec4<f32>;
//@group(2) @binding(6)
//var<uniform> color_middle: array<f32, 3>;
//@group(2) @binding(7)
//var<uniform> color_end: array<f32, 3>;

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
    let aspect_ratio = viewport_width / viewport_height;
    
    // Correct the UV coordinates by scaling the y-coordinate by the aspect ratio
    let uv_corrected = vec2<f32>(uv.x, uv.y / aspect_ratio);

    // Adjust the center of the circle to be the middle of the UV space after correction
    let center = vec2<f32>(0.5, 0.5 / aspect_ratio);

    // Calculate the angle from the current UV coordinate to the center
    var angle_uv = atan2(uv_corrected.y - center.y, uv_corrected.x - center.x);

    // Rotate the angle by 90 degrees (π/2 radians)
    angle_uv += 3.14159 * 0.5;

    // Normalize the angle between 0 and 2π
    if (angle_uv > 2.0 * 3.14159) {
        angle_uv -= 2.0 * 3.14159;
    } else if (angle_uv < 0.0) {
        angle_uv += 2.0 * 3.14159;
    }

    // Reflect the angle to the right half-circle if it's in the left half
    if (angle_uv > 3.14159) {
        angle_uv = 2.0 * 3.14159 - angle_uv;
    }

    // Since we're now only dealing with half the circle for the data,
    // we need to double the index range to cover the full range of audio data.
    let index = i32(angle_uv / (3.14159) * 64.0); // 64 sections in half-circle

    // Calculate which component of vec4<f32> to use
    let component_index = index % 4;
    let array_index = index / 4;

    // Extract the correct audio value from the normalized_data array
    var audio_value = 0.0;
    if (uv.x > 0.5) {
        audio_value = right_data[array_index][component_index];
    } else {
        audio_value = left_data[array_index][component_index];
    }

    // Define a radius based on the audio_value
    let radius = 0.1 + audio_value * 0.15;

    // Calculate distance from the corrected UV coordinate to the center
    let distance_to_center = distance(center, uv_corrected);

    // Determine if the current UV coordinate is within the circle's radius
    if (distance_to_center < radius) {
        if (monochrome == 1u){
            return value_to_monochrome(audio_value);
        }else {
            return value_to_color(audio_value);
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color
    }
}

