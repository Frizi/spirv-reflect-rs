#[macro_use]
extern crate bitflags;

pub mod convert;
pub mod ffi;
pub mod types;

impl Default for ffi::SpvReflectShaderModule {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for ffi::SpvReflectDescriptorSet {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Default, Clone)]
pub struct ShaderModule {
    module: Option<ffi::SpvReflectShaderModule>,
}

impl ShaderModule {
    pub fn code_size(&self) -> usize {
        match self.module {
            Some(module) => unsafe { ffi::spvReflectGetCodeSize(&module) as usize },
            None => 0,
        }
    }

    pub fn code_slice(&self) -> &[u32] {
        let code_size = self.code_size();
        let module = self.module.unwrap();
        unsafe { std::slice::from_raw_parts(ffi::spvReflectGetCode(&module), code_size / 4) }
    }

    pub fn descriptor_set_count(&self) -> Result<u32, &str> {
        match self.module {
            Some(module) => {
                let mut count: u32 = 0;
                let result = unsafe {
                    ffi::spvReflectEnumerateDescriptorSets(
                        &module,
                        &mut count,
                        ::std::ptr::null_mut(),
                    )
                };
                match result {
                    ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => Ok(count),
                    _ => Err(convert::result_to_string(result)),
                }
            }
            None => Ok(0),
        }
    }

    pub fn descriptor_sets(&self) -> Result<Vec<types::ReflectDescriptorSet>, &str> {
        let count = self.descriptor_set_count()?;
        if let Some(ref module) = self.module {
            if count > 0 {
                let mut ffi_sets: Vec<*mut ffi::SpvReflectDescriptorSet> =
                    vec![::std::ptr::null_mut(); count as usize];
                let result = unsafe {
                    let mut out_count: u32 = count;
                    ffi::spvReflectEnumerateDescriptorSets(
                        module,
                        &mut out_count,
                        ffi_sets.as_mut_ptr(),
                    )
                };
                match result {
                    ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => {
                        let mut sets = Vec::new();
                        for ffi_set in ffi_sets {
                            let ffi_set_ref = unsafe { &*ffi_set };
                            let mut bindings: Vec<
                                types::ReflectDescriptorBinding,
                            > = Vec::with_capacity(ffi_set_ref.binding_count as usize);
                            let ffi_bindings = unsafe {
                                std::slice::from_raw_parts(
                                    ffi_set_ref.bindings,
                                    ffi_set_ref.binding_count as usize,
                                )
                            };
                            for ffi_binding in ffi_bindings {
                                let ffi_binding_ref = unsafe { &**ffi_binding };
                                let c_str: &std::ffi::CStr =
                                    unsafe { std::ffi::CStr::from_ptr(ffi_binding_ref.name) };
                                let str_slice: &str = c_str.to_str().unwrap();
                                bindings.push(types::ReflectDescriptorBinding {
                                    spirv_id: ffi_binding_ref.spirv_id,
                                    name: str_slice.to_owned(),
                                    binding: ffi_binding_ref.binding,
                                    input_attachment_index: ffi_binding_ref.input_attachment_index,
                                    set: ffi_binding_ref.set,
                                    descriptor_type: convert::ffi_to_descriptor_type(ffi_binding_ref.descriptor_type),

                            /*            pub descriptor_type: ReflectDescriptorType,
    pub resource_type: ReflectResourceType,
    pub image: ReflectImageTraits,
    pub block: ReflectBlockVariable,
    pub array: ReflectBindingArrayTraits,
    pub count: u32,
    pub uav_counter_id: u32,
    //pub uav_counter_binding: *mut SpvReflectDescriptorBinding,
    //pub type_description: *mut SpvReflectTypeDescription,
    pub word_offset: ReflectDescriptorBindingSet,
*/


                                    ..Default::default() // TODO
                                });
                            }
                            sets.push(types::ReflectDescriptorSet {
                                set: ffi_set_ref.set,
                                bindings,
                            });
                        }
                        Ok(sets)
                    }
                    _ => Err(convert::result_to_string(result)),
                }
            } else {
                // No descriptor sets
                Ok(Vec::new())
            }
        } else {
            // Invalid shader module
            Ok(Vec::new())
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        println!("Dropping!");
        if let Some(mut module) = self.module {
            unsafe {
                ffi::spvReflectDestroyShaderModule(&mut module);
            }
        }
    }
}

pub fn create_shader_module(spv_data: &[u8]) -> Result<ShaderModule, &str> {
    let mut module: ffi::SpvReflectShaderModule = unsafe { std::mem::zeroed() };
    let result: ffi::SpvReflectResult = unsafe {
        ffi::spvReflectCreateShaderModule(
            spv_data.len(),
            spv_data.as_ptr() as *const std::os::raw::c_void,
            &mut module,
        )
    };
    match result {
        ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => Ok(ShaderModule {
            module: Some(module),
        }),
        _ => Err(convert::result_to_string(result)),
    }
}

/*
let mut module: SpvReflectShaderModule = empty_shader_module();

    let mut result: SpvReflectResult = unsafe {
        spvReflectCreateShaderModule(
            spv_data.len(),
            spv_data.as_ptr() as *const ::std::os::raw::c_void,
            &mut module,
        )
    };
    */
