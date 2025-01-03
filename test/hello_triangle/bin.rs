use dashi::*;

pub struct ImageLoadInfo<T> {
    pub filename: String,
    pub size: [u32; 2],
    pub format: dashi::Format,
    pub bytes: Vec<T>,
}

impl<T> ImageLoadInfo<T> {
    pub fn into_gpu(&self, ctx: &mut Context) -> miso::TextureInfo {
        let img = ctx
            .make_image(&ImageInfo {
                debug_name: &self.filename,
                dim: [self.size[0], self.size[1], 1],
                layers: 1,
                format: self.format,
                mip_levels: 1,
                initial_data: unsafe { Some(self.bytes.align_to::<u8>().1) },
            })
            .unwrap();

        let view = ctx
            .make_image_view(&ImageViewInfo {
                debug_name: &self.filename,
                img,
                ..Default::default()
            })
            .unwrap();

        let sampler = ctx
            .make_sampler(&SamplerInfo {
                ..Default::default()
            })
            .unwrap();

        miso::TextureInfo {
            image: img,
            view,
            sampler,
            dim: self.size,
        }
    }
}

pub fn load_image_rgba8(path: &str) -> ImageLoadInfo<u8> {
    println!("Loading {}", path);
    let img = image::open(&path).unwrap();

    // Convert the image to RGBA8 format
    let rgba_image = img.to_rgba8();

    // Flip the image vertically (upside down)
    //    let rgba_image = image::imageops::flip_vertical(&rgba_image);

    let (width, height) = rgba_image.dimensions();
    let bytes = rgba_image.into_raw();
    assert!((width * height * 4) as usize == bytes.len());
    ImageLoadInfo::<u8> {
        filename: path.to_string(),
        size: [width, height],
        format: dashi::Format::RGBA8,
        bytes,
    }
}

#[cfg(feature = "miso-tests")]
fn main() {
    use glam::vec4;
    use miso::{MaterialInfo, MeshInfo, ObjectInfo, Vertex};

    let cfg = format!("{}/cfg/render_graph.json", env!("CARGO_MANIFEST_DIR"));
    let img = format!("{}/test/assets/default.png", env!("CARGO_MANIFEST_DIR"));

    let mut ctx = dashi::Context::new(&Default::default()).unwrap();
    let mut scene = miso::MisoScene::new(&mut ctx, &miso::MisoSceneInfo { cfg: cfg.clone() });

    let VERTICES: [Vertex; 3] = [
        Vertex { position: vec4(0.0, -0.5, 0.0, 1.0), ..Default::default() }, // Vertex 0: Bottom
        Vertex { position: vec4(0.5, 0.5, 0.0, 1.0), ..Default::default() }, // Vertex 0: Bottom
        Vertex { position: vec4(-0.5, 0.5, 0.0, 1.0), ..Default::default() }, // Vertex 0: Bottom
    ];


    const INDICES: [u32; 3] = [
        0, 1, 2, // Triangle: uses vertices 0, 1, and 2
    ];

    // Allocate the vertices & indices.
    let vertices = ctx
        .make_buffer(&BufferInfo {
            debug_name: "vertices",
            byte_size: (VERTICES.len() * std::mem::size_of::<Vertex>()) as u32,
            visibility: MemoryVisibility::Gpu,
            usage: BufferUsage::VERTEX,
            initial_data: unsafe { Some(VERTICES.align_to::<u8>().1) },
        })
        .unwrap();

    let indices = ctx
        .make_buffer(&BufferInfo {
            debug_name: "indices",
            byte_size: (INDICES.len() * std::mem::size_of::<u32>()) as u32,
            visibility: MemoryVisibility::Gpu,
            usage: BufferUsage::INDEX,
            initial_data: unsafe { Some(INDICES.align_to::<u8>().1) },
        })
        .unwrap();

    let mesh = scene.register_mesh(&MeshInfo {
        name: "hello-triangle triangle".to_string(),
        vertices,
        num_vertices: VERTICES.len(),
        indices,
        num_indices: INDICES.len(),
    });

    let base_color = scene.register_texture(&load_image_rgba8(&img).into_gpu(&mut ctx));

    let material = scene.register_material(&MaterialInfo {
        name: "hello-triangle".to_string(),
        passes: vec!["base-color".to_string()],
        base_color,
        ..Default::default()
    });

    let object = scene.register_object(&ObjectInfo {
        mesh,
        material,
        transform: Default::default(),
    });
    
    loop {
        scene.update();
    }
//
    ctx.clean_up();
}

#[cfg(not(feature = "miso-tests"))]
fn main() { //none
}
