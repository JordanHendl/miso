use dashi::{
    utils::{per_frame::PerFrame, Handle},
    BindGroup, BufferUsage, Context, Display, DisplayInfo, DrawBegin, DrawIndexed,
    DynamicAllocator, DynamicAllocatorInfo, Filter, FramedCommandList, ImageBlit, ImageView,
    IndexedBindGroupInfo, IndexedBindingInfo, IndexedResource, Semaphore, ShaderResource,
    SubmitInfo, WindowInfo,
};
pub use sdl2::{event::Event, keyboard::Keycode};
use std::fs;

mod json;
mod pipeline;
mod reflection;
mod renderpass;
mod resource_manager;
mod subpass;
pub mod types;
mod util;
use glam::*;
use renderpass::*;
use resource_manager::*;
use subpass::*;
pub use types::*;
use util::*;

pub mod lights;
pub use lights::*;

const DEFAULT_CONFIG: &str = r##"{
  "render_pass": {
    "subpasses": [{
        "name": "main-pass",
        "passes": [{"Draw": {
            "name": "non-transparent",
            "camera": "main-camera",
            "pipeline": {"Standard": {"type": "standard"}},
            "render_masks": ["standard"],
            "depth_info": {
              "should_write": true,
              "should_test": true
            }
          }}
        ],
        "attachments": [
          {
            "name": "color",
            "type": "Color",
            "size": [1280, 720]
          },
          {
            "name": "depth",
            "type": "Depth",
            "size": [1280, 720]
          }
        ]
      }
    ]
  },
  
  "cameras": [ {
      "name": "main-camera",
      "transform": [1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0]
    }
  ],

  "display": {
    "name": "Miso",
    "size": [1280, 720],
    "input": "main-pass.color"
  }
}"##;

pub struct SceneInfo {
    pub cfg: Option<String>,
}

#[derive(Default, Clone)]
struct Deletion {
    tex: DeletionQueue<Handle<Texture>>,
}

#[derive(Default, Clone)]
struct PerFrameResources {
    out_image: Handle<ImageView>,
    delete_queue: Deletion,
    sems: Vec<Handle<Semaphore>>,
}

struct GlobalResources {
    ctx: *mut Context,
    res: ResourceManager,
    dynamic: DynamicAllocator,
    render_pass: RenderPass,
    bindless: Handle<BindGroup>,
    display: Option<Display>,
}

impl Default for GlobalResources {
    fn default() -> Self {
        Self {
            ctx: std::ptr::null_mut(),
            render_pass: Default::default(),
            display: Option::default(),
            bindless: Default::default(),
            dynamic: Default::default(),
            res: Default::default(),
        }
    }
}

pub struct Scene {
    global_res: GlobalResources,
    frame: PerFrame<PerFrameResources>,
    draw_cmd: FramedCommandList,
    dirty: bool,
}

impl Scene {
    pub fn new(ctx: &mut dashi::Context, info: &SceneInfo) -> Self {
        let cfg: json::Config = if info.cfg.is_some() {
            let json_data = fs::read_to_string(info.cfg.as_ref().unwrap().clone());
            match json_data {
                Ok(json) => serde_json::from_str(&json).unwrap(),
                Err(_) => serde_json::from_str(&DEFAULT_CONFIG).unwrap(),
            }
        } else {
            serde_json::from_str(&DEFAULT_CONFIG).unwrap()
        };

        let mut frame: PerFrame<PerFrameResources> = PerFrame::new(2);
        let rp = RenderPass::new(ctx, &cfg, &mut frame);

        let mut s = Self {
            dirty: false,
            global_res: GlobalResources {
                ctx,
                render_pass: rp,
                ..Default::default()
            },
            frame,
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

        //        s.make_per_frame_attachments(&cfg);
        s.make_display(&cfg);
        s.global_res.res.lights = LightCollection::new(ctx);
        s
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

    pub fn register_texture(&mut self, info: &TextureInfo) -> Handle<Texture> {
        return self.global_res.res.textures.push(Texture {
            handle: info.image,
            sampler: info.sampler,
            dim: info.dim,
            view: info.view,
        });
    }

    pub fn unregister_texture(&mut self, h: Handle<Texture>) {
        self.global_res.res.textures.release(h);
    }

    pub fn register_camera(&mut self, info: &CameraInfo) -> Handle<Camera> {
        let h = self.global_res.res.cameras.push(Camera {
            transform: info.transform,
            projection: info.projection,
        });

        for pass in &mut self.global_res.render_pass.subpasses {
            for p in &mut pass.draws {
                if &p.name == info.pass || info.pass == "ALL" {
                    p.camera = h;
                }
            }
        }

        h
    }

    pub fn update_camera_projection(&mut self, h: Handle<Camera>, transform: &Mat4) {
        self.global_res.res.cameras.get_ref_mut(h).projection = *transform;
    }

    pub fn update_camera_transform(&mut self, h: Handle<Camera>, transform: &Mat4) {
        self.global_res.res.cameras.get_ref_mut(h).transform = *transform;
    }

    pub fn unregister_camera(&mut self, h: Handle<Camera>) {
        self.global_res.res.cameras.release(h);
    }

    pub fn register_mesh(&mut self, info: &MeshInfo) -> Handle<Mesh> {
        self.dirty = true;

        self.global_res
            .res
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
        self.global_res.res.meshes.release(handle);
    }

    pub fn register_material(&mut self, info: &MaterialInfo) -> Handle<Material> {
        self.dirty = true;
        self.global_res
            .res
            .materials
            .insert(Material {
                data: info.into(),
                passes: info.passes.clone(),
            })
            .unwrap()
    }

    pub fn unregister_material(&mut self, handle: Handle<Material>) {
        self.dirty = true;
        self.global_res.res.materials.release(handle);
    }

    pub fn register_directional_light(
        &mut self,
        info: &DirectionalLightInfo,
    ) -> Handle<DirectionalLight> {
        self.dirty = true;
        return self
            .global_res
            .res
            .lights
            .lights
            .insert(Light {
                dir: GPUOption::new(info.into()),
            })
            .unwrap();
    }

    pub fn unregister_directional_light(&mut self, _handle: Handle<DirectionalLight>) {
        todo!()
    }

    pub fn register_object(&mut self, info: &ObjectInfo) -> Handle<Renderable> {
        let h = self
            .global_res
            .res
            .renderables
            .insert(Renderable {
                mesh: info.mesh,
                material: info.material,
                transform: info.transform,
            })
            .unwrap();

        let mat = self
            .global_res
            .res
            .materials
            .get_ref(info.material)
            .unwrap();
        for subpass in &mut self.global_res.render_pass.subpasses {
            for pass in &mut subpass.draws {
                for name in &mat.passes {
                    if pass.name == name.as_str() || name.as_str() == "ALL" {
                        let po = pass.objects.insert(PassObject { original: h }).unwrap();
                        pass.non_batched.push(MisoBatch {
                            handle: po,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        h
    }

    pub fn update_object_transform(&mut self, handle: Handle<Renderable>, transform: &Mat4) {
        self.global_res
            .res
            .renderables
            .get_mut_ref(handle)
            .unwrap()
            .transform = *transform;
    }

    pub fn unregister_object(&mut self, handle: Handle<Renderable>) {
        self.global_res.res.renderables.release(handle);
    }

    fn get_ctx(&mut self) -> &mut Context {
        unsafe { &mut *(self.global_res.ctx) }
    }

    fn clean_up(&mut self) {
        self.frame.curr_mut().delete_queue.tex.delete_all();
    }

    fn reconfigure_scene(&mut self) {
        const BINDLESS_SET: u32 = 10;

        let mut bindings = Vec::new();

        self.global_res.res.textures.for_each_handle(|h| {
            let t = self.global_res.res.textures.get_ref(h);
            bindings.push(IndexedResource {
                resource: ShaderResource::SampledImage(t.view, t.sampler),
                slot: h.slot as u32,
            });
        });

        let bindless = self.global_res.render_pass.bg_layouts.bindless;
        let resource = ShaderResource::DynamicStorage(&self.global_res.dynamic);
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
                            resources: &[IndexedResource {
                                resource: ShaderResource::StorageBuffer(
                                    self.global_res.res.lights.lights.get_gpu_handle(),
                                ),
                                slot: 0,
                            }],
                            binding: 11,
                        },
                        IndexedBindingInfo {
                            resources: &[IndexedResource { resource, slot: 0 }],
                            binding: 20,
                        },
                    ],
                    set: BINDLESS_SET,
                })
                .unwrap()
        };

        for subpass in &mut self.global_res.render_pass.subpasses {
            for pass in &mut subpass.draws {
                pass.bind_groups = [Some(bg), None, None, None];
            }
        }

        self.global_res.bindless = bg;
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.reconfigure_scene();
            self.dirty = false;
        }

        self.clean_up();
        let ctx = self.global_res.ctx;
        assert!(self.global_res.display.is_some());

        let (img, sem, _idx, _good) = unsafe { &mut *(ctx) }
            .acquire_new_image(&mut self.global_res.display.as_mut().unwrap())
            .unwrap();

        self.draw_cmd.record_enumerated(|list, frame_idx| {
            if frame_idx == 0 {
                self.global_res.dynamic.reset();
            }

            for subpass in &self.global_res.render_pass.subpasses {
                for pass in &subpass.draws {
                    list.begin_drawing(&DrawBegin {
                        viewport: subpass.viewport,
                        pipeline: pass.pipeline.gfx,
                        subpass: dashi::Subpass {
                            colors: &subpass.attachments[frame_idx as usize].colors,
                            depth: subpass.attachments[frame_idx as usize].depth.clone(),
                        },
                    })
                    .expect("Error beginning render pass!");

                    let viewproj = if pass.camera.valid() {
                        let l = self.global_res.res.cameras.get_ref(pass.camera);
                        l.projection * l.transform
                    } else {
                        Mat4::default()
                    };

                    for batch in &pass.non_batched {
                        let p = pass.objects.get_ref(batch.handle).unwrap();
                        let renderable =
                            self.global_res.res.renderables.get_ref(p.original).unwrap();
                        let mesh = self.global_res.res.meshes.get_ref(renderable.mesh).unwrap();
                        let material = self
                            .global_res
                            .res
                            .materials
                            .get_ref(renderable.material)
                            .unwrap();

                        #[repr(C)]
                        struct PerFrameInfo {
                            transform: Mat4,
                            material: MaterialShaderData,
                        }

                        let mut alloc = self.global_res.dynamic.bump().unwrap();
                        let info = &mut alloc.slice::<PerFrameInfo>()[0];
                        info.material = material.data;
                        info.transform = (viewproj * renderable.transform).transpose();
                        list.draw_indexed(&DrawIndexed {
                            vertices: mesh.vertices,
                            indices: mesh.indices,
                            dynamic_buffers: [Some(alloc), None, None, None],
                            bind_groups: pass.bind_groups,
                            index_count: mesh.num_indices,
                            instance_count: 1,
                            first_instance: 0,
                        });
                    }

                    list.end_drawing().expect("Error ending render pass!");
                }
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
