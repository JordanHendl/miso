use dashi::utils::Pool;

use crate::{Camera, Material, Mesh, Renderable, ResourceList, Texture};

#[derive(Default)]
pub struct ResourceManager {
    pub cameras: ResourceList<Camera>,
    pub meshes: Pool<Mesh>,
    pub materials: Pool<Material>,
    pub textures: ResourceList<Texture>,
    pub renderables: Pool<Renderable>,
}
