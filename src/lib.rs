use std::{collections::HashMap, fs};

use common::Hotbuffer;
use dashi::{
    utils::{Handle, Pool},
    Attachment, BindGroup, Buffer, Context, FRect2D, GraphicsPipeline, Image, ImageView, Rect2D,
    RenderPass, RenderPassInfo, Viewport,
};
use glam::*;
use pass::RenderManager;

pub mod algorithm;
mod common;
pub mod graph;
mod json;
mod pass;

fn make_rp(ctx: &mut dashi::Context, config: &json::Config) -> MisoRenderPass {
    let mut subpasses = Vec::with_capacity(512);
    let mut color_attachments = Vec::with_capacity(1024);
    let mut depth_attachments = Vec::with_capacity(1024);
    let mut images = HashMap::new();
    // Map subpasses from Config to the rendering library's Subpass type

    for config_subpass in &config.render_pass.subpasses {
        // Map color attachments
        for attachment in &config_subpass.attachments {
            let (img, view) = create_view_from_attachment(ctx, attachment);
            images.insert(attachment.name.clone(), MisoRenderImage { img, view });
            if attachment.kind == "Color" {
                color_attachments.push(dashi::Attachment {
                    view,
                    clear_color: [0.0, 0.0, 0.0, 1.0], // Default clear color
                    ..Default::default()
                });
            } else if attachment.kind == "Depth" {
                depth_attachments.push(Attachment {
                    view,
                    clear_color: [0.0, 0.0, 0.0, 0.0], // Depth attachments typically don't use clear colors
                    ..Default::default()
                });
            }
        }
    }

    let mut color_offset = 0;
    let mut depth_offset = 0;
    for config_subpass in &config.render_pass.subpasses {
        let d: Vec<&json::Attachment> = config_subpass
            .attachments
            .iter()
            .filter(|kind| return kind.kind == "Depth")
            .collect();

        let dep = if d.is_empty() {
            None
        } else {
            let t = depth_offset;
            depth_offset += 1;
            Some(&depth_attachments[t])
        };

        subpasses.push(dashi::Subpass {
            color_attachments: &color_attachments[color_offset..config_subpass.attachments.len()],
            depth_stencil_attachment: dep,
            subpass_dependencies: &[], // Add dependencies if needed [JHTODO]
        });

        color_offset += config_subpass.attachments.len();
    }

    // Use the first attachment to configure the viewport and scissor area
    let first_attachment_size = config.render_pass.subpasses[0].attachments[0].size;

    // Create the render pass using the configuration
    let render_pass = ctx
        .make_render_pass(&RenderPassInfo {
            viewport: Viewport {
                area: FRect2D {
                    w: first_attachment_size[0] as f32,
                    h: first_attachment_size[1] as f32,
                    ..Default::default()
                },
                scissor: Rect2D {
                    w: first_attachment_size[0],
                    h: first_attachment_size[1],
                    ..Default::default()
                },
                ..Default::default()
            },
            subpasses: &subpasses,
            debug_name: "renderpass",
        })
        .unwrap();

    MisoRenderPass {
        handle: render_pass,
        images,
        cameras: Default::default(),
        camera_pool: Default::default(),
        subpasses: Default::default(),
    }
}

// Helper function to create a view for an attachment
fn create_view_from_attachment(
    ctx: &mut Context,
    attachment: &json::Attachment,
) -> (Handle<Image>, Handle<ImageView>) {
    // Placeholder: Replace this with actual logic to create a view
    //    RenderView::new(attachment.name.clone())
    todo!()
}

pub struct MisoSceneInfo {
    pub cfg: String,
}

pub struct MeshInfo {
    pub name: String,
    pub vertices: Handle<Buffer>,
    pub num_vertices: usize,
    pub indices: Handle<Buffer>,
    pub num_indices: usize,
}

pub struct Mesh {
    vertices: Handle<Buffer>,
    num_vertices: u32,
    indices: Handle<Buffer>,
    num_indices: u32,
    first_index: u32,
}

pub enum TextureType {
    BaseColor,
    Normal,
}

pub struct MaterialInfo {
    pub name: String,
    pub passes: Vec<String>,
    pub textures: HashMap<TextureType, Handle<Image>>,
}

pub struct Material {
    base_color: Handle<Image>,
    normal: Handle<Image>,
}

pub struct ObjectInfo {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub transform: Mat4,
}

pub struct Renderable {
    mesh: Handle<Mesh>,
    material: Handle<Material>,
    transform: Mat4,
}

pub struct CameraInfo {}

pub struct Camera {}

struct PassObject {
    original: Handle<Renderable>,
}

struct MisoIndirectBatch {
    handle: Handle<Renderable>,
    first: u32,
    count: u32,
}

struct MisoMultiIndirectBatch {
    first: u32,
    count: u32,
}

struct MisoBatch {
    handle: Handle<PassObject>,
    sort_key: u64,
}

struct MisoRenderImage {
    img: Handle<Image>,
    view: Handle<ImageView>,
}

struct MisoSubpass {
    name: String,
    pipeline: Handle<GraphicsPipeline>,
    bind_groups: [Option<Handle<BindGroup>>; 4],
    non_batched: Vec<MisoBatch>,
    objects: Pool<PassObject>,
    objects_to_add: Vec<Handle<Renderable>>,
}

struct MisoRenderPass {
    handle: Handle<RenderPass>,
    images: HashMap<String, MisoRenderImage>,
    cameras: HashMap<String, Handle<Camera>>,
    camera_pool: Pool<Camera>,
    subpasses: Vec<MisoSubpass>,
}

impl MisoRenderPass {
    fn parse_cameras(&mut self, cfg: &json::Config) {
       for camera in &cfg.cameras {
            
       }
    }
}

pub struct MisoScene {
    ctx: *mut Context,
    passes: RenderManager,
    meshes: Pool<Mesh>,
    materials: Pool<Material>,
    renderables: Pool<Renderable>,
    render_pass: MisoRenderPass,
    dirty: bool,
}

impl MisoScene {
    pub fn new(ctx: &mut dashi::Context, info: &MisoSceneInfo) -> Self {
        let json_data = fs::read_to_string(info.cfg.clone()).expect("Unable to read file");
        let cfg: json::Config = serde_json::from_str(&json_data).unwrap();
        let mut rp = make_rp(ctx, &cfg);
        
        Self {
            ctx,
            passes: RenderManager::new(&info.cfg),
            meshes: Default::default(),
            materials: Default::default(),
            renderables: Default::default(),
            dirty: false,
            render_pass: make_rp(ctx, &cfg),
        }
    }

    pub fn register_camera(&mut self, info: &CameraInfo) -> Handle<Camera> {
        todo!()
    }

    pub fn unregister_camera(&mut self, h: Handle<Camera>) {
        todo!()
    }

    pub fn register_mesh(&mut self, info: &MeshInfo) -> Handle<Mesh> {
        self.dirty = true;

        self.meshes
            .insert(Mesh {
                vertices: info.vertices,
                indices: info.indices,
                num_vertices: info.num_vertices as u32,
                num_indices: info.num_indices as u32,
                first_index: 0,
            })
            .unwrap()
    }

    pub fn register_meshes(&mut self, infos: &[MeshInfo]) -> Vec<Handle<Mesh>> {
        let mut o = Vec::with_capacity(infos.len());
        for m in infos {
            o.push(self.register_mesh(m));
        }

        o
    }

    pub fn unregister_mesh(&mut self, handle: Handle<Mesh>) {
        self.dirty = true;
        self.meshes.release(handle);
    }

    pub fn register_material(&mut self, info: &MaterialInfo) -> Handle<Material> {
        self.dirty = true;
        self.materials
            .insert(Material {
                base_color: todo!(),
                normal: todo!(),
            })
            .unwrap()
    }

    pub fn unregister_material(&mut self, handle: Handle<Material>) {
        self.dirty = true;
        self.materials.release(handle);
    }

    pub fn register_object(&mut self, info: &ObjectInfo) -> Handle<Renderable> {
        self.renderables
            .insert(Renderable {
                mesh: info.mesh,
                material: info.material,
                transform: info.transform,
            })
            .unwrap()
    }

    pub fn unregister_object(&mut self, handle: Handle<Renderable>) {
        self.renderables.release(handle);
    }

    fn reconfigure_scene(&mut self) {}

    pub fn update(&mut self) {
        if self.dirty {
            self.reconfigure_scene();
            self.dirty = false;
        }

        todo!()
    }
}
