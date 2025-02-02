use dashi::utils::*;
use dashi::*;
use glam::*;
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec4,
    pub color: Vec4,
    pub tex_coords: Vec2,
    pub joint_ids: IVec4,
    pub joints: Vec4,
}

pub struct TextureInfo {
    pub image: Handle<Image>,
    pub view: Handle<ImageView>,
    pub sampler: Handle<Sampler>,
    pub dim: [u32; 2],
}

#[allow(dead_code)]
pub struct Texture {
    pub(crate) handle: Handle<Image>,
    pub(crate) view: Handle<ImageView>,
    pub(crate) sampler: Handle<Sampler>,
    pub(crate) dim: [u32; 2],
}

pub struct MeshInfo {
    pub name: String,
    pub vertices: Handle<Buffer>,
    pub num_vertices: usize,
    pub indices: Handle<Buffer>,
    pub num_indices: usize,
}

#[allow(dead_code)]
pub struct Mesh {
    pub(crate) vertices: Handle<Buffer>,
    pub(crate) num_vertices: u32,
    pub(crate) indices: Handle<Buffer>,
    pub(crate) num_indices: u32,
    pub(crate) first_index: u32,
}

#[derive(Default)]
pub struct MaterialInfo {
    pub name: String,
    pub passes: Vec<String>,
    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub base_color: Handle<Texture>,
    pub normal: Handle<Texture>,
    pub emissive: Handle<Texture>,
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct MaterialShaderData {
    pub(crate) base_color_factor: Vec4,
    pub(crate) emissive_factor: Vec4,
    pub(crate) base_color: Handle<Texture>,
    pub(crate) normal: Handle<Texture>,
    pub(crate) emissive: Handle<Texture>,
}

impl From<&MaterialInfo> for MaterialShaderData {
    fn from(value: &MaterialInfo) -> Self {
        Self {
            base_color_factor: value.base_color_factor,
            emissive_factor: value.emissive_factor,
            base_color: value.base_color,
            normal: value.normal,
            emissive: value.emissive,
        }
    }
}
#[derive(Default)]
pub struct Material {
    pub(crate) data: MaterialShaderData,
    pub(crate) passes: Vec<String>,
}

pub struct ObjectInfo {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub transform: Mat4,
}

pub struct CameraInfo<'a> {
    pub pass: &'a str,
    pub transform: Mat4,
    pub projection: Mat4,
}

pub struct Camera {
    pub(crate) transform: Mat4,
    pub(crate) projection: Mat4,
}

pub struct Renderable {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) material: Handle<Material>,
    pub(crate) transform: Mat4,
}
