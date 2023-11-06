use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::ARRAY_UNIFORM_SIZE;

#[derive(Resource)]
pub struct StringEntity(pub Option<Entity>);
impl Default for StringEntity {
    fn default() -> Self {
        StringEntity(None)
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "fcf0ff0e-23f6-41f9-98a2-896a7407c235"]
pub struct StringMaterial {
    #[uniform(0)]
    pub left_data: [Vec4; ARRAY_UNIFORM_SIZE], // Use an array of vec4s (which is an array of [f32; 4] in Rust)}
    #[uniform(1)]
    pub right_data: [Vec4; ARRAY_UNIFORM_SIZE], // Use an array of vec4s (which is an array of [f32; 4] in Rust)}
    #[uniform(2)]
    pub viewport_width: f32,
    #[uniform(3)]
    pub viewport_height: f32,
    #[uniform(4)]
    pub monochrome: u32,
    #[uniform(5)]
    pub colors: [Vec4; 4],
}
impl Material2d for StringMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/string_fragment.wgsl".into()
    }
}
//a Mandelbrot material with the given uniforms.
pub fn prepare_string_material(
    materials: &mut ResMut<Assets<StringMaterial>>,
    width: f32,
    height: f32,
) -> Handle<StringMaterial> {
    let material = StringMaterial {
        left_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        right_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        viewport_width: width,
        viewport_height: height,
        monochrome: 0,
        colors: [
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::ZERO,
        ],
        //color_start: [0f32; 4],
        //color_middle: [0f32; 3],
        //color_end: [0f32; 3],
    };
    materials.add(material)
}
