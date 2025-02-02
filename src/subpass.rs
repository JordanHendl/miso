use crate::{Camera, Renderable};
use crate::pipeline::Pipeline;
use dashi::{utils::*, *};

#[derive(Default)]
pub struct PassObject {
    pub original: Handle<Renderable>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct MisoIndirectBatch {
    pub handle: Handle<Renderable>,
    pub first: u32,
    pub count: u32,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct MisoMultiIndirectBatch {
    pub first: u32,
    pub count: u32,
}

#[derive(Default)]
pub struct MisoBatch {
    pub handle: Handle<PassObject>,
    pub _sort_key: u64,
}

#[derive(Default)]
pub struct DrawPass {
    pub name: String,
    pub pipeline: Pipeline,
    pub camera: Handle<Camera>,
    pub bind_groups: [Option<Handle<BindGroup>>; 4],
    pub non_batched: Vec<MisoBatch>,
    pub objects: Pool<PassObject>,
    pub _objects_to_add: Vec<Handle<Renderable>>,
}

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct SubpassAttachments {
    pub name: String,
    pub colors: Vec<Attachment>,
    pub depth: Option<Attachment>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Subpass {
    pub name: String,
    pub id: u32,
    pub num_color_attachments: u32,
    pub has_depth: bool,
    pub viewport: Viewport,
    pub draws: Vec<DrawPass>,
    pub attachments: Vec<SubpassAttachments>, // Per-Frame
}


