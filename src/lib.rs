#[macro_use]
mod unity_audio_dsp;
mod plugin_ring_modulator;

use std::{
    cmp::min,
    ffi::CStr,
};

use unity_audio_dsp::{
    UnityAudioEffectDefinition, UnityAudioEffect_CreateCallback,
    UnityAudioEffect_GetFloatBufferCallback, UnityAudioEffect_GetFloatParameterCallback,
    UnityAudioEffect_ProcessCallback, UnityAudioEffect_ReleaseCallback,
    UnityAudioEffect_SetFloatParameterCallback, UnityAudioParameterDefinition,
    UNITY_AUDIO_PLUGIN_API_VERSION,
};

macro_rules! cstr {
    ($str:expr) => {
        unsafe { CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes()) }
    };
}

// Export symbol
unity_dsp_callback!(
    // This is the entry point of the plugin.
    export fn UnityGetAudioEffectDefinitions(desc_ptr: *mut *mut *mut UnityAudioEffectDefinition) -> i32 {
        let ring_mod = declare_effect(
            "Rusty Ring Modulator",
            plugin_ring_modulator::create_callback,
            plugin_ring_modulator::release_callback,
            plugin_ring_modulator::process_callback,
            plugin_ring_modulator::set_float_parameter_callback,
            plugin_ring_modulator::get_float_parameter_callback,
            plugin_ring_modulator::get_float_buffer_callback,
            &[
                declare_parameter("Frequency", "Hz", cstr!("The frequency of the sine wave"), 0.0, 22050.0, 1000.0, 1.0, 3.0),
                declare_parameter("Mix Amount", "%", cstr!("The amount of mix!"), 0.0, 1.0, 0.5, 1.0, 1.0),
            ],
        );

        // Indirection magic.
        // TODO: Leaking pointers...
        let array_of_effects = [
            Box::leak(Box::new(ring_mod)) as *mut UnityAudioEffectDefinition,
        ];

        let array_ptr = Box::leak(Box::new(array_of_effects)) as *mut *mut UnityAudioEffectDefinition;

        unsafe { *desc_ptr = array_ptr; }


        // TODO: Plugin is registered, but parameters are not detected??

        array_of_effects.len() as i32
    }
);

fn declare_effect(
    name: &str,
    create_callback: UnityAudioEffect_CreateCallback,
    release_callback: UnityAudioEffect_ReleaseCallback,
    process_callback: UnityAudioEffect_ProcessCallback,
    set_float_callback: UnityAudioEffect_SetFloatParameterCallback,
    get_float_callback: UnityAudioEffect_GetFloatParameterCallback,
    get_float_buffer_callback: UnityAudioEffect_GetFloatBufferCallback,
    param_defs: &[UnityAudioParameterDefinition],
    //declareParametersFn: impl FnOnce(&mut Vec<UnityAudioParameterDefinition>),
) -> UnityAudioEffectDefinition {
    let params_ptr: Box<[UnityAudioParameterDefinition]> = param_defs.into();

    UnityAudioEffectDefinition {
        structsize: std::mem::size_of::<UnityAudioEffectDefinition>() as u32,
        paramstructsize: std::mem::size_of::<UnityAudioParameterDefinition>() as u32,
        apiversion: UNITY_AUDIO_PLUGIN_API_VERSION,
        pluginversion: 0x010000,
        name: fit_cstr_array(name.as_bytes()),
        create: Some(create_callback),
        release: Some(release_callback),
        process: Some(process_callback),
        setfloatparameter: Some(set_float_callback),
        getfloatparameter: Some(get_float_callback),
        getfloatbuffer: Some(get_float_buffer_callback),
        numparameters: param_defs.len() as u32,
        channels: 2,
        flags: 0,
        reset: None,
        setposition: None,
        paramdefs: Box::leak(params_ptr).as_ptr(), // TODO: Leaking memory.
    }
}

fn declare_parameter(
    name: &str,
    unit: &str,
    description: &'static CStr,
    min: f32,
    max: f32,
    default_val: f32,
    display_scale: f32,
    display_exponent: f32,
) -> UnityAudioParameterDefinition {
    UnityAudioParameterDefinition {
        name: fit_cstr_array(name.as_bytes()),
        unit: fit_cstr_array(unit.as_bytes()),
        description: description.as_ptr() as *const u8,
        min,
        max,
        defaultval: default_val,
        displayscale: display_scale,
        displayexponent: display_exponent,
    }
}

fn fit_cstr_array<const SIZE: usize>(slice: &[u8]) -> [u8; SIZE] {
    let mut array = [0u8; SIZE];
    let limit = min(slice.len(), SIZE);
    array[..limit].copy_from_slice(&slice[..limit]);
    array[min(limit, SIZE - 1)] = 0; // insert null-terminator
    array
}
