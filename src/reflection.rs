extern crate spirv_cross;
use spirv_cross::{spirv, hlsl};

#[derive(Debug, Clone)]
pub struct BindingDetails {
    pub binding: u32,
    pub set: u32,
    pub descriptor_type: String,
    pub name: String,
}

#[derive(Default)]
pub struct ShaderInspector {
    compiler_modules: Vec<spirv::Ast<spirv_cross::hlsl::Target>>,
}

impl ShaderInspector {
    /// Creates a new `ShaderInspector` from multiple SPIR-V binary slices.
    pub fn new(spirv_data_slices: &[&[u32]]) -> Result<Self, &'static str> {
        let mut compiler_modules = Vec::new();

        for spirv_data in spirv_data_slices {
            let module = spirv::Module::from_words(spirv_data);
            let compiler = spirv::Ast::<hlsl::Target>::parse(&module).map_err(|_| "Failed to create SPIR-V compiler")?;
            compiler_modules.push(compiler);
        }

        Ok(Self { compiler_modules })
    }

    /// Combines all bindings across multiple SPIR-V modules and checks for a specific binding.
    pub fn get_binding_details(&mut self, binding_name: &str) -> Option<BindingDetails> {
        for compiler in &mut self.compiler_modules {
            if let Ok(resources) = compiler.get_shader_resources() {
                for resource in resources.uniform_buffers.iter().chain(&resources.storage_buffers).chain(&resources.sampled_images).chain(&resources.storage_images) {
                    let name = compiler.get_name(resource.id).unwrap_or_default();
                    if name == binding_name {
                        let binding_info = compiler.get_decoration(resource.id, spirv::Decoration::Binding).ok()?;
                        let set_info = compiler.get_decoration(resource.id, spirv::Decoration::DescriptorSet).ok()?;
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

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_inspector() {
        // Load SPIR-V binaries (replace with actual SPIR-V binary data).
        let spirv_data_1: Vec<u32> = vec![
            // SPIR-V binary 1 goes here...
        ];

        let spirv_data_2: Vec<u32> = vec![
            // SPIR-V binary 2 goes here...
        ];

        let inspector = ShaderInspector::new(&[&spirv_data_1, &spirv_data_2])
            .expect("Failed to create inspector");

        // Check for a specific binding
        if let Some(binding) = inspector.get_binding_details("myBinding") {
            println!("Found binding: {:?}", binding);
        } else {
            println!("Binding not found");
        }
    }
}

//use spirv_reflect::{types::*, ShaderModule};
//
//#[derive(Debug, Clone)]
//pub struct BindingDetails {
//    pub binding: u32,
//    pub set: u32,
//    pub descriptor_type: ReflectDescriptorType,
//    pub name: String,
//}
//
//pub struct ShaderInspector {
//    shader_modules: Vec<ShaderModule>,
//}
//
//impl ShaderInspector {
//    /// Creates a new `ShaderInspector` from multiple SPIR-V binary slices.
//    pub fn new(spirv_data_slices: &[&[u32]]) -> Result<Self, &'static str> {
//        let mut shader_modules = Vec::new();
//
//        for spirv_data in spirv_data_slices {
//            let shader_module = ShaderModule::load_u32_data(spirv_data)?;
//            let l = shader_module.enumerate_descriptor_bindings(None);
//            shader_modules.push(shader_module);
//        }
//
//        Ok(Self { shader_modules })
//    }
//
//    /// Combines all bindings across multiple SPIR-V modules and checks for a specific binding.
//    pub fn get_binding_details(&self, binding_name: &str) -> Option<BindingDetails> {
//        for shader_module in &self.shader_modules {
//            if let Ok(bindings) = shader_module.enumerate_descriptor_bindings(None) {
//                for binding in bindings {
//                    if binding.name == binding_name {
//                        return Some(BindingDetails {
//                            binding: binding.binding,
//                            set: binding.set,
//                            descriptor_type: binding.descriptor_type,
//                            name: binding.name.clone(),
//                        });
//                    }
//                }
//            }
//        }
//        None
//    }
//}
//
//// Example usage
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn test_shader_inspector() {
//        // Load SPIR-V binaries (replace with actual SPIR-V binary data).
//        let spirv_data_1: Vec<u32> = vec![
//            // SPIR-V binary 1 goes here...
//        ];
//
//        let spirv_data_2: Vec<u32> = vec![
//            // SPIR-V binary 2 goes here...
//        ];
//
//        let inspector = ShaderInspector::new(&[&spirv_data_1, &spirv_data_2])
//            .expect("Failed to create inspector");
//
//        // Check for a specific binding
//        if let Some(binding) = inspector.get_binding_details("myBinding") {
//            println!("Found binding: {:?}", binding);
//        } else {
//            println!("Binding not found");
//        }
//    }
//}

