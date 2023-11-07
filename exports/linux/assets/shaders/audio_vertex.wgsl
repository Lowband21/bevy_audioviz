struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> transform: mat4x4<f32>; // a uniform matrix for transformations (this could be a ModelViewProjection matrix)

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Applying transformation to the vertex positions
    output.clip_position = transform * vec4<f32>(input.position, 1.0);

    // Setting world position, normal, and uv coordinates for the fragment shader
    output.world_position = output.clip_position; // This could be modified based on your needs
    output.world_normal = vec3<f32>(0.0, 0.0, 1.0); // Setting a default normal vector
    output.uv = input.uv; // Passing the UV coordinates directly to the fragment shader

    return output;
}