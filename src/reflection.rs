extern crate spirv_cross;
use dashi::utils::Handle;
use dashi::{BindGroupLayout, Buffer, Image};
use spirv_cross::{hlsl, spirv};
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HandleType {
    Buffer(Handle<Buffer>),
    Image(Handle<Image>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BindingDetails {
    pub binding: u32,
    pub set: u32,
    pub descriptor_type: String,
    pub name: String,
}

type BindGroupLayoutCallback = Arc<dyn Fn(&BindingDetails) -> bool + Send + Sync>;

#[allow(dead_code)]
#[derive(Default)]
pub struct ShaderInspector {
    compiler_modules: Vec<spirv::Ast<spirv_cross::hlsl::Target>>,
    bg_layout_callbacks: Vec<(Handle<BindGroupLayout>, BindGroupLayoutCallback)>,
    handle_bindings: HashMap<HandleType, BindingDetails>,
}

#[allow(dead_code)]
impl ShaderInspector {
    pub fn new() -> Self {
        Self {
            compiler_modules: Default::default(),
            handle_bindings: Default::default(),
            bg_layout_callbacks: Default::default(),
        }
    }
    /// Creates a new `ShaderInspector` from multiple SPIR-V binary slices.
    pub fn parse(
        &mut self,
        spirv_data_slices: &[&[u32]],
    ) -> Result<[Option<Handle<BindGroupLayout>>; 4], &'static str> {
        self.compiler_modules.clear();
        let mut compiler_modules = Vec::new();
        let mut layouts = [None, None, None, None];
        for spirv_data in spirv_data_slices {
            let module = spirv::Module::from_words(spirv_data);
            let compiler = spirv::Ast::<hlsl::Target>::parse(&module)
                .map_err(|_| "Failed to create SPIR-V compiler")?;
            compiler_modules.push(compiler);
        }

        self.compiler_modules = compiler_modules;

        let mut idx = 0;
        let mut cbs = self.bg_layout_callbacks.clone();
        self.iter_binding_details(|details| {
            for (layout, cb) in &mut cbs {
                if cb(&details) && idx < layouts.len() {
                    layouts[idx] = Some(*layout);
                    idx += 1;
                }
            }
        });
        Ok(layouts)
    }

    pub fn get_bg_layout_binding_details(
        &self,
        handle: Handle<BindGroupLayout>,
    ) -> Option<Vec<BindingDetails>> {
        self.bg_layout_callbacks.iter().find_map(|(h, _)| {
            if *h == handle {
                Some(self.handle_bindings.values().cloned().collect())
            } else {
                None
            }
        })
    }

    pub fn add_bg_layout(&mut self, handle: Handle<BindGroupLayout>, cb: BindGroupLayoutCallback) {
        self.bg_layout_callbacks.push((handle, cb));
    }

    pub fn get_handle_mappings(&self) -> &HashMap<HandleType, BindingDetails> {
        &self.handle_bindings
    }

    fn iter_binding_details<F>(&mut self, mut func: F)
    where
        F: FnMut(BindingDetails),
    {
        for compiler in &mut self.compiler_modules {
            if let Ok(resources) = compiler.get_shader_resources() {
                for resource in resources
                    .uniform_buffers
                    .iter()
                    .chain(&resources.storage_buffers)
                    .chain(&resources.sampled_images)
                    .chain(&resources.storage_images)
                {
                    let name = compiler.get_name(resource.id).unwrap_or_default();
                    let binding_info = compiler
                        .get_decoration(resource.id, spirv::Decoration::Binding)
                        .ok()
                        .unwrap_or_default();
                    let set_info = compiler
                        .get_decoration(resource.id, spirv::Decoration::DescriptorSet)
                        .ok()
                        .unwrap_or_default();
                    let descriptor_type = format!("{:?}", resource.type_id);

                    func(BindingDetails {
                        binding: binding_info,
                        set: set_info,
                        descriptor_type,
                        name,
                    });
                }
            }
        }
    }

    /// Combines all bindings across multiple SPIR-V modules and checks for a specific binding.
    pub fn get_binding_details(&mut self, binding_name: &str) -> Option<BindingDetails> {
        for compiler in &mut self.compiler_modules {
            if let Ok(resources) = compiler.get_shader_resources() {
                for resource in resources
                    .uniform_buffers
                    .iter()
                    .chain(&resources.storage_buffers)
                    .chain(&resources.sampled_images)
                    .chain(&resources.storage_images)
                {
                    let name = compiler.get_name(resource.id).unwrap_or_default();
                    if name == binding_name {
                        let binding_info = compiler
                            .get_decoration(resource.id, spirv::Decoration::Binding)
                            .ok()?;
                        let set_info = compiler
                            .get_decoration(resource.id, spirv::Decoration::DescriptorSet)
                            .ok()?;
                        let descriptor_type = format!("{:?}", resource.type_id);

                        return Some(BindingDetails {
                            binding: binding_info,
                            set: set_info,
                            descriptor_type,
                            name,
                        });
                    }
                }
            }
        }
        None
    }
}
