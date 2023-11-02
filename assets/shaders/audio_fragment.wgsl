@group(1) @binding(0)
var<uniform> normalized_data: array<vec4<f32>, 8>;

@group(1) @binding(1)
var<uniform> viewport_width: f32;
@group(1) @binding(2)
var<uniform> viewport_height: f32;

// This function maps audio values to a color gradient between two colors
fn value_to_color(value: f32) -> vec4<f32> {
    // Define start and end colors for the gradient
    let start_color = vec3<f32>(0.0, 0.0, 1.0); // Blue
    let end_color = vec3<f32>(1.0, 0.0, 0.0); // Red

    // Interpolate between start and end colors based on the value
    let color = mix(start_color, end_color, value);

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
    //return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
    // Calculate the index for the audio data based on the uv.x coordinate
    var vec_index = i32(uv.x * 32.0);
    // Ensure vec_index is within bounds
    vec_index = clamp(vec_index, 0, 31);

    // Calculate which component of vec4 to use
    let component_index = vec_index % 4;

    // Initialize audio_value
    var audio_value: f32 = 0.0;

    // Use a switch statement to avoid dynamic indexing
    switch (vec_index / 4) {
        case 0: {audio_value = normalized_data[0][component_index]; break;}
        case 1: {audio_value = normalized_data[1][component_index]; break;}
        case 2: {audio_value = normalized_data[2][component_index]; break;}
        case 3: {audio_value = normalized_data[3][component_index]; break;}
        case 4: {audio_value = normalized_data[4][component_index]; break;}
        case 5: {audio_value = normalized_data[5][component_index]; break;}
        case 6: {audio_value = normalized_data[6][component_index]; break;}
        case 7: {audio_value = normalized_data[7][component_index]; break;}
        default: {break;} // This should never happen as vec_index is clamped to 31
    }

    // Get the color based on the audio value
    let color = value_to_color(audio_value);

    // The height of the bar is represented by the audio value itself
    // Assuming audio_value is normalized between 0.0 and 1.0
    let bar_height = audio_value;

    // Flip the y coordinate system to start from the bottom
    let flipped_y = 1.0 - uv.y; // uv.y is normalized, no need for viewport_height

    // Check if the current fragment's y position is below the bar height
    if (flipped_y <= bar_height) {
        // Draw the bar color
        return color;
    } else {
        // Above the bar height, make it transparent or background color
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}