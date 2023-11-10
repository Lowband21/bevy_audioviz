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

    // Constants for DNA visualization
    let num_strands: i32 = 128;
    let strand_spacing = 1.0 / f32(num_strands);
    let time_scale = globals.time * 5.0;
    let pi = 3.14159265358979323846;
    let two_pi = 2.0 * pi;

    let uv_corrected = vec2<f32>(uv.x, uv.y);

    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Initialize audio_value before using it to calculate twist_offset
    var audio_value = 0.0;
    if (uv_corrected.x > 0.5) {
        let index = i32((abs(uv_corrected.x - 0.5) * 2.0) * f32(num_strands / 2));
        let component_index = index % 4;
        let array_index = index / 4;
        audio_value = right_data[array_index][component_index];
    } else {
        let index = i32((abs(uv_corrected.x - 0.5) * 2.0) * f32(num_strands / 2));
        let component_index = index % 4;
        let array_index = index / 4;
        audio_value = left_data[array_index][component_index];
    }

    // Now calculate the twist_offset using the initialized audio_value
    let twist_intensity = audio_value * 0.5;
    let twist_offset = sin(uv_corrected.y * two_pi * f32(num_strands) + time_scale) * twist_intensity;

    // Use the updated audio_value to perform further calculations
    let strand_x_pos = f32(i32(uv_corrected.x * f32(num_strands))) * strand_spacing;

    let top_strand = 0.5 + audio_value * 0.1;
    let bottom_strand = 0.5 - audio_value * 0.1;

    let alpha = mix(0.1, 0.8, audio_value);

    // Improved gradient calculation
    let gradient = uv_corrected.y * 0.5 + 0.5;
    
    // Glow effect
    let glow = exp(-10.0 * abs(uv_corrected.y - (top_strand + bottom_strand) * 0.5));

    if (abs(uv_corrected.y - top_strand) < 0.001 || abs(uv_corrected.y - bottom_strand) < 0.001) {
        if (monochrome == 1u) {
            color = value_to_monochrome(audio_value);
        } else {
            color = value_to_color(audio_value);
        }
        color.a = alpha;
    } else if (uv_corrected.y < top_strand - 0.001 && uv_corrected.y > bottom_strand + 0.001) {
        if (monochrome == 1u) {
            color = value_to_monochrome(audio_value);
        } else {
            color = value_to_color(audio_value);
        }
        color.a = alpha * glow;
    }

    // Mix the base color with the gradient for a smooth transition
    color = vec4<f32>(mix(color.rgb, color.rgb * gradient, 0.3), color.a);

    return color;
}
