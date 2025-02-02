use std::{collections::HashMap, sync::Arc};

use dashi::{
    utils::{per_frame::PerFrame, *},
    *,
};
type DRenderPass = dashi::RenderPass;
use crate::{json, pipeline, reflection, Camera, DrawPass, PerFrameResources, SubpassAttachments};

#[derive(Default)]
pub struct BindGroupLayouts {
    pub(crate) bindless: Handle<BindGroupLayout>,
}

fn make_bind_group_layouts(ctx: &mut dashi::Context) -> BindGroupLayouts {
    // Bindless BG contains all textures.
    let bindless = ctx
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

    BindGroupLayouts { bindless }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct RenderPass {
    pub handle: Handle<DRenderPass>,
    pub cameras: HashMap<String, Handle<Camera>>,
    pub camera_pool: Pool<Camera>,
    pub bg_layouts: BindGroupLayouts,
    pub shader_cache: reflection::ShaderInspector,
    pub subpasses: Vec<crate::Subpass>,
}

impl RenderPass {
    pub fn new(ctx: &mut dashi::Context, config: &json::Config, per_frame: &mut PerFrame<PerFrameResources>) -> Self {
        let mut rp = RenderPass::make_rp(ctx, config);
        rp.register_bg_layouts();
        rp.make_pipelines(ctx, config);
        rp.make_per_frame_attachments(ctx, config, per_frame);
        rp
    }

    fn register_bg_layouts(&mut self) {
        self.shader_cache.add_bg_layout(
            self.bg_layouts.bindless,
            Arc::new(|info| info.name == "bless_textures" || info.name == "per_obj"),
        );
    }

    // Initializes all draw & dispatch passes inside each subpass.
    fn make_pipelines(&mut self, ctx: &mut dashi::Context, cfg: &json::Config) {
        let ptr: *mut RenderPass = self;
        for subpass in &mut self.subpasses {
            let cfg_subpass = &cfg.render_pass.subpasses[subpass.id as usize];
            for pass in &cfg_subpass.passes {
                // This is safe. make_pipeline only mutates the shader cache, which is OK since it
                // is not being concurrently accessed.
                let p = pipeline::make_pipeline(
                    ctx,
                    unsafe { &mut (*ptr) },
                    subpass,
                    pass,
                );

                let name = match pass {
                    json::Pass::Draw(gfx) => gfx.name.clone(),
                    json::Pass::Dispatch(_) => todo!(),
                };
                subpass.draws.push(DrawPass {
                    name,
                    pipeline: p,
                    camera: Default::default(),
                    bind_groups: Default::default(),
                    ..Default::default()
                });
            }
        }
    }
    fn make_per_frame_attachments(
        &mut self,
        ctx: &mut dashi::Context,
        cfg: &json::Config,
        frame: &mut PerFrame<PerFrameResources>,
    ) {
        frame.for_each_mut(|f| {
            for (idx, subpass) in cfg.render_pass.subpasses.iter().enumerate() {
                let mut colors = Vec::new();
                let mut depth = None;

                for attach in &subpass.attachments {
                    let full_name = format!("{}.{}", subpass.name, attach.name);
                    let (_img, view) = RenderPass::create_view_from_attachment(ctx, attach);
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
                self.subpasses[idx].attachments.push(SubpassAttachments {
                    name: subpass.name.clone(),
                    colors,
                    depth,
                });
            }

            f.sems = ctx.make_semaphores(64).unwrap();
        });
    }

    // This creates JUST the render pass + collects info about subpasses. We cannot create the
    // pipelines yet.
    pub fn make_rp(ctx: &mut dashi::Context, config: &json::Config) -> RenderPass {
        let mut subpasses = Vec::new();
        let mut subpass_descriptions = Vec::with_capacity(512);
        let mut color_attachments = Vec::with_capacity(1024);
        let mut depth_attachments = Vec::with_capacity(1024);
        // Map subpasses from Config to the rendering library's Subpass type

        for config_subpass in &config.render_pass.subpasses {
            // Map color attachments
            for attachment in &config_subpass.attachments {
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
        for (i, config_subpass) in config.render_pass.subpasses.iter().enumerate() {
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

            subpass_descriptions.push(SubpassDescription {
                color_attachments: &color_attachments
                    [color_offset..config_subpass.attachments.len() - 1],
                depth_stencil_attachment: dep,
                subpass_dependencies: &[], // Add dependencies if needed [JHTODO]
            });

            color_offset += config_subpass.attachments.len();

            let first_attachment_size = config_subpass.attachments[0].size;
            let viewport = Viewport {
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
            };
            subpasses.push(crate::Subpass {
                name: config_subpass.name.clone(),
                id: i as u32,
                num_color_attachments: if dep.is_some() {
                    config_subpass.attachments.len() - 1
                } else {
                    config_subpass.attachments.len()
                } as u32,
                has_depth: dep.is_some(),
                viewport,
                draws: Default::default(),
                attachments: Default::default(),
            });
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
                subpasses: &subpass_descriptions,
                debug_name: "renderpass",
            })
            .unwrap();

        RenderPass {
            handle: render_pass,
            subpasses,
            bg_layouts: make_bind_group_layouts(ctx),
            cameras: Default::default(),
            camera_pool: Default::default(),
            shader_cache: Default::default(),
        }
    }

    // Helper function to create a view for an attachment
    pub fn create_view_from_attachment(
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
}
