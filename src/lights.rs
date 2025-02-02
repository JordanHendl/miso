use dashi::{
    utils::gpupool::GPUPool,
    BufferInfo, BufferUsage, MemoryVisibility,
};
use glam::*;


#[allow(dead_code)]
pub struct GPUOption<T> {
    data: T,
    available: u32,
    padding: [u32; 3],
}

impl<T> GPUOption<T> {
    pub fn new(s: T) -> Self {
        Self {
            data: s,
            available: 1,
            ..Default::default()
        }
    }
}

impl<T> Default for GPUOption<T> {
    fn default() -> Self {
        Self {
            data: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            available: 0,
            padding: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct DirectionalLightInfo {
    pub direction: Vec4,
    pub color: Vec4,
    pub intensity: f32,
}

#[repr(C)]
#[derive(Default)]
pub struct ShaderDirectionalLight {
    pub direction: Vec4,
    pub color: Vec4,
    pub intensity: f32,
}

#[repr(C)]
#[derive(Default)]
pub struct Light {
    pub(crate) dir: GPUOption<ShaderDirectionalLight>,
}

impl From<&DirectionalLightInfo> for ShaderDirectionalLight {
    fn from(value: &DirectionalLightInfo) -> Self {
        Self {
            direction: value.direction,
            color: value.color,
            intensity: value.intensity,
        }
    }
}

pub type DirectionalLight = Light;
#[derive(Default)]
pub struct LightCollection {
    pub(crate) lights: GPUPool<Light>,
}

impl LightCollection {
    pub fn new(ctx: &mut dashi::Context) -> Self {
        const NUM_LIGHTS: usize = 1024;
        let lights: GPUPool<Light> = GPUPool::new(
            ctx,
            &BufferInfo {
                debug_name: "[MISO] Light List",
                byte_size: std::mem::size_of::<Light>() as u32 * NUM_LIGHTS as u32,
                visibility: MemoryVisibility::Gpu,
                usage: BufferUsage::STORAGE,
                initial_data: None,
            },
        );

        Self { lights }
    }
}
