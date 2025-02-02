use dashi::{
    utils::Handle, CullMode, GraphicsPipeline, GraphicsPipelineDetails,
    GraphicsPipelineInfo, GraphicsPipelineLayout, GraphicsPipelineLayoutInfo, PipelineShaderInfo,
    ShaderType, VertexDescriptionInfo, VertexEntryInfo, VertexOrdering,
};
use inline_spirv::include_spirv;

use crate::{json, Subpass, Vertex};

#[allow(dead_code)]
#[derive(Default)]
pub struct Pipeline {
    pub gfx_layout: Handle<GraphicsPipelineLayout>,
    pub gfx: Handle<GraphicsPipeline>,
}

pub fn make_pipeline(
    ctx: &mut dashi::Context,
    rp: &mut crate::RenderPass,
    subpass: &Subpass,
    pass: &json::Pass,
) -> Pipeline {
    //        let stdvert = include_spirv!("a.spv");
    let stdvert = include_spirv!("target/spirv/stdvert.spv");
    let stdfrag = include_spirv!("target/spirv/stdfrag.spv");

    match pass {
        json::Pass::Draw(gfx) => {
            let (vshader, pshader) = match &gfx.pipeline {
                json::GraphicsPipeline::Custom(_) => todo!(),
                json::GraphicsPipeline::Standard(pipeline) => match pipeline.kind.as_str() {
                    "standard" => (stdvert.as_slice(), stdfrag.as_slice()),
                    _ => todo!(),
                },
            };

            let bg_layouts = rp.shader_cache.parse(&[vshader, pshader]).unwrap();
            let depth_info = if subpass.has_depth {
                if let Some(i) = gfx.depth_info {
                    Some(i.clone())
                } else {
                    Some(Default::default())
                }
            } else {
                None
            };

            let color_blends = gfx
                .blends
                .as_ref()
                .unwrap_or(&vec![
                    Default::default();
                    subpass.num_color_attachments as usize
                ])
                .clone();

            let pipeline_shader_info = PipelineShaderInfo {
                stage: ShaderType::Fragment,
                spirv: &pshader,
                specialization: &[],
            };

            let layout = ctx
                .make_graphics_pipeline_layout(&GraphicsPipelineLayoutInfo {
                    debug_name: &gfx.name,
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
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 2,
                                offset: std::mem::offset_of!(Vertex, color),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec2,
                                location: 3,
                                offset: std::mem::offset_of!(Vertex, tex_coords),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::IVec4,
                                location: 4,
                                offset: std::mem::offset_of!(Vertex, joint_ids),
                            },
                            VertexEntryInfo {
                                format: dashi::ShaderPrimitiveType::Vec4,
                                location: 5,
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
                        pipeline_shader_info,
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

            let pipeline = ctx
                .make_graphics_pipeline(&GraphicsPipelineInfo {
                    debug_name: &gfx.name,
                    layout,
                    render_pass: rp.handle,
                    subpass_id: subpass.id as u8,
                })
                .unwrap();

            Pipeline {
                gfx_layout: layout,
                gfx: pipeline,
                ..Default::default()
            }
        }
        json::Pass::Dispatch(_dispatch) => {
            todo!()
        }
    }

    //    for pipe in &cfg.passes {
    //        let mut reflection = reflection::ShaderInspector::new();
    //        reflection.parse(&[vshader, pshader]).unwrap();
    //
    //        let bindless_textures_details = reflection.get_binding_details("bless_textures");
    //        let dynamic_details = reflection.get_binding_details("per_obj");
    //        let mut bg_layouts = [None, None, None, None];
    //
    //        if bindless_textures_details.is_some() || dynamic_details.is_some() {
    //            bg_layouts[0] = Some(self.global_res.bg_layouts.bindless);
    //        }


}
