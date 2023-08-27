use std::f32::consts::PI;

use crate::unity_audio_dsp::{UnityAudioEffectState, UnityAudioResult};

const PARAM_COUNT: usize = 2;
const PARAM_FREQ: usize = 0;
const PARAM_MIX: usize = 1;

struct EffectData {
    param: [f32; PARAM_COUNT],
    sin: f32,
    cos: f32,
}

unity_dsp_callback!(
    pub fn create_callback(state: *mut UnityAudioEffectState) -> UnityAudioResult {
        let effect_data = Box::new(EffectData {
            param: [1000.0, 0.5],
            sin: 0.0,
            cos: 1.0,
        });

        // dropped in release_callback
        unsafe {
            (*state).data.effectdata = Box::leak(effect_data) as *mut EffectData as *mut ();
        }

        UnityAudioResult::Ok
    }
);

unity_dsp_callback!(
    pub fn release_callback(state: *mut UnityAudioEffectState) -> UnityAudioResult {
        unsafe {
            let effect_data: *mut EffectData = (*state).get_effect_data();
            drop(Box::from_raw(effect_data));
        }

        UnityAudioResult::Ok
    }
);

unity_dsp_callback!(
    pub fn set_float_parameter_callback(
        state: *mut UnityAudioEffectState,
        index: i32,
        value: f32,
    ) -> UnityAudioResult {
        let data: &mut EffectData = unsafe { (*state).get_effect_data() };

        if index < 0 || index > data.param.len() as i32 {
            return UnityAudioResult::ErrUnsupported;
        }

        data.param[index as usize] = value;

        UnityAudioResult::Ok
    }
);

unity_dsp_callback!(
    pub fn get_float_parameter_callback(
        state: *mut UnityAudioEffectState,
        index: i32,
        value: *mut f32,
        value_str: *mut u8,
    ) -> UnityAudioResult {
        let data: &mut EffectData = unsafe { (*state).get_effect_data() };

        if index < 0 || index > data.param.len() as i32 {
            return UnityAudioResult::ErrUnsupported;
        }

        if !value.is_null() {
            unsafe {
                *value = data.param[index as usize];
            }
        }

        if !value_str.is_null() {
            unsafe {
                *value_str = 0;
            }
        }

        UnityAudioResult::Ok
    }
);

unity_dsp_callback!(
    pub fn get_float_buffer_callback(
        state: *mut UnityAudioEffectState,
        name: *const u8,
        buffer: *mut f32,
        num_samples: i32,
    ) -> UnityAudioResult {
        UnityAudioResult::Ok
    }
);

unity_dsp_callback!(
    pub fn process_callback(
        state: *mut UnityAudioEffectState,
        in_buffer: *const f32,
        out_buffer: *mut f32,
        length: u32,
        in_channels: i32,
        out_channels: i32,
    ) -> UnityAudioResult {
        let data: &mut EffectData = unsafe { (*state).get_effect_data() };
        let samplerate = unsafe { (*state).data.samplerate };

        let w = 2.0 * (PI * data.param[PARAM_FREQ] / samplerate as f32).sin();

        for n in 0..length {
            for i in 0..(out_channels as u32) {
                let offset = (n * out_channels as u32 + i) as usize;
                let in_value = unsafe { in_buffer.add(offset).read() };
                let out_value =
                    in_value * (1.0 - data.param[PARAM_MIX] + data.param[PARAM_MIX] * data.sin);
                unsafe { out_buffer.add(offset).write(out_value) };
            }
            data.sin += data.cos * w; // cheap way to calculate a steady sine-wave
            data.cos -= data.sin * w;
        }

        UnityAudioResult::Ok
    }
);
