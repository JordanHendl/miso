use rhai::{Engine, EvalAltResult};
use std::collections::HashMap;
use std::fs;
use dashi::*;
use dashi::utils::handle::Handle;
use common::*;
use crate::common;

enum Pipeline {
    Graphics(Handle<GraphicsPipeline>),
    Compute(Handle<ComputePipeline>),
}

pub struct ExecutionStrategy {
    pipeline: Pipeline,
    dispatch: Option<[u32; 3]>,
}

pub struct Algorithm {
    variables: HashMap<String,GraphResource>,
    
}

impl Algorithm {

}

