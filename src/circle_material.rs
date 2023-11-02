use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::ARRAY_UNIFORM_SIZE;

#[derive(Resource)]
pub struct CircleEntity(pub Option<Entity>);
impl Default for CircleEntity {
    fn default() -> Self {
        CircleEntity(None)
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "00adadef-a1e7-4601-9169-87493ce3fa5c"]
pub struct CircleMaterial {
    #[uniform(0)]
    pub normalized_data: [Vec4; ARRAY_UNIFORM_SIZE], // Use an array of vec4s (which is an array of [f32; 4] in Rust)}
    #[uniform(1)]
    pub viewport_width: f32,
    #[uniform(2)]
    pub viewport_height: f32,
}
impl Material2d for CircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_fragment.wgsl".into()
    }
}
//a Mandelbrot material with the given uniforms.
pub fn prepare_circle_material(
    materials: &mut ResMut<Assets<CircleMaterial>>,
    width: f32,
    height: f32,
) -> Handle<CircleMaterial> {
    let material = CircleMaterial {
        normalized_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        viewport_width: width,
        viewport_height: height,
    };
    materials.add(material)
}
