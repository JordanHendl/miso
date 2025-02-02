use std::collections::HashMap;
use dashi::*;
use dashi::utils::handle::Handle;
use common::*;
use crate::common;

enum Pipeline {
    Graphics(Handle<GraphicsPipeline>),
    Compute((Handle<ComputePipeline>, [u32; 3])),
}

pub struct ExecutionStrategy {
    pipeline: Pipeline,
}

pub struct Algorithm {
    variables: HashMap<String,GraphResource>,
    
}

impl Algorithm {

}

