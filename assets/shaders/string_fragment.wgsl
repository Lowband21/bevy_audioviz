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

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let aspect_ratio = viewport_width / viewport_height;

    // Total number of circles
    let num_circles: i32 = 128;
    // Determine the size of each section for a circle
    let section_width = 1.0 / f32(num_circles);
    // Calculate the center of each section
    let section_center_x = (floor(uv.x / section_width) + 0.5) * section_width;

    // Correct the UV coordinates for the aspect ratio
    let uv_corrected = vec2<f32>(uv.x, -uv.y / aspect_ratio);

    // Calculate the index for the audio data based on the x-coordinate
    let index = i32((abs(uv_corrected.x - 0.5) * 2.0) * f32(num_circles / 2)); // *2 because we have half as many data points as circles

    // Determine which quarter of the vec4 to use
    let component_index = index % 4;
    let array_index = index / 4;

    // Retrieve the current audio value
    var audio_value = 0.0;
    // Access the audio data from the appropriate array
    if (uv_corrected.x > 0.5) {
        audio_value = right_data[array_index][component_index];
    } else {
        audio_value = left_data[array_index][component_index];
    }

    // Use the audio value to define the circle's vertical center and diameter
    let scaled_audio_value = -(audio_value / 5.0) + 0.6;
    // Map audio value to circle diameter, then calculate radius
    let max_diameter = section_width; // Maximum diameter is the width of one section
    let diameter = ((audio_value * 0.8) + 0.2) * max_diameter;
    let radius = diameter * 0.5;

    // Calculate distance from the pixel to the section's center
    let dist_to_center = distance(vec2<f32>(uv_corrected.x, -uv_corrected.y), vec2<f32>(section_center_x, scaled_audio_value / aspect_ratio));

    // If the distance is within the circle's radius, color the pixel
    if (dist_to_center < radius) {
        if (monochrome == 1u) {
            return value_to_monochrome(audio_value);
        } else {
            return value_to_color(audio_value);
        }
    } else {
        // If outside the radius, the pixel is not part of the circle
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

