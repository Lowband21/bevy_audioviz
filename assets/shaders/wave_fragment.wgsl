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

// Constants to replace global time
const FIXED_TIME_SCALE: f32 = 0.5;

// Grid parameters
const GRID_CELL_COUNT: f32 = 10.0;  // Number of major grid cells
const GRID_LINE_SOFTNESS: f32 = 0.002;  // Softness of grid lines
const MINOR_GRID_INTENSITY: f32 = 0.3;  // Intensity of minor grid lines
const GRID_WARP_INTENSITY: f32 = 0.15;  // How much the grid warps with audio
const GRID_PERSPECTIVE: f32 = 2.0;  // Perspective strength (higher = more dramatic)

// Removed global struct and binding
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

// Function to apply spacetime warping based on audio data
fn warp_uv(uv: vec2<f32>, audio_data: array<vec4<f32>, 16>) -> vec2<f32> {
    var warped_uv = uv;
    
    // Create warp field based on audio data from multiple frequency bands
    // This creates a complex distortion field where different areas of the grid 
    // respond to different frequency bands
    
    // Low frequencies (bass) create larger waves
    let low_freq = (audio_data[0][0] + audio_data[0][1]) / 2.0;
    let mid_freq = (audio_data[1][0] + audio_data[1][1]) / 2.0;
    let high_freq = (audio_data[2][0] + audio_data[2][1]) / 2.0;
    
    // Apply different distortion patterns for each frequency range
    // Bass causes large, slow waves
    warped_uv.x += sin(uv.y * 4.0) * sin(uv.x * 3.0) * low_freq * GRID_WARP_INTENSITY;
    warped_uv.y += sin(uv.x * 4.0) * sin(uv.y * 3.0) * low_freq * GRID_WARP_INTENSITY;
    
    // Mid frequencies cause medium distortions
    warped_uv.x += sin(uv.y * 8.0) * mid_freq * GRID_WARP_INTENSITY * 0.7;
    warped_uv.y += sin(uv.x * 8.0) * mid_freq * GRID_WARP_INTENSITY * 0.7;
    
    // High frequencies cause small ripples
    warped_uv.x += sin(uv.y * 16.0) * high_freq * GRID_WARP_INTENSITY * 0.3;
    warped_uv.y += sin(uv.x * 16.0) * high_freq * GRID_WARP_INTENSITY * 0.3;
    
    return warped_uv;
}

// Create a grid with perspective and audio reactivity
fn create_grid(uv: vec2<f32>, audio_left: array<vec4<f32>, 16>, audio_right: array<vec4<f32>, 16>) -> vec4<f32> {
    // Calculate average audio amplitude
    let avg_low = (audio_left[0][0] + audio_right[0][0]) / 2.0;
    let avg_mid = (audio_left[1][1] + audio_right[1][1]) / 2.0;
    let avg_high = (audio_left[2][2] + audio_right[2][2]) / 2.0;
    let avg_audio = (avg_low + avg_mid + avg_high) / 3.0;
    
    // Center UV and apply perspective transformation
    var centered_uv = uv * 2.0 - 1.0;
    
    // Warp the UV coordinates based on audio
    centered_uv = warp_uv(centered_uv, audio_left);
    
    // Apply perspective transformation (further = more compressed)
    let perspective = vec2<f32>(
        centered_uv.x / (1.0 + abs(centered_uv.y) * GRID_PERSPECTIVE),
        centered_uv.y / (1.0 + abs(centered_uv.x) * GRID_PERSPECTIVE * 0.5)
    );
    
    // Scale back to 0-1 range
    var grid_uv = perspective * 0.5 + 0.5;
    
    // Create major grid lines with adjustable cell count
    // Multiply by larger number for more grid cells
    let major_grid_size = GRID_CELL_COUNT;
    let minor_grid_size = GRID_CELL_COUNT * 4.0; // 4x resolution for minor grid
    
    // Create major and minor grid patterns
    let major_grid_x = abs(fract(grid_uv.x * major_grid_size) - 0.5) * 2.0;
    let major_grid_y = abs(fract(grid_uv.y * major_grid_size) - 0.5) * 2.0;
    let minor_grid_x = abs(fract(grid_uv.x * minor_grid_size) - 0.5) * 2.0;
    let minor_grid_y = abs(fract(grid_uv.y * minor_grid_size) - 0.5) * 2.0;
    
    // Use smoothstep for softer grid lines
    let line_width_major = mix(0.01, 0.03, avg_audio); // Line width increases with audio
    let line_width_minor = line_width_major * 0.5;
    
    // Calculate grid values with smoothstep for soft edges
    let major_x_line = 1.0 - smoothstep(line_width_major - GRID_LINE_SOFTNESS, line_width_major + GRID_LINE_SOFTNESS, major_grid_x);
    let major_y_line = 1.0 - smoothstep(line_width_major - GRID_LINE_SOFTNESS, line_width_major + GRID_LINE_SOFTNESS, major_grid_y);
    let minor_x_line = 1.0 - smoothstep(line_width_minor - GRID_LINE_SOFTNESS, line_width_minor + GRID_LINE_SOFTNESS, minor_grid_x);
    let minor_y_line = 1.0 - smoothstep(line_width_minor - GRID_LINE_SOFTNESS, line_width_minor + GRID_LINE_SOFTNESS, minor_grid_y);
    
    // Combine major and minor grid lines
    let major_grid = max(major_x_line, major_y_line);
    let minor_grid = max(minor_x_line, minor_y_line) * MINOR_GRID_INTENSITY;
    
    // Final grid intensity (use max to avoid double-brightening at intersections)
    let grid_intensity = max(major_grid, minor_grid);
    
    // Calculate depth-based fading (further = dimmer)
    let depth_fade = 1.0 - pow(abs(centered_uv.y), 2.0) * 0.5;
    
    // Calculate grid color based on audio spectrum
    // Use separate frequency bands to influence different color channels
    let grid_base_color = value_to_color(avg_audio * 0.5 + 0.2);
    
    // Apply audio reactivity to grid brightness
    let base_brightness = 0.02; // Minimum brightness
    let reactive_brightness = base_brightness + avg_audio * 0.04;
    
    // Apply edge glow effect based on uv coordinates
    let edge_glow = 1.0 - pow(abs(centered_uv.x), 3.0) - pow(abs(centered_uv.y), 3.0);
    let edge_glow_factor = max(0.0, edge_glow) * avg_low * 0.05;
    
    // Combine everything for the final grid color
    let grid_color = grid_base_color.rgb * reactive_brightness * depth_fade * (grid_intensity + edge_glow_factor);
    
    return vec4<f32>(grid_color, 1.0);
}

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
    let start_color = vec4<f32>(colors[0].x, colors[0].y, colors[0].z, colors[0].w); 
    let middle_color = vec4<f32>(colors[1].x, colors[1].y, colors[1].z, colors[1].w); 
    let end_color = vec4<f32>(colors[2].x, colors[2].y, colors[2].z, colors[2].w); 

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
    // Using fixed time value instead of globals.time
    let time_scale = FIXED_TIME_SCALE;
    let pi = 3.14159265358979323846;
    let two_pi = 2.0 * pi;

    // Correct UV for aspect ratio
    let uv_corrected = vec2<f32>(uv.x, uv.y);
    
    // Start with the enhanced spacetime grid as our background
    var color: vec4<f32> = create_grid(uv_corrected, left_data, right_data);

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

    // Declare a variable for the wave color
    var wave_color: vec4<f32>;

    if (abs(uv_corrected.y - top_strand) < 0.001 || abs(uv_corrected.y - bottom_strand) < 0.001) {
        if (monochrome == 1u) {
            wave_color = value_to_monochrome(audio_value);
        } else {
            wave_color = value_to_color(audio_value);
        }
        wave_color.a = alpha;
        
        // Blend the wave on top of our background
        color = mix(color, wave_color, wave_color.a);
    } else if (uv_corrected.y < top_strand - 0.001 && uv_corrected.y > bottom_strand + 0.001) {
        if (monochrome == 1u) {
            wave_color = value_to_monochrome(audio_value);
        } else {
            wave_color = value_to_color(audio_value);
        }
        wave_color.a = alpha * glow;
        
        // Blend the wave on top of our background
        color = mix(color, wave_color, wave_color.a);
    }

    // Mix the wave color with the gradient for a smooth transition
    return color;
}
