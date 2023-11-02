@group(1) @binding(0)
var<uniform> normalized_data: array<vec4<f32>, 16>;

@group(1) @binding(1)
var<uniform> viewport_width: f32;
@group(1) @binding(2)
var<uniform> viewport_height: f32;

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
    //return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
    // Calculate the index for the audio data based on the uv.x coordinate
    var vec_index = i32(uv.x * 64.0);
    // Ensure vec_index is within bounds
    vec_index = clamp(vec_index, 0, 63);

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
        case 8: {audio_value = normalized_data[8][component_index]; break;}
        case 9: {audio_value = normalized_data[9][component_index]; break;}
        case 10: {audio_value = normalized_data[10][component_index]; break;}
        case 11: {audio_value = normalized_data[11][component_index]; break;}
        case 12: {audio_value = normalized_data[12][component_index]; break;}
        case 13: {audio_value = normalized_data[13][component_index]; break;}
        case 14: {audio_value = normalized_data[14][component_index]; break;}
        case 15: {audio_value = normalized_data[15][component_index]; break;}
        default: {break;} // This should never happen as vec_index is clamped to 31
    }

    // Get the color based on the audio value
    let color = value_to_color(audio_value);

    // Calculate the dynamic bar width based on the audio value
    let bar_width = mix(0.02, 0.1, audio_value); // Linearly interpolate between min and max widths
    let half_bar_width = bar_width * 0.5;

    // Calculate the x position relative to the center of the screen
    let centered_x = uv.x - 0.5;

    // Calculate bar height and flip y coordinate system
    let bar_height = audio_value;
    let flipped_y = 1.0 - uv.y;

    // Soft edges using smoothstep function
    let edge_softness = 0.01; // Edge softness value
    let alpha = smoothstep(0.0, edge_softness, bar_height - flipped_y);

    // Draw the bar with soft edges
    if (flipped_y <= bar_height) {
        return vec4<f32>(color.rgb, alpha);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Draw black if above the bar height
    }
}
