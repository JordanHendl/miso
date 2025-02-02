use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct RenderPass {
    pub subpasses: Vec<Subpass>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Pass {
    Draw(DrawPass),
    Dispatch(DispatchPass),
}

#[derive(Deserialize, Clone, Debug)]

#[allow(dead_code)]
pub struct Subpass {
    pub name: String,
    pub passes: Vec<Pass>,
    pub attachments: Vec<Attachment>,
}


#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Attachment {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub size: [u32; 2], // Size of the attachment
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Camera {
    pub name: String,
    pub transform: [f32; 16],
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct DispatchPass {
    pub name: String,
    pub compute: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StandardGraphicsPipeline {
    #[serde(rename = "type")]
    pub kind: String,
}


#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct CustomGraphicsPipeline {
    pub vertex: String,
    pub fragment: String,
}


#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub enum GraphicsPipeline {
    Custom(CustomGraphicsPipeline),
    Standard(StandardGraphicsPipeline)
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct DrawPass {
    pub name: String,
    pub camera: String, // References a camera by name
    pub pipeline: GraphicsPipeline,
    pub blends: Option<Vec<dashi::ColorBlendState>>,
    pub depth_info: Option<dashi::DepthInfo>,
    pub render_masks: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Display {
    pub name: String,
    pub size: [u32; 2],
    pub input: String, // References a render pass attachment
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub render_pass: RenderPass,
    pub cameras: Vec<Camera>,
    pub display: Display,
}
