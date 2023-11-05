@group(1) @binding(0)
var<uniform> left_data: array<vec4<f32>, 16>;
@group(1) @binding(1)
var<uniform> right_data: array<vec4<f32>, 16>;

@group(1) @binding(2)
var<uniform> viewport_width: f32;
@group(1) @binding(3)
var<uniform> viewport_height: f32;
@group(1) @binding(4)
var<uniform> monochrome: u32;

//@group(1) @binding(5)
//var<uniform> color_start: vec4<f32>;
//@group(1) @binding(6)
//var<uniform> color_middle: array<f32, 3>;
//@group(1) @binding(7)
//var<uniform> color_end: array<f32, 3>;

struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    time: f32,
    // The delta time since the previous frame in seconds
    delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: f32
#endif
}

@group(0) @binding(1)
var<uniform> globals: Globals;



fn value_to_monochrome(value: f32) -> vec4<f32> {
    // Define a grayscale value by setting all color components to the value
    let grayscale = value; // Value between 0.0 (black) and 1.0 (white)

    // Create a color vector using the grayscale value for all components
    let color = vec3<f32>(grayscale, grayscale, grayscale);

    // Return the color with full opacity
    return vec4<f32>(color, 1.0);
}
fn value_to_color(value: f32) -> vec4<f32> {
    // Define start, middle, and end colors for the gradient
    let start_color = vec3<f32>(0.0, 0.0, 1.0); // Blue
    let middle_color = vec3<f32>(0.0, 1.0, 0.0); // Green
    let end_color = vec3<f32>(1.0, 0.0, 0.0); // Red

    // Declare a variable for the color
    var color: vec3<f32>;

    // Use an if statement to determine which gradient range to use
    if (value < 0.5) {
        color = mix(start_color, middle_color, value * 2.0);
    } else {
        color = mix(middle_color, end_color, (value - 0.5) * 2.0);
    }

    // Return the color with full opacity
    return vec4<f32>(color, 1.0);
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

