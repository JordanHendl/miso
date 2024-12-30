use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct RenderPass {
    pub subpasses: Vec<Subpass>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Subpass {
    pub name: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Attachment {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub size: [u32; 2], // Size of the attachment
}

#[derive(Deserialize, Clone, Debug)]
pub struct Camera {
    pub name: String,
    pub position: [f32; 3],
    pub front: [f32; 3],
    pub up: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct Pass {
    pub name: String,
    pub camera: String, // References a camera by name
    pub graphics: String,
    pub subpass: String,
    pub render_masks: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Display {
    pub name: String,
    pub size: [u32; 2],
    pub input: String, // References a render pass attachment
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub render_pass: RenderPass,
    pub cameras: Vec<Camera>,
    pub passes: Vec<Pass>,
    pub display: Display,
}
