use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    io::Read,
};

use common::Hotbuffer;
use dashi::{
    utils::{Handle, Pool},
    Attachment, BindGroup, BindGroupLayout, BindGroupLayoutInfo, BindGroupVariable,
    BindGroupVariableType, BindlessBindGroupLayoutInfo, Buffer, Context, CullMode, FRect2D, Format,
    GraphicsPipeline, GraphicsPipelineDetails, GraphicsPipelineInfo, GraphicsPipelineLayout,
    GraphicsPipelineLayoutInfo, Image, ImageInfo, ImageView, ImageViewInfo, IndexedBindGroupInfo,
    IndexedBindingInfo, IndexedResource, PipelineShaderInfo, Rect2D, RenderPass, RenderPassInfo,
    Sampler, ShaderInfo, ShaderResource, ShaderType, VertexDescriptionInfo, VertexEntryInfo,
    VertexOrdering, Viewport,
};
use glam::*;
use inline_spirv::include_spirv;
use pass::RenderManager;

pub mod algorithm;
mod common;
pub mod graph;
mod json;
mod pass;

struct ResourceList<T> {
    pub pool: Pool<T>,
    pub entries: Vec<Handle<T>>,
}

impl<T> ResourceList<T> {
    pub fn new(size: usize) -> Self {
        Self {
            pool: Pool::new(size),
            entries: Vec::with_capacity(size),
        }
    }

    pub fn push(&mut self, v: T) -> Handle<T> {
        let h = self.pool.insert(v).unwrap();
        self.entries.push(h);
        h
    }

    pub fn release(&mut self, h: Handle<T>) {
        if let Some(idx) = self.entries.iter().position(|a| a.slot == h.slot) {
            self.entries.remove(idx);
            self.pool.release(h);
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get_ref(&self, h: Handle<T>) -> &T {
        self.pool.get_ref(h).unwrap()
    }

    pub fn get_ref_mut(&mut self, h: Handle<T>) -> &mut T {
        self.pool.get_mut_ref(h).unwrap()
    }

    pub fn for_each_occupied<F>(&self, func: F)
    where
        F: Fn(&T),
    {
        for item in &self.entries {
            let r = self.pool.get_ref(item.clone()).unwrap();
            func(r);
        }
    }

    pub fn for_each_handle<F>(&self, mut func: F)
    where
        F: FnMut(Handle<T>),
    {
        for h in &self.entries {
            func(*h);
        }
    }

    pub fn for_each_occupied_mut<F>(&mut self, mut func: F)
    where
        F: FnMut(&T),
    {
        for item in &self.entries {
            let r = self.pool.get_mut_ref(item.clone()).unwrap();
            func(r);
        }
    }
}

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
            color_attachments: &color_attachments
                [color_offset..config_subpass.attachments.len() - 1],
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
    //
    let format = if attachment.kind.to_lowercase() == "color" {
        dashi::Format::RGBA8
    } else if attachment.kind.to_lowercase() == "depth" {
        dashi::Format::D24S8
    } else {
        dashi::Format::RGBA8
    };

    let image = ctx
        .make_image(&ImageInfo {
            debug_name: &attachment.name,
            dim: [attachment.size[0], attachment.size[1], 1],
            layers: 1,
            format,
            mip_levels: 1,
            initial_data: None,
        })
        .unwrap();

    let view = ctx
        .make_image_view(&ImageViewInfo {
            debug_name: &attachment.name,
            img: image,
            ..Default::default()
        })
        .unwrap();

    (image, view)
}

pub struct MisoSceneInfo {
    pub cfg: String,
}

#[repr(C)]
#[derive(Default)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec4,
    pub tex_coords: Vec2,
    pub joint_ids: IVec4,
    pub joints: Vec4,
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

#[derive(Default)]
pub struct MaterialInfo {
    pub name: String,
    pub passes: Vec<String>,
    pub base_color: Handle<Texture>,
    pub normal: Handle<Texture>,
}

#[derive(Default)]
pub struct Material {
    base_color: Handle<Texture>,
    normal: Handle<Texture>,
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

#[derive(Default)]
struct PassObject {
    original: Handle<Renderable>,
}

#[derive(Default)]
struct MisoIndirectBatch {
    handle: Handle<Renderable>,
    first: u32,
    count: u32,
}

#[derive(Default)]
struct MisoMultiIndirectBatch {
    first: u32,
    count: u32,
}

#[derive(Default)]
struct MisoBatch {
    handle: Handle<PassObject>,
    sort_key: u64,
}

#[derive(Default)]
struct MisoRenderImage {
    img: Handle<Image>,
    view: Handle<ImageView>,
}

#[derive(Default)]
struct Pipeline {
    gfx_layout: Handle<GraphicsPipelineLayout>,
    gfx: Handle<GraphicsPipeline>,
    per_pipeline_bg: Handle<BindGroup>,
}

#[derive(Default)]
struct MisoSubpass {
    name: String,
    pipeline: Pipeline,
    per_pipeline_bg: Handle<BindGroup>,
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
        for camera in &cfg.cameras {}
    }
}

pub struct TextureInfo {
    pub image: Handle<Image>,
    pub view: Handle<ImageView>,
    pub sampler: Handle<Sampler>,
    pub dim: [u32; 2],
}

pub struct Texture {
    handle: Handle<Image>,
    view: Handle<ImageView>,
    sampler: Handle<Sampler>,
    dim: [u32; 2],
}

#[derive(Default)]
struct RendererInfo {
    bindless_bg_layout: Handle<BindGroupLayout>,
    bg_layout: Handle<BindGroupLayout>,
    bindless_bg: Handle<BindGroup>,
}

#[derive(Default)]
pub struct DeletionQueue<T> {
    queue: VecDeque<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T> DeletionQueue<T> {
    /// Creates a new, empty `DeletionQueue`.
    pub fn new() -> Self {
        DeletionQueue {
            queue: VecDeque::new(),
        }
    }

    /// Adds a deletion operation to the queue.
    ///
    /// # Arguments
    /// * `operation` - A closure or function that takes no arguments and returns a value of type `T`.
    pub fn push<F>(&mut self, operation: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.queue.push_back(Box::new(operation));
    }

    /// Processes all operations in the queue and clears it.
    ///
    /// Returns a `Vec<T>` containing the results of all processed operations.
    pub fn delete_all(&mut self) -> Vec<T> {
        let mut results = Vec::new();

        while let Some(operation) = self.queue.pop_front() {
            results.push(operation());
        }

        results
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of operations currently in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[derive(Default)]
struct Deletion {
    tex: DeletionQueue<Handle<Texture>>,
}

#[derive(Default)]
struct MisoBGLayouts {
    bindless: Handle<BindGroupLayout>,
    per_pipeline: Handle<BindGroupLayout>,
    per_frame: Handle<BindGroupLayout>,
    per_object: Handle<BindGroupLayout>,
}

pub struct MisoScene {
    ctx: *mut Context,
    delete: Deletion,
    textures: ResourceList<Texture>,
    meshes: Pool<Mesh>,
    materials: Pool<Material>,
    renderables: Pool<Renderable>,
    render_pass: MisoRenderPass,
    bg_layouts: MisoBGLayouts,
    dirty: bool,
}

impl MisoScene {
    pub fn new(ctx: &mut dashi::Context, info: &MisoSceneInfo) -> Self {
        let json_data = fs::read_to_string(info.cfg.clone()).expect("Unable to read file");
        let cfg: json::Config = serde_json::from_str(&json_data).unwrap();
        let mut rp = make_rp(ctx, &cfg);

        let mut s = Self {
            ctx,
            delete: Default::default(),
            meshes: Default::default(),
            materials: Default::default(),
            renderables: Default::default(),
            dirty: false,
            render_pass: make_rp(ctx, &cfg),
            textures: ResourceList::new(1024),
            bg_layouts: Default::default(),
        };

        s.make_bind_group_layouts();
        s.make_cameras(&cfg);
        s.make_pipelines(&cfg);
        s
    }
    fn make_cameras(&mut self, cfg: &json::Config) {
        for pipe in &cfg.passes {}
    }

    fn load_shader_from_file(path: &str) -> Vec<u32> {
        // Open the file
        let mut file = File::open(path).unwrap();

        // Read the file into a byte buffer
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        // Ensure the byte buffer's length is a multiple of 4
        assert!(buffer.len() % 4 == 0);

        // Convert the byte buffer into a Vec<u32>
        let word_count = buffer.len() / 4;
        let mut byte_code = Vec::with_capacity(word_count);
        for chunk in buffer.chunks_exact(4) {
            let word = u32::from_le_bytes(chunk.try_into().unwrap());
            byte_code.push(word);
        }

        byte_code
    }

    fn make_bind_group_layouts(&mut self) {
        // Bindless BG contains all textures.
        let bindless = self
            .get_ctx()
            .make_bind_group_layout(&BindGroupLayoutInfo {
                debug_name: "[MISO] Bindless Bind Group Layout",
                shaders: &[ShaderInfo {
                    shader_type: ShaderType::All,
                    variables: &[BindGroupVariable {
                        var_type: BindGroupVariableType::SampledImage,
                        binding: 0,
                    }],
                }],
            })
            .unwrap();

        // Global Bindings. These describe the environment, rendering settings, etc.
        let per_pipeline = self
            .get_ctx()
            .make_bind_group_layout(&BindGroupLayoutInfo {
                debug_name: "[MISO] Global Bind Group Layout",
                shaders: &[ShaderInfo {
                    shader_type: ShaderType::All,
                    variables: &[BindGroupVariable {
                        var_type: BindGroupVariableType::Uniform,
                        binding: 1,
                    }],
                }],
            })
            .unwrap();

        // Bindings for per-frame data. Camera, transformations, etc.
        let per_frame = self
            .get_ctx()
            .make_bind_group_layout(&BindGroupLayoutInfo {
                debug_name: "[MISO] Per-Frame Bind Group Layout",
                shaders: &[ShaderInfo {
                    shader_type: ShaderType::All,
                    variables: &[BindGroupVariable {
                        var_type: BindGroupVariableType::Uniform,
                        binding: 2,
                    }],
                }],
            })
            .unwrap();

        // Bindings per render. These should be just dynamic buffer data.
        let per_object = self
            .get_ctx()
            .make_bind_group_layout(&BindGroupLayoutInfo {
                debug_name: "[MISO] Per-Frame Bind Group Layout",
                shaders: &[ShaderInfo {
                    shader_type: ShaderType::All,
                    variables: &[BindGroupVariable {
                        var_type: BindGroupVariableType::DynamicUniform,
                        binding: 3,
                    }],
                }],
            })
            .unwrap();

        self.bg_layouts = MisoBGLayouts {
            bindless,
            per_pipeline,
            per_frame,
            per_object,
        };
    }

    fn make_pipelines(&mut self, cfg: &json::Config) {
        let stdvert = include_spirv!("target/spirv/stdvert.spv");
        let stdfrag = include_spirv!("target/spirv/stdvert.spv");

        for pipe in &cfg.passes {
            let (vshader, pshader) = match pipe.graphics[0].as_str() {
                "standard" => (stdvert.as_slice(), stdfrag.as_slice()),
                _ => todo!(),
            };
            let bg_layouts = [
                Some(self.bg_layouts.bindless),
                Some(self.bg_layouts.per_pipeline),
                Some(self.bg_layouts.per_frame),
                Some(self.bg_layouts.per_object),
            ];

            let layout = self
                .get_ctx()
                .make_graphics_pipeline_layout(&GraphicsPipelineLayoutInfo {
                    debug_name: &pipe.name,
                    vertex_info: VertexDescriptionInfo {
                        entries: &[
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 0,
                                offset: 0,
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 1,
                                offset: 0 + 16,
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec2,
                                location: 2,
                                offset: 0 + 16 + 16,
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 3,
                                offset: 0 + 16 + 16 + 8,
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 4,
                                offset: 0 + 16 + 16 + 8 + 16,
                            },
                        ],
                        stride: std::mem::size_of::<Vertex>(),
                        rate: dashi::VertexRate::Vertex,
                    },
                    bg_layouts,
                    shaders: &[
                        PipelineShaderInfo {
                            stage: ShaderType::Vertex,
                            spirv: &vshader,
                            specialization: &[],
                        },
                        PipelineShaderInfo {
                            stage: ShaderType::Fragment,
                            spirv: &pshader,
                            specialization: &[],
                        },
                    ],
                    details: GraphicsPipelineDetails {
                        topology: dashi::Topology::TriangleList,
                        culling: CullMode::None,
                        front_face: VertexOrdering::Clockwise,
                        depth_test: false,
                    },
                })
                .unwrap();

            let rp = self.render_pass.handle;
            let pipeline = self
                .get_ctx()
                .make_graphics_pipeline(&GraphicsPipelineInfo {
                    debug_name: &pipe.name,
                    layout,
                    render_pass: rp,
                    subpass_id: 0, // JHTODO prob should be configurable
                })
                .unwrap();

            self.render_pass.subpasses.push(MisoSubpass {
                name: pipe.name.clone(),
                pipeline: Pipeline {
                    gfx_layout: layout,
                    gfx: pipeline,
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }

    pub fn register_texture(&mut self, info: &TextureInfo) -> Handle<Texture> {
        return self.textures.push(Texture {
            handle: info.image,
            sampler: info.sampler,
            dim: info.dim,
            view: info.view,
        });
    }

    pub fn unregister_texture(&mut self, h: Handle<Texture>) {
        self.textures.release(h);
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
                base_color: info.base_color,
                normal: info.normal,
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

    fn get_ctx(&mut self) -> &mut Context {
        unsafe { &mut *(self.ctx) }
    }

    fn reconfigure_scene(&mut self) {
        const BINDLESS_SET: u32 = 0;
        let mut bindings = Vec::new();
        self.textures.for_each_handle(|h| {
            let t = self.textures.get_ref(h);
            bindings.push(IndexedResource {
                resource: ShaderResource::SampledImage(t.view, t.sampler),
                slot: h.slot as u32,
            });
        });

        let bindless = self.bg_layouts.bindless;
        let bg = self
            .get_ctx()
            .make_indexed_bind_group(&IndexedBindGroupInfo {
                debug_name: "[MISO] Bindless Bind Group",
                layout: bindless,
                bindings: &[IndexedBindingInfo {
                    resources: &bindings,
                    binding: 0,
                }],
                set: BINDLESS_SET,
            });
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.reconfigure_scene();
            self.dirty = false;
        }

        self.delete.tex.delete_all();
    }
}
