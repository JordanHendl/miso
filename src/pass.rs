use std::collections::HashMap;

use dashi::{utils::Handle, ComputePipeline, GraphicsPipeline, Image};

pub struct Batch {

}

pub struct IndirectBatch {
    
}


pub struct RenderNode {
    pub compute: Option<(Handle<ComputePipeline>, [u32; 3])>,
    pub gfx: Option<Handle<GraphicsPipeline>>,
}

pub struct RenderPass {
    pub nodes: Vec<RenderNode>,
    pub flat_batches: Vec<Batch>,
    pub indirect_batches: Vec<IndirectBatch>,
}

pub struct RenderManager {
    pub rp: Handle<dashi::RenderPass>,
    pub attachments: HashMap<String, Handle<Image>>,
}

impl RenderManager {
    pub fn new(cfg: &str) -> Self {
        todo!()
    }
}
