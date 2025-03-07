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
const GRID_LINE_SOFTNESS: f32 = 0.002;  // Softness of grid lines
const GRID_OPACITY: f32 = 0.15;         // Base opacity of the grid lines
const MINOR_GRID_INTENSITY: f32 = 0.3;  // Intensity of minor grid lines

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
    let top_barrier = 0.95;
    let bottom_barrier = 0.05;
    
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

// Create a clean grid with the transformed coordinates
fn create_deformed_grid(uv: vec2<f32>, top_strand: f32, bottom_strand: f32, audio_value: f32) -> vec4<f32> {
    // Transform the entire coordinate space using our improved function
    let transformed_uv = transform_coordinate_space(uv, top_strand, bottom_strand, audio_value);
    
    // Center between strands for void calculation
    let center_y = (top_strand + bottom_strand) * 0.5;
    
    // Calculate distance to center line between strands
    let dist_to_center = abs(uv.y - center_y);
    
    // Create a void zone with reduced size
    let void_size = max((top_strand - bottom_strand) * 0.7, 0.04 + audio_value * 0.06);
    
    // Generate grid using the transformed coordinates
    let major_grid_size = GRID_CELL_COUNT;
    let minor_grid_size = GRID_CELL_COUNT * 4.0;  // 4x density for minor grid
    
    // Calculate grid line positions
    let major_grid_x_pos = transformed_uv.x * major_grid_size;
    let major_grid_y_pos = transformed_uv.y * major_grid_size;
    let minor_grid_x_pos = transformed_uv.x * minor_grid_size;
    let minor_grid_y_pos = transformed_uv.y * minor_grid_size;
    
    // Calculate minimum distance to nearest grid line
    let major_grid_x_dist = min(fract(major_grid_x_pos), 1.0 - fract(major_grid_x_pos));
    let major_grid_y_dist = min(fract(major_grid_y_pos), 1.0 - fract(major_grid_y_pos));
    let minor_grid_x_dist = min(fract(minor_grid_x_pos), 1.0 - fract(minor_grid_x_pos));
    let minor_grid_y_dist = min(fract(minor_grid_y_pos), 1.0 - fract(minor_grid_y_pos));
    
    // Line width parameters - adjusted for cleaner look
    let base_width_major = 0.015;
    let base_width_minor = 0.008;
    
    // Calculate line width scaling based on distance from center
    let center_distance_scale = smoothstep(0.0, 0.5, distance(uv, vec2<f32>(0.5, 0.5)));
    let line_width_major = base_width_major * (1.0 - center_distance_scale * 0.2);
    let line_width_minor = base_width_minor * (1.0 - center_distance_scale * 0.2);
    
    // Calculate AA width
    let aa_width_major = max(0.001, GRID_LINE_SOFTNESS);
    let aa_width_minor = max(0.0005, GRID_LINE_SOFTNESS * 0.5);
    
    // Calculate grid lines with improved antialiasing
    let major_x_line = smoothstep(line_width_major, line_width_major - aa_width_major, major_grid_x_dist);
    let major_y_line = smoothstep(line_width_major, line_width_major - aa_width_major, major_grid_y_dist);
    let minor_x_line = smoothstep(line_width_minor, line_width_minor - aa_width_minor, minor_grid_x_dist);
    let minor_y_line = smoothstep(line_width_minor, line_width_minor - aa_width_minor, minor_grid_y_dist);
    
    // Combine grid lines
    let major_grid = max(major_x_line, major_y_line);
    let minor_grid = max(minor_x_line, minor_y_line) * MINOR_GRID_INTENSITY;
    
    // Create an even softer void factor
    let void_factor = smoothstep(void_size * 0.5, void_size * 1.8, dist_to_center);
    // Higher minimum visibility (50%)
    let soft_void_factor = mix(0.5, 1.0, void_factor);
    
    // Apply gentler audio-reactive grid intensity
    let grid_intensity = (1.0 + audio_value * 0.1) * soft_void_factor;
    
    // Use smoother blending for combined grid
    let grid = smoothstep(0.0, 1.0, max(major_grid, minor_grid)) * grid_intensity;
    
    // Enhanced distance-based effects
    let center_dist = distance(uv, vec2<f32>(0.5, 0.5));
    
    // Create a more pronounced radial fade that enhances the void effect
    let radial_fade = 1.0 - smoothstep(0.35, 0.85, center_dist);
    
    // Improved depth gradient with audio reactivity
    let depth_gradient = 1.0 - pow(abs(uv.y - 0.5) * 2.0, 2.0) * 0.3;
    
    // Audio-reactive color shift with enhanced edge contrast
    let base_grid_color = vec3<f32>(0.1, 0.2, 0.5);
    let highlight_color = vec3<f32>(0.2, 0.4, 0.8);
    
    // Gentler color mix
    let color_mix_factor = audio_value * 0.2 * (1.0 - center_distance_scale * 0.5);
    let grid_color_mix = mix(base_grid_color, highlight_color, color_mix_factor);
    
    // Combine gradients with gentler audio reactivity
    let edge_emphasis = 1.0 + (1.0 - soft_void_factor) * audio_value * 0.15;
    let combined_gradient = mix(depth_gradient, radial_fade, 0.3) * edge_emphasis;
    
    // Calculate final grid color with adjusted opacity
    let grid_opacity = GRID_OPACITY * 1.3 * soft_void_factor; 
    let grid_color = grid_color_mix * grid_opacity * combined_gradient * grid;
    
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
