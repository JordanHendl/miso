use dashi::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct AttachmentCFG {
    name: String,
    samples: SampleCount,
    load_op: LoadOp,
    store_op: StoreOp,
    stencil_load_op: LoadOp,
    stencil_store_op: StoreOp,
    clear_color: [f32; 4],
}

#[derive(Deserialize, Serialize, Clone)]
struct SubpassCFG {
    name: String,
    attachments: Vec<AttachmentCFG>,
}

#[derive(Deserialize, Serialize, Clone)]
struct RenderPassCFG {
    name: Option<String>,
    size: [u32; 2],
    subpasses: Vec<SubpassCFG>,
}

#[derive(Deserialize, Serialize, Clone)]
struct RenderMaskCFG {}

#[derive(Deserialize, Serialize, Clone)]
struct GraphNodeCFG {
    name: String,
    graphics: Option<String>,
    subpass: Option<String>,
    compute: Option<String>,
    render_masks: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RenderGraphCFG {
    render_pass: RenderPassCFG,
    nodes: Vec<GraphNodeCFG>,
    execution_order: Vec<String>,
}
