use crate::utils::texture::*;
use dashi::utils::Handle;
use dashi::*;

use crate::database::*;

#[repr(C)]
pub struct Material {
    base_color: Handle<Image>,
    specular: Handle<Image>,
}

pub struct Mesh {
    pub name: String,
    pub vertices: Handle<Buffer>,
    pub indices: Handle<Buffer>,
    pub material: Material,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}
