#import bevy_pbr::forward_io::VertexOutput

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

fn value_to_monochrome(value: f32) -> vec4<f32> {
    // Define a grayscale value by setting all color components to the value
    let grayscale = value; // Value between 0.0 (black) and 1.0 (white)

    // Create a color vector using the grayscale value for all components
    let color = vec4<f32>(grayscale, grayscale, grayscale, 1.0);

    // Return the color with full opacity
    return color;
}

fn value_to_color(value: f32, current_colors: array<vec4<f32>, 4>) -> vec4<f32> {
    // Define start, middle, and end colors for the gradient
    let start_color = vec4<f32>(current_colors[0].x, current_colors[0].y, current_colors[0].z, current_colors[0].w); // Blue
    let middle_color = vec4<f32>(current_colors[1].x, current_colors[1].y, current_colors[1].z, current_colors[1].w); // Green
    let end_color = vec4<f32>(current_colors[2].x, current_colors[2].y, current_colors[2].z, current_colors[2].w); // Red

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
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    
    var vec_index = i32(uv.x * 64.0);
    vec_index = clamp(vec_index, 0, 63);

    // Calculate which component of vec4 and which array to use
    let array_index = vec_index / 4;
    let component_index = vec_index % 4;

    // Assuming normalized_data is a two-dimensional array declared as:
    var audio_value: f32 = normalized_data[array_index][component_index];

    // Calculate bar height and flip y coordinate system
    let bar_height = audio_value * 0.8; // Scale the bar height to 80%
    let flipped_y = 1.0 - uv.y;

    // Get the color based on the audio value
    var color: vec4<f32>;
    if (monochrome == 1u){
        color = value_to_monochrome(audio_value);
    }else{
        color = value_to_color(audio_value * ((-(uv.y * 0.8) + 1.0) + 0.2), colors);
    }
     
    if (flipped_y <= bar_height) {
        // Inside the bar - show color
        return color;
    } else {
        // Above the bar - black background
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
