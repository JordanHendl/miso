use dashi::utils::handle::Handle;
use dashi::*;
use dashi::{Buffer, BufferCopy, BufferInfo, CommandList, Context};
use std::collections::HashMap;
use std::fs;

pub struct Hotbuffer {
    front: Handle<Buffer>,
    back: Handle<Buffer>,
    info: BufferInfo<'static>,
    mapped: *mut u8,
}

impl Hotbuffer {
    pub fn new(ctx: &mut Context, info: &BufferInfo) -> Self {
        let mut i = info.clone();
        i.visibility = dashi::MemoryVisibility::Gpu;

        let back = ctx.make_buffer(&i).unwrap();
        i.visibility = dashi::MemoryVisibility::CpuAndGpu;
        let front = ctx.make_buffer(&i).unwrap();

        i.debug_name = "";
        i.initial_data = None;
        Self {
            front,
            back,
            info: BufferInfo {
                debug_name: "",
                byte_size: i.byte_size,
                visibility: i.visibility,
                usage: i.usage,
                initial_data: None,
            },
            mapped: ctx.map_buffer_mut::<u8>(front).unwrap().as_mut_ptr(),
        }
    }

    pub fn info(&self) -> &'static BufferInfo {
        &self.info
    }

    pub fn record_sync_up(&mut self, cmd: &mut CommandList) {
        cmd.copy_buffers(&BufferCopy {
            src: self.front,
            dst: self.back,
            src_offset: 0,
            dst_offset: 0,
            size: self.info.byte_size as usize,
        });
    }

    pub fn record_sync_down(&mut self, cmd: &mut CommandList) {
        cmd.copy_buffers(&BufferCopy {
            src: self.back,
            dst: self.front,
            src_offset: 0,
            dst_offset: 0,
            size: self.info.byte_size as usize,
        });
    }

    pub fn staging(&self) -> Handle<Buffer> {
        self.front
    }

    pub fn data(&self) -> Handle<Buffer> {
        self.back
    }

    pub fn slice<T>(&self) -> &'static [T] {
        let slice =
            unsafe { std::slice::from_raw_parts(self.mapped, self.info.byte_size as usize) };
        return unsafe { slice.align_to::<T>().1 };
    }

    pub fn slice_mut<T>(&mut self) -> &'static mut [T] {
        let slice =
            unsafe { std::slice::from_raw_parts_mut(self.mapped, self.info.byte_size as usize) };
        return unsafe { slice.align_to_mut::<T>().1 };
    }
}

pub struct GraphImage {
    pub name: String,
    pub handle: Handle<Image>,
    pub view: Handle<ImageView>,
    pub info: ImageInfo<'static>,
}

pub enum GraphResource {
    Image(GraphImage),
    Buffer(Hotbuffer),
}

pub struct GraphResources {
    pub buffers: HashMap<String, Hotbuffer>,
    pub images: HashMap<String, GraphImage>,
}
