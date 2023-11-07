use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::ARRAY_UNIFORM_SIZE;

#[derive(Resource)]
pub struct AudioEntity(pub Option<Entity>);
impl Default for AudioEntity {
    fn default() -> Self {
        AudioEntity(None)
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath, Asset)]
#[uuid = "e71681d9-3499-4bba-881d-2eaeed7c1c31"]
pub struct AudioMaterial {
    #[uniform(0)]
    pub normalized_data: [Vec4; ARRAY_UNIFORM_SIZE], // Use an array of vec4s (which is an array of [f32; 4] in Rust)}
    #[uniform(1)]
    pub viewport_width: f32,
    #[uniform(2)]
    pub viewport_height: f32,
    #[uniform(4)]
    pub monochrome: u32,
    #[uniform(5)]
    pub colors: [Vec4; 4],
}
impl Material2d for AudioMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/audio_fragment.wgsl".into()
    }
}
//a Mandelbrot material with the given uniforms.
pub fn prepare_audio_material(
    materials: &mut ResMut<Assets<AudioMaterial>>,
    width: f32,
    height: f32,
) -> Handle<AudioMaterial> {
    let material = AudioMaterial {
        normalized_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); ARRAY_UNIFORM_SIZE],
        viewport_width: width,
        viewport_height: height,
        monochrome: 1,
        colors: [
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::ZERO,
        ],
    };
    materials.add(material)
}
