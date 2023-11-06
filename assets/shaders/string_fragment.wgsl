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

@group(1) @binding(5)
var<uniform> colors: array<vec4<f32>, 4>;

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
    let start_color = vec3<f32>(colors[0].x, colors[0].y, colors[0].z); // Blue
    let middle_color = vec3<f32>(colors[1].x, colors[1].y, colors[1].z); // Green
    let end_color = vec3<f32>(colors[2].x, colors[2].y, colors[2].z); // Red

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


//@fragment
//fn fragment(
//    @builtin(position) coord: vec4<f32>,
//    @location(0) world_position: vec4<f32>,
//    @location(1) normals: vec3<f32>,
//    @location(2) uv: vec2<f32>,
//) -> @location(0) vec4<f32> {
//    let aspect_ratio = viewport_width / viewport_height;
//
//    // Correct the UV coordinates by scaling the y-coordinate by the aspect ratio
//    let uv_corrected = vec2<f32>(uv.x, uv.y / aspect_ratio);
//
//    // Calculate the index for accessing audio data
//    let index = i32(abs((uv_corrected.x - 0.5) * 2.0) * 64.0);
//
//    // Calculate which component of vec4<f32> to use
//    let component_index = index % 4;
//    let array_index = index / 4;
//
//    // Extract the correct audio value from the normalized_data array
//    var audio_value = 0.0;
//    if (uv_corrected.x > 0.5) {
//        audio_value = right_data[array_index][component_index];
//    } else {
//        audio_value = left_data[array_index][component_index];
//    }
//
//    // Define the line's thickness
//    let line_thickness = 0.001; // This can be adjusted as needed
//
//    // Calculate the distance of the current pixel from the line position
//    let line_position = audio_value / 5.0 + 0.4;
//    let distance_to_line = abs(uv_corrected.y - line_position);
//
//    // Determine if the current UV coordinate is within the line's thickness
//    if (distance_to_line < line_thickness) {
//        // Render the pixel if it falls within the line's thickness
//        if (monochrome == 1u){
//            return value_to_monochrome(audio_value);
//        } else {
//            return value_to_color(audio_value);
//        }
//    } else {
//        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color
//    }
//}



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

    // Calculate the indices for the audio data
    let index = i32(abs((uv_corrected.x - 0.5) * 2.0) * 64.0);
    let prev_index = max(index - 1, 0);
    let next_index = min(index + 1, 63 * 4); // 64 sections in half-circle, each with 4 values

    // Calculate which component of vec4<f32> to use for the current, previous, and next indices
    let component_index = index % 4;
    let prev_component_index = prev_index % 4;
    let next_component_index = next_index % 4;
    let array_index = index / 4;
    let prev_array_index = prev_index / 4;
    let next_array_index = next_index / 4;

    // Retrieve the previous, current, and next audio values for interpolation
    var prev_audio_value = 0.0;
    var current_audio_value = 0.0;
    var next_audio_value = 0.0;
    if (uv_corrected.x > 0.5) {
        prev_audio_value = right_data[prev_array_index][prev_component_index];
        current_audio_value = right_data[array_index][component_index];
        next_audio_value = right_data[next_array_index][next_component_index];
    } else {
        prev_audio_value = left_data[prev_array_index][prev_component_index];
        current_audio_value = left_data[array_index][component_index];
        next_audio_value = left_data[next_array_index][next_component_index];
    }

    // Calculate the mix ratio for interpolation
    let mix_ratio = fract(abs((uv_corrected.x - 0.5) * 2.0) * 64.0);

    // Interpolate between the previous and next audio values to get the interpolated value
    let audio_value = mix(prev_audio_value, next_audio_value, mix_ratio);

    // Scale and position the audio value for rendering
    let audio_value_scaled = (audio_value / 5.0) + 0.4;

    // Define the line's thickness
    let line_thickness = 0.001; // Adjust this value for a thicker or thinner line

    // Determine if the current UV coordinate is within the thickness of the line
    if (abs(uv_corrected.y - audio_value_scaled) < line_thickness) {
        // Return the color if the pixel falls within the line
        if (monochrome == 1u) {
            return value_to_monochrome(audio_value);
        } else {
            return value_to_color(audio_value);
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color for pixels outside the line
    }
}

