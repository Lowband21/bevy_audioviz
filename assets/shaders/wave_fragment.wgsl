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
const GRID_CELL_COUNT: f32 = 16.0;      // Number of grid cells
const GRID_LINE_SOFTNESS: f32 = 0.005;  // Softness of grid lines - increased from 0.002
const GRID_OPACITY: f32 = 0.18;         // Base opacity of the grid lines - slightly increased
const MINOR_GRID_INTENSITY: f32 = 0.0;  // Intensity of minor grid lines - set to 0 to remove them

// Dot grid parameters
const DOT_SIZE: f32 = 0.003;            // Size of dots (smaller value = smaller dots)
const DOT_FALLOFF: f32 = 0.5;          // Sharpness of dot edges (0.0-1.0, lower = smoother)
const VERTICAL_GRID_COUNT: f32 = 32.0;  // Number of vertical grid divisions
const HORIZONTAL_GRID_COUNT: f32 = 16.0; // Number of horizontal grid divisions
const AUDIO_BAND_SPACING: f32 = 2.0;  // Sample every Nth audio band
const DOT_BRIGHTNESS: f32 = 1.0;       // Brightness multiplier for dots

// Deformation parameters
const DEFORM_STRENGTH: f32 = 0.8;       // Overall deformation strength (reduced for stability)
const INFLUENCE_RADIUS: f32 = 0.25;     // Radius of influence for wave deformation

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

// Function for smooth falloff
fn smooth_falloff(distance: f32, radius: f32) -> f32 {
    // Ensure smooth falloff that reaches exactly 0 at the radius boundary
    return smoothstep(radius, 0.0, distance);
}

// Improved coordinate space transformation for continuous deformation
fn transform_coordinate_space(uv: vec2<f32>, top_strand: f32, bottom_strand: f32, audio_value: f32) -> vec2<f32> {
    // Create a copy of the original coordinates
    var transformed_uv = uv;
    
    // Center between strands
    let center_y = (top_strand + bottom_strand) * 0.5;
    
    // Half distance between strands
    let strand_half_width = (top_strand - bottom_strand) * 0.5;
    
    // Calculate distance to center line between strands
    let dist_to_center = abs(uv.y - center_y);
    
    // Define hard barriers (screen boundaries with small margin)
    let top_barrier = 1.00;
    let bottom_barrier = 0.00;
    
    // Calculate a gentler repulsion factor - reduced overall strength
    let repulsion_strength = audio_value * DEFORM_STRENGTH * 0.8; // Reduced from 1.8
    
    // Create a smaller void zone around the waveform
    let void_size = max(strand_half_width * 1.1, 0.03 + audio_value * 0.05);
    
    // Apply the transformation with more controlled behavior
    var new_y = uv.y; // Start with original y position
    
    // Inside the void zone - simple vertical push away
    if (dist_to_center < void_size) {
        // Gentler push factor
        let push_factor = (1.0 - dist_to_center / void_size) * repulsion_strength * 0.4;
        
        // Direction vector pointing away from center_y
        let direction = sign(uv.y - center_y);
        
        // Apply a simpler, more controlled vertical push
        new_y = uv.y + direction * push_factor * 0.08;
    } 
    // Outside the void zone - consistent vertical scaling toward barriers
    else {
        // Distance from void zone edge (normalized 0-1)
        let outside_distance = (dist_to_center - void_size) / max(0.5 - void_size, 0.001);
        let outside_factor = smoothstep(0.0, 1.0, outside_distance);
        
        // Reduced compression factor
        let vertical_compression = outside_factor * repulsion_strength * 0.25;
        
        // Apply scaling based on whether point is above or below center
        if (uv.y > center_y) {
            // Calculate compression toward top barrier
            // How close we are to the barrier (0 = at center, 1 = at barrier)
            let barrier_factor = (uv.y - center_y) / max(top_barrier - center_y, 0.001);
            
            // Create a smooth compression effect that increases toward the barrier
            // Points closer to the barrier compress more dramatically
            let compression_curve = smoothstep(0.0, 1.0, barrier_factor);
            
            // Calculate new position with compression toward barrier
            new_y = center_y + (uv.y - center_y) * (1.0 + vertical_compression * compression_curve);
            
            // Ensure we don't exceed the top barrier
            if (new_y > top_barrier) {
                // Compress coordinates to stay within barrier
                let overshoot = (new_y - top_barrier) / max(new_y - center_y, 0.001);
                new_y = top_barrier - overshoot * 0.01;
            }
        } else {
            // Calculate compression toward bottom barrier
            // How close we are to the barrier (0 = at center, 1 = at barrier)
            let barrier_factor = (center_y - uv.y) / max(center_y - bottom_barrier, 0.001);
            
            // Create a smooth compression effect that increases toward the barrier
            // Points closer to the barrier compress more dramatically
            let compression_curve = smoothstep(0.0, 1.0, barrier_factor);
            
            // Calculate new position with compression toward barrier
            // Note: we're being careful with the signs here to ensure symmetry
            new_y = center_y - (center_y - uv.y) * (1.0 + vertical_compression * compression_curve);
            
            // Ensure we don't exceed the bottom barrier
            if (new_y < bottom_barrier) {
                // Compress coordinates to stay within barrier
                let overshoot = (bottom_barrier - new_y) / max(center_y - new_y, 0.001);
                new_y = bottom_barrier + overshoot * 0.01;
            }
        }
    }
    
    // Apply the transformed y coordinate
    transformed_uv.y = new_y;
    
    // Apply minimal horizontal adjustment for visual consistency
    // Reduced horizontal effect
    transformed_uv.x = 0.5 + (uv.x - 0.5) * (1.0 - abs(uv.y - center_y) * audio_value * 0.01);
    
    return transformed_uv;
}

// Create a dot-based grid with the transformed coordinates
fn create_deformed_grid(uv: vec2<f32>, top_strand: f32, bottom_strand: f32, audio_value: f32) -> vec4<f32> {
    // Background color
    let background_color = vec3<f32>(0.00, 0.00, 0.00);
    var final_color = background_color;
    
    // Apply deformation to coordinates
    let transformed_uv = transform_coordinate_space(uv, top_strand, bottom_strand, audio_value);
    
    // Audio band setup - to ensure dots align with audio bands
    let num_audio_bands = 128.0;
    let audio_band_width = 1.0 / num_audio_bands;
    
    // Get the center of the waveform
    let waveform_center = (top_strand + bottom_strand) * 0.5;
    
    // Vertical grid - now centered around the waveform
    let cell_height = 1.0 / VERTICAL_GRID_COUNT;
    
    // Calculate which audio band this pixel belongs to
    let band_index = floor(transformed_uv.x * num_audio_bands);
    
    // Only draw dots for certain audio bands (for sparser grid)
    // WGSL requires integer modulo, so we convert to int first
    let band_index_i = i32(band_index);
    let spacing_i = i32(AUDIO_BAND_SPACING);
    
    // Number of horizontal lines to draw (must be odd to have one at center)
    let num_lines = 7; // Total number of horizontal lines
    let half_lines = num_lines / 2; // Lines on each side of center
    
    // Store grid point positions to reuse for vertical and horizontal lines
    var grid_points_x: array<f32, 64>; // Max 64 x positions
    var grid_points_y: array<f32, 64>;  // Max 7 y positions
    var num_x_points = 0;
    
    // Pre-calculate all y positions
    for (var i = -half_lines; i <= half_lines; i++) {
        // Scale determines how far apart the lines are
        let scale = 0.12; // Adjust this to control vertical spread
        let grid_center_y = waveform_center + f32(i) * scale;
        
        // Skip lines that would be off-screen
        if (grid_center_y < 0.0 || grid_center_y > 1.0) {
            continue;
        }
        
        // Store y position
        grid_points_y[i + half_lines] = grid_center_y;
    }
    
    // Store x positions where we'll draw dots (audio bands)
    for (var b = 0; b < i32(num_audio_bands); b++) {
        if (b % spacing_i == 0 && num_x_points < 64) {
            let band_center_x = (f32(b) + 0.5) * audio_band_width;
            grid_points_x[num_x_points] = band_center_x;
            num_x_points = num_x_points + 1;
        }
    }
    
    // Draw dots at grid intersections
    for (var x = 0; x < num_x_points; x++) {
        let grid_x = grid_points_x[x];
        
        for (var i = -half_lines; i <= half_lines; i++) {
            // Scale determines how far apart the lines are
            let scale = 0.12; // Adjust this to control vertical spread
            let grid_y = waveform_center + f32(i) * scale;
            
            // Skip lines that would be off-screen
            if (grid_y < 0.0 || grid_y > 1.0) {
                continue;
            }
            
            // Calculate distance to this grid point
            let dist_x = transformed_uv.x - grid_x;
            let dist_y = transformed_uv.y - grid_y;
            let dist = sqrt(dist_x * dist_x + dist_y * dist_y);
            
            // Draw dot if we're close enough
            let dot_intensity = smoothstep(DOT_SIZE, DOT_SIZE * (1.0 - DOT_FALLOFF), dist);
            
            // Get the audio value for this x position (band)
            let band_index_at_x = floor(grid_x * num_audio_bands);
            let band_index_i_at_x = i32(band_index_at_x);
            var band_audio_value = audio_value;
            
            if (band_index_at_x < 64.0) {
                // Left channel
                let comp_index = band_index_i_at_x % 4;
                let array_index = band_index_i_at_x / 4;
                if (array_index < 16 && comp_index < 4) {
                    band_audio_value = left_data[array_index][comp_index];
                }
            } else {
                // Right channel
                let right_index = band_index_at_x - 64.0;
                let comp_index = i32(right_index) % 4;
                let array_index = i32(right_index) / 4;
                if (array_index < 16 && comp_index < 4) {
                    band_audio_value = right_data[array_index][comp_index];
                }
            }
            
            // Basic dot color - cyan with audio intensity
            let dot_color = vec3<f32>(0.2, 0.8, 1.0) * (1.0 + band_audio_value * DOT_BRIGHTNESS);
            
            // Apply dot to background
            final_color = mix(final_color, dot_color, dot_intensity);
        }
    }
    
    return vec4<f32>(final_color, 1.0);
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

    // Constants for wave visualization
    let num_strands: i32 = 128;
    let strand_spacing = 1.0 / f32(num_strands);
    let time_scale = FIXED_TIME_SCALE;
    let pi = 3.14159265358979323846;
    let two_pi = 2.0 * pi;

    // Correct UV for aspect ratio
    let uv_corrected = vec2<f32>(uv.x, uv.y);

    // Initialize audio_value before using it to calculate strand positions
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

    // Calculate wave strand positions
    let top_strand = 0.5 + audio_value * 0.1;
    let bottom_strand = 0.5 - audio_value * 0.1;

    // Create the deformed grid based on strand positions
    var color: vec4<f32> = create_deformed_grid(uv_corrected, top_strand, bottom_strand, audio_value);

    let twist_intensity = audio_value * 0.5;
    let twist_offset = sin(uv_corrected.y * two_pi * f32(num_strands) + time_scale) * twist_intensity;

    // Use the updated audio_value to perform further calculations
    let strand_x_pos = f32(i32(uv_corrected.x * f32(num_strands))) * strand_spacing;

    let alpha = mix(0.1, 0.8, audio_value);
    
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

    return color;
}
