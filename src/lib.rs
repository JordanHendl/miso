pub use sdl2::{event::Event, keyboard::Keycode};
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    io::Read,
};

use common::Hotbuffer;
use dashi::{
    utils::{per_frame::PerFrame, Handle, Pool},
    Attachment, AttachmentDescription, BindGroup, BindGroupInfo, BindGroupLayout,
    BindGroupLayoutInfo, BindGroupVariable, BindGroupVariableType, BindlessBindGroupLayoutInfo,
    Buffer, BufferUsage, CommandListInfo, Context, CullMode, Display, DisplayInfo, DrawBegin,
    DrawIndexed, DynamicAllocator, DynamicAllocatorInfo, FRect2D, Filter, Format,
    FramedCommandList, GraphicsPipeline, GraphicsPipelineDetails, GraphicsPipelineInfo,
    GraphicsPipelineLayout, GraphicsPipelineLayoutInfo, Image, ImageBlit, ImageInfo, ImageView,
    ImageViewInfo, IndexedBindGroupInfo, IndexedBindingInfo, IndexedResource, PipelineShaderInfo,
    Rect2D, RenderPass, RenderPassInfo, Sampler, Semaphore, ShaderInfo, ShaderResource, ShaderType,
    SubmitInfo, Subpass, SubpassDescription, VertexDescriptionInfo, VertexEntryInfo,
    VertexOrdering, Viewport, WindowInfo,
};
use glam::*;
use inline_spirv::include_spirv;
use pass::RenderManager;

pub mod algorithm;
mod common;
pub mod graph;
mod json;
mod pass;
mod reflection;

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
                color_attachments.push(dashi::AttachmentDescription {
                    ..Default::default()
                });
            } else if attachment.kind == "Depth" {
                depth_attachments.push(AttachmentDescription {
                    format: Format::D24S8,
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

        subpasses.push(SubpassDescription {
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

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct MaterialShaderData {
    base_color: Handle<Texture>,
    normal: Handle<Texture>,
}

#[derive(Default)]
pub struct Material {
    data: MaterialShaderData,
    passes: Vec<String>,
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

pub struct CameraInfo<'a> {
    pass: &'a str,
    transform: Mat4,
    projection: Mat4,
}

pub struct Camera {
    transform: Mat4,
    projection: Mat4,
}

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
    camera: Handle<Camera>,
    per_pipeline_bg: Handle<BindGroup>,
    non_batched: Vec<MisoBatch>,
    objects: Pool<PassObject>,
    reflection: reflection::ShaderInspector,
    bg_layouts: [Option<Handle<BindGroupLayout>>; 4],
    objects_to_add: Vec<Handle<Renderable>>,
}

#[derive(Default)]
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

impl<T: Clone> Clone for DeletionQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Default::default(),
        }
    }
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

#[derive(Default, Clone)]
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

#[derive(Default, Clone)]
struct SubpassAttachments {
    name: String,
    colors: Vec<Attachment>,
    depth: Option<Attachment>,
}

#[derive(Default, Clone)]
struct PerFrameResources {
    subpasses: Vec<SubpassAttachments>,
    out_image: Handle<ImageView>,
    delete_queue: Deletion,
    sems: Vec<Handle<Semaphore>>,
}

struct GlobalResources {
    ctx: *mut Context,
    subpasses: Vec<MisoSubpass>,
    cameras: ResourceList<Camera>,
    dynamic: DynamicAllocator,
    meshes: Pool<Mesh>,
    materials: Pool<Material>,
    textures: ResourceList<Texture>,
    renderables: Pool<Renderable>,
    render_pass: MisoRenderPass,
    bg_layouts: MisoBGLayouts,
    bindless: Handle<BindGroup>,
    display: Option<Display>,
}

impl Default for GlobalResources {
    fn default() -> Self {
        const RESOURCE_LIST_SZ: usize = 1024;
        Self {
            ctx: std::ptr::null_mut(),
            subpasses: Default::default(),
            cameras: ResourceList::new(RESOURCE_LIST_SZ),
            textures: ResourceList::new(RESOURCE_LIST_SZ),
            meshes: Pool::new(RESOURCE_LIST_SZ),
            materials: Default::default(),
            renderables: Default::default(),
            render_pass: Default::default(),
            bg_layouts: Default::default(),
            display: Option::default(),
            bindless: Default::default(),
            dynamic: Default::default(),
        }
    }
}

pub struct MisoScene {
    global_res: GlobalResources,
    frame: PerFrame<PerFrameResources>,
    draw_cmd: FramedCommandList,
    dirty: bool,
}

impl MisoScene {
    pub fn new(ctx: &mut dashi::Context, info: &MisoSceneInfo) -> Self {
        let json_data = fs::read_to_string(info.cfg.clone()).expect("Unable to read file");
        let cfg: json::Config = serde_json::from_str(&json_data).unwrap();
        let mut rp = make_rp(ctx, &cfg);

        let mut s = Self {
            dirty: false,
            global_res: GlobalResources {
                ctx,
                render_pass: rp,
                ..Default::default()
            },
            frame: PerFrame::new(2),
            draw_cmd: FramedCommandList::new(ctx, "[MISO] Main Draw Command List", 3),
        };

        s.global_res.dynamic = ctx
            .make_dynamic_allocator(&DynamicAllocatorInfo {
                debug_name: "[MISO] Per-Object Dynamic Allocator",
                usage: BufferUsage::ALL,
                num_allocations: 16000,
                byte_size: 16000 * 256,
                ..Default::default()
            })
            .unwrap();

        s.make_per_frame_attachments(&cfg);
        s.make_bind_group_layouts();
        s.make_display(&cfg);
        s.make_cameras(&cfg);
        s.make_pipelines(&cfg);
        s
    }

    fn make_cameras(&mut self, cfg: &json::Config) {
        for pipe in &cfg.passes {}
    }

    fn make_per_frame_attachments(&mut self, cfg: &json::Config) {
        let ctx = self.global_res.ctx;
        self.frame.for_each_mut(|f| {
            for subpass in &cfg.render_pass.subpasses {
                let mut colors = Vec::new();
                let mut depth = None;

                for attach in &subpass.attachments {
                    let full_name = format!("{}.{}", subpass.name, attach.name);
                    let (img, view) = create_view_from_attachment(unsafe { &mut *(ctx) }, attach);
                    match attach.kind.to_lowercase().as_str() {
                        "color" => {
                            colors.push(Attachment {
                                img: view,
                                clear_color: Default::default(),
                            });
                        }
                        "depth" => {
                            depth = Some(Attachment {
                                img: view,
                                clear_color: [1.0, 1.0, 1.0, 1.0],
                            });
                        }
                        _ => {}
                    }

                    if full_name == cfg.display.input {
                        f.out_image = view;
                    }
                }
                f.subpasses.push(SubpassAttachments {
                    name: subpass.name.clone(),
                    colors,
                    depth,
                });
            }

            f.sems = unsafe { &mut *(ctx) }.make_semaphores(64).unwrap();
        });
    }

    fn make_bind_group_layouts(&mut self) {
        // Bindless BG contains all textures.
        let bindless = self
            .get_ctx()
            .make_bind_group_layout(&BindGroupLayoutInfo {
                debug_name: "[MISO] Bindless Bind Group Layout",
                shaders: &[ShaderInfo {
                    shader_type: ShaderType::All,
                    variables: &[
                        BindGroupVariable {
                            var_type: BindGroupVariableType::SampledImage,
                            binding: 10,
                        },
                        BindGroupVariable {
                            var_type: BindGroupVariableType::DynamicUniform,
                            binding: 11,
                        },
                    ],
                }],
            })
            .unwrap();

        self.global_res.bg_layouts = MisoBGLayouts {
            bindless,
            per_pipeline: Default::default(),
            per_frame: Default::default(),
            per_object: Default::default(),
        };
    }

    fn make_display(&mut self, cfg: &json::Config) {
        self.global_res.display = Some(
            self.get_ctx()
                .make_display(&DisplayInfo {
                    window: WindowInfo {
                        title: cfg.display.name.clone(),
                        size: cfg.display.size,
                        resizable: false,
                    },
                    vsync: false,
                    buffering: dashi::WindowBuffering::Triple,
                })
                .unwrap(),
        );
    }

    fn make_pipelines(&mut self, cfg: &json::Config) {
        //        let stdvert = include_spirv!("a.spv");
        let stdvert = include_spirv!("target/spirv/stdvert.spv");
        let stdfrag = include_spirv!("target/spirv/stdfrag.spv");

        for pipe in &cfg.passes {
            let (vshader, pshader) = match pipe.graphics.as_str() {
                "standard" => (stdvert.as_slice(), stdfrag.as_slice()),
                _ => todo!(),
            };

            let mut reflection = reflection::ShaderInspector::new(&[vshader, pshader]).unwrap();

            let bindless_textures_details = reflection.get_binding_details("bless_textures");
            let dynamic_details = reflection.get_binding_details("per_obj");
            let mut bg_layouts = [None, None, None, None];

            if bindless_textures_details.is_some() || dynamic_details.is_some() {
                bg_layouts[0] = Some(self.global_res.bg_layouts.bindless);
            }

            let rp = self.global_res.render_pass.handle.clone();
            let subpass_info = cfg.render_pass.subpasses[pipe.subpass as usize].clone();
            let has_depth = subpass_info
                .attachments
                .iter()
                .find(|a| a.kind.to_lowercase() == "depth")
                .is_some();

            let num_attachments = if has_depth {
                (subpass_info.attachments.len() - 1) as u32
            } else {
                subpass_info.attachments.len() as u32
            };

            let color_blends = pipe
                .blends
                .as_ref()
                .unwrap_or(&vec![Default::default(); num_attachments as usize])
                .clone();

            let depth_info = if has_depth {
                if let Some(i) = pipe.depth_info {
                    Some(i.clone())
                } else {
                    Some(Default::default())
                }
            } else {
                None
            };

            let layout = self
                .get_ctx()
                .make_graphics_pipeline_layout(&GraphicsPipelineLayoutInfo {
                    debug_name: &pipe.name,
                    vertex_info: VertexDescriptionInfo {
                        entries: &[
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 0,
                                offset: std::mem::offset_of!(Vertex, position),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 1,
                                offset: std::mem::offset_of!(Vertex, normal),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec2,
                                location: 2,
                                offset: std::mem::offset_of!(Vertex, tex_coords),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 3,
                                offset: std::mem::offset_of!(Vertex, joint_ids),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 4,
                                offset: std::mem::offset_of!(Vertex, joints),
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
                        depth_test: depth_info,
                        color_blend_states: color_blends.clone(),
                    },
                })
                .unwrap();

            let pipeline = self
                .get_ctx()
                .make_graphics_pipeline(&GraphicsPipelineInfo {
                    debug_name: &pipe.name,
                    layout,
                    render_pass: rp,
                    subpass_id: pipe.subpass as u8,
                })
                .unwrap();

            self.global_res.render_pass.subpasses.push(MisoSubpass {
                name: pipe.name.clone(),
                reflection,
                bg_layouts,
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
        return self.global_res.textures.push(Texture {
            handle: info.image,
            sampler: info.sampler,
            dim: info.dim,
            view: info.view,
        });
    }

    pub fn unregister_texture(&mut self, h: Handle<Texture>) {
        self.global_res.textures.release(h);
    }

    pub fn register_camera(&mut self, info: &CameraInfo) -> Handle<Camera> {
        let h = self.global_res.cameras.push(Camera {
            transform: info.transform,
            projection: info.projection,
        });

        for pass in &mut self.global_res.render_pass.subpasses {
            if &pass.name == info.pass {
                pass.camera = h;
            }
        }

        h
    }

    pub fn update_camera_transform(&mut self, h: Handle<Camera>, transform: &Mat4) {
        self.global_res.cameras.get_ref_mut(h).transform = *transform;
    }

    pub fn unregister_camera(&mut self, h: Handle<Camera>) {
        self.global_res.cameras.release(h);
    }

    pub fn register_mesh(&mut self, info: &MeshInfo) -> Handle<Mesh> {
        self.dirty = true;

        self.global_res
            .meshes
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
        self.global_res.meshes.release(handle);
    }

    pub fn register_material(&mut self, info: &MaterialInfo) -> Handle<Material> {
        self.dirty = true;
        self.global_res
            .materials
            .insert(Material {
                data: MaterialShaderData {
                    base_color: info.base_color,
                    normal: info.normal,
                },
                passes: info.passes.clone(),
            })
            .unwrap()
    }

    pub fn unregister_material(&mut self, handle: Handle<Material>) {
        self.dirty = true;
        self.global_res.materials.release(handle);
    }

    pub fn register_object(&mut self, info: &ObjectInfo) -> Handle<Renderable> {
        let h = self
            .global_res
            .renderables
            .insert(Renderable {
                mesh: info.mesh,
                material: info.material,
                transform: info.transform,
            })
            .unwrap();

        let mat = self.global_res.materials.get_ref(info.material).unwrap();
        for subpass in &mut self.global_res.render_pass.subpasses {
            for name in &mat.passes {
                if subpass.name == name.as_str() {
                    let po = subpass.objects.insert(PassObject { original: h }).unwrap();
                    subpass.non_batched.push(MisoBatch {
                        handle: po,
                        sort_key: 0,
                    });
                }
            }
        }

        h
    }

    pub fn update_object_transform(&mut self, handle: Handle<Renderable>, transform: &Mat4) {
        self.global_res
            .renderables
            .get_mut_ref(handle)
            .unwrap()
            .transform = *transform;
    }

    pub fn unregister_object(&mut self, handle: Handle<Renderable>) {
        self.global_res.renderables.release(handle);
    }

    fn get_ctx(&mut self) -> &mut Context {
        unsafe { &mut *(self.global_res.ctx) }
    }

    fn reconfigure_scene(&mut self) {
        const BINDLESS_SET: u32 = 10;
        let mut bindings = Vec::new();
        self.global_res.textures.for_each_handle(|h| {
            let t = self.global_res.textures.get_ref(h);
            bindings.push(IndexedResource {
                resource: ShaderResource::SampledImage(t.view, t.sampler),
                slot: h.slot as u32,
            });
        });

        let bindless = self.global_res.bg_layouts.bindless;
        let resource = ShaderResource::Dynamic(&self.global_res.dynamic);
        let ctx = self.global_res.ctx;
        let bg = unsafe {
            (*(ctx))
                .make_indexed_bind_group(&IndexedBindGroupInfo {
                    debug_name: "[MISO] Bindless Bind Group",
                    layout: bindless,
                    bindings: &[
                        IndexedBindingInfo {
                            resources: &bindings,
                            binding: 10,
                        },
                        IndexedBindingInfo {
                            resources: &[IndexedResource { resource, slot: 0 }],
                            binding: 11,
                        },
                    ],
                    set: BINDLESS_SET,
                })
                .unwrap()
        };

        self.global_res.bindless = bg;
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.reconfigure_scene();
            self.dirty = false;
        }

        const WIDTH: u32 = 1280;
        const HEIGHT: u32 = 1024;

        self.frame.curr_mut().delete_queue.tex.delete_all();
        let ctx = self.global_res.ctx;
        let (img, sem, _idx, _good) = unsafe { &mut *(ctx) }
            .acquire_new_image(&mut self.global_res.display.as_mut().unwrap())
            .unwrap();

        self.global_res.dynamic.reset();
        let curr_frame = self.frame.curr();
        self.draw_cmd.record(|list| {
            for (i, pass) in self.global_res.render_pass.subpasses.iter().enumerate() {
                list.begin_drawing(&DrawBegin {
                    viewport: Viewport {
                        area: FRect2D {
                            w: WIDTH as f32,
                            h: HEIGHT as f32,
                            ..Default::default()
                        },
                        scissor: Rect2D {
                            w: WIDTH,
                            h: HEIGHT,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    pipeline: pass.pipeline.gfx,
                    subpass: Subpass {
                        colors: &curr_frame.subpasses[0].colors,
                        depth: curr_frame.subpasses[0].depth.clone(),
                    },
                })
                .expect("Error beginning render pass!");

                let viewproj = if pass.camera.valid() {
                    let l = self.global_res.cameras.get_ref(pass.camera);
                    l.transform * l.projection
                } else {
                    Mat4::default()
                };

                for batch in &pass.non_batched {
                    let p = pass.objects.get_ref(batch.handle).unwrap();
                    let renderable = self.global_res.renderables.get_ref(p.original).unwrap();
                    let mesh = self.global_res.meshes.get_ref(renderable.mesh).unwrap();
                    let material = self
                        .global_res
                        .materials
                        .get_ref(renderable.material)
                        .unwrap();

                    #[repr(C)]
                    struct PerFrameInfo {
                        transform: Mat4,
                        material: MaterialShaderData,
                        fid: u32,
                    }

                    let mut alloc = self.global_res.dynamic.bump().unwrap();
                    let info = &mut alloc.slice::<PerFrameInfo>()[0];
                    info.material = material.data;
                    info.transform = viewproj * renderable.transform;
                    list.draw_indexed(&DrawIndexed {
                        vertices: mesh.vertices,
                        indices: mesh.indices,
                        dynamic_buffers: [Some(alloc), None, None, None],
                        bind_groups: [Some(self.global_res.bindless.clone()), None, None, None],
                        index_count: mesh.num_indices,
                        instance_count: 1,
                        first_instance: 0,
                    });
                }

                list.end_drawing().expect("Error ending render pass!");
            }

            // Blit the framebuffer to the display's image
            list.blit(ImageBlit {
                src: self.frame.curr().out_image,
                dst: img,
                filter: Filter::Nearest,
                ..Default::default()
            });
        });

        // Submit our recorded commands
        self.draw_cmd.submit(&SubmitInfo {
            wait_sems: &[sem],
            signal_sems: &[self.frame.curr().sems[0], self.frame.curr().sems[1]],
            ..Default::default()
        });

        unsafe { &mut *(ctx) }
            .present_display(
                self.global_res.display.as_mut().unwrap(),
                &[self.frame.curr().sems[0], self.frame.curr().sems[1]],
            )
            .unwrap();

        self.frame.advance_next_frame();
    }
}
