use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::ARRAY_UNIFORM_SIZE;

#[derive(Resource)]
pub struct CircleSplitEntity(pub Option<Entity>);
impl Default for CircleSplitEntity {
    fn default() -> Self {
        CircleSplitEntity(None)
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "a3dafd0f-45ef-4d05-9a78-e309a208859b"]
pub struct CircleSplitMaterial {
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
    //#[uniform(5)]
    //pub color_start: [f32; 4],
    //#[uniform(6)]
    //pub color_middle: [f32; 3],
    //#[uniform(7)]
    //pub color_end: [f32; 3],
}
impl Material2d for CircleSplitMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_split_fragment.wgsl".into()
    }
}
//a Mandelbrot material with the given uniforms.
pub fn prepare_circle_split_material(
    materials: &mut ResMut<Assets<CircleSplitMaterial>>,
    width: f32,
    height: f32,
) -> Handle<CircleSplitMaterial> {
    let material = CircleSplitMaterial {
        left_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        right_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        viewport_width: width,
        viewport_height: height,
        monochrome: 1,
        //color_start: [0f32; 4],
        //color_middle: [0f32; 3],
        //color_end: [0f32; 3],
    };
    materials.add(material)
}
