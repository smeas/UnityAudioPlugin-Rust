pub const UNITY_AUDIO_PLUGIN_API_VERSION: u32 = 0x010402;

#[macro_use]
#[cfg(target_os = "windows")]
mod windows {
    #[macro_export]
    macro_rules! unity_dsp_callback {
        (export $($t:tt)*) => { #[no_mangle] pub extern "stdcall" $($t)* };
        (pub $($t:tt)*) => { pub extern "stdcall" $($t)* };
        ($($t:tt)*) => { extern "stdcall" $($t)* };
    }
}

#[macro_use]
#[cfg(not(target_os = "windows"))]
mod unix {
    #[macro_export]
    macro_rules! unity_dsp_callback {
        (export $($t:tt)*) => { #[no_mangle] pub extern $($t)* };
        (pub $($t:tt)*) => { pub extern $($t)* };
        ($($t:tt)*) => { extern $($t)* };
    }
}


#[repr(i32)]
pub enum UnityAudioResult {
    Ok = 0,
    ErrUnsupported = 1,
}

#[repr(C)]
pub union UnityAudioEffectState {
    pub data: UnityAudioEffectState_Data,
    pub pad: [u8; 80],
}

impl UnityAudioEffectState {
    pub unsafe fn get_effect_data<'a, T>(&'a mut self) -> &'a mut T {
        let ptr = self.data.effectdata as *mut T;
        &mut *ptr
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnityAudioEffectState_Data {
    pub structsize: u32,                // Size of this struct
    pub samplerate: u32,                // System sample rate
    pub currdsptick: u64,               // Pointer to a sample counter marking the start of the current block being processed
    pub prevdsptick: u64,               // Used for determining when DSPs are bypassed and so sidechain info becomes invalid
    pub sidechainbuffer: *const f32,    // Side-chain buffers to read from
    pub effectdata: *mut (),            // Internal data for the effect
    pub flags: u32,                     // Various flags through which information can be queried from the host
    pub internal: *const (),            // Internal data, do not touch!

    // Version 1.0 of the plugin API only contains data up to here, so perform a state->structsize >= sizeof(UnityAudioEffectState) in your code before you
    // access any of this data in order to detect whether the host API is older than the plugin.

    pub spatializerdata: *const UnityAudioSpatializerData,    // Data for spatializers
    pub dspbuffersize: u32,                                   // Number of frames being processed per process callback. Use this to allocate temporary buffers before processing starts.
    pub hostapiversion: u32,                                  // Version of plugin API used by host

    pub ambisonicdata: *const UnityAudioAmbisonicData,        // Data for ambisonic plugins. Added in Unity 2017.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010400.
}

// This callback can be used to override the way distance attenuation is performed on AudioSources.
// distanceIn is the distance between the source and the listener and attenuationOut is the output volume.
// attenuationIn is the volume-curve based attenuation that would have been applied by Unity if this callback were not set.
// A typical attenuation curve may look like this: *attenuationOut = 1.0f / max(1.0f, distanceIn);
// The callback may also be used to apply a secondary gain on top of the one through attenuationIn by Unity's AudioSource curve.
//typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_DistanceAttenuationCallback)(UnityAudioEffectState* state, float distanceIn, float attenuationIn, float* attenuationOut);
pub type UnityAudioEffect_DistanceAttenuationCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, distanceIn: f32, attenuationIn: f32, attenuationOut: *mut f32) -> UnityAudioResult );

// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_CreateCallback)(UnityAudioEffectState* state);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_ReleaseCallback)(UnityAudioEffectState* state);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_ResetCallback)(UnityAudioEffectState* state);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_ProcessCallback)(UnityAudioEffectState* state, float* inbuffer, float* outbuffer, unsigned int length, int inchannels, int outchannels);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_SetPositionCallback)(UnityAudioEffectState* state, unsigned int pos);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_SetFloatParameterCallback)(UnityAudioEffectState* state, int index, float value);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_GetFloatParameterCallback)(UnityAudioEffectState* state, int index, float* value, char *valuestr);
// typedef UNITY_AUDIODSP_RESULT (UNITY_AUDIODSP_CALLBACK * UnityAudioEffect_GetFloatBufferCallback)(UnityAudioEffectState* state, const char* name, float* buffer, int numsamples);
pub type UnityAudioEffect_CreateCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState) -> UnityAudioResult );
pub type UnityAudioEffect_ReleaseCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState) -> UnityAudioResult );
pub type UnityAudioEffect_ResetCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState) -> UnityAudioResult );
pub type UnityAudioEffect_ProcessCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, inBuffer: *const f32, outBuffer: *mut f32, length  : u32, inChannels: i32, outChannels: i32) -> UnityAudioResult );
pub type UnityAudioEffect_SetPositionCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, pos: u32) -> UnityAudioResult );
pub type UnityAudioEffect_SetFloatParameterCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, index: i32, value: f32) -> UnityAudioResult );
pub type UnityAudioEffect_GetFloatParameterCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, index: i32, value: *mut f32, valueStr: *mut u8) -> UnityAudioResult );
pub type UnityAudioEffect_GetFloatBufferCallback = unity_dsp_callback!( fn(state: *mut UnityAudioEffectState, name: *const u8, buffer: *mut f32, numSamples: i32) -> UnityAudioResult );


#[repr(C)]
pub struct UnityAudioSpatializerData {
    pub listenermatrix: [f32; 16],                                                 // Matrix that transforms sourcepos into the local space of the listener
    pub sourcematrix: [f32; 16],                                                   // Transform matrix of audio source
    pub spatialblend: f32,                                                         // Distance-controlled spatial blend
    pub reverbzonemix: f32,                                                        // Reverb zone mix level parameter (and curve) on audio source
    pub spread: f32,                                                               // Spread parameter of the audio source (0..360 degrees)
    pub stereopan: f32,                                                            // Stereo panning parameter of the audio source (-1 = fully left, 1 = fully right)
    pub distanceattenuationcallback: UnityAudioEffect_DistanceAttenuationCallback, // The spatializer plugin may override the distance attenuation in order to influence the voice prioritization (leave this callback as NULL to use the built-in audio source attenuation curve)
    pub minDistance: f32,                                                          // Min distance of the audio source. This value may be helpful to determine when to apply near-field effects. Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401.
    pub maxDistance: f32,                                                          // Max distance of the audio source. Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401.
}

#[repr(C)]
pub struct UnityAudioAmbisonicData
{
    pub listenermatrix: [f32; 16],                                                 // Matrix that transforms sourcepos into the local space of the listener
    pub sourcematrix: [f32; 16],                                                   // Transform matrix of audio source
    pub spatialblend: f32,                                                         // Distance-controlled spatial blend
    pub reverbzonemix: f32,                                                        // Reverb zone mix level parameter (and curve) on audio source
    pub spread: f32,                                                               // Spread parameter of the audio source (0..360 degrees)
    pub stereopan: f32,                                                            // Stereo panning parameter of the audio source (-1 = fully left, 1 = fully right)
    pub distanceattenuationcallback: UnityAudioEffect_DistanceAttenuationCallback, // The ambisonic decoder plugin may override the distance attenuation in order to influence the voice prioritization (leave this callback as NULL to use the built-in audio source attenuation curve)
    pub ambisonicOutChannels: i32,                                                 // This tells ambisonic decoders how many output channels will actually be used.
    pub volume: f32,                                                               // Volume/mute of the audio source. If the the source is muted, volume is set to 0.0; otherwise, it is set to the audio source's volume. Volume is applied after the ambisonic decoder, so this is just informational. Added in Unity 2018.1, with UNITY_AUDIO_PLUGIN_API_VERSION 0x010401.
}

#[repr(C)]
pub struct UnityAudioEffectDefinition
{
    pub structsize: u32,                                                        // Size of this struct
    pub paramstructsize: u32,                                                   // Size of paramdesc fields
    pub apiversion: u32,                                                        // Plugin API version
    pub pluginversion: u32,                                                     // Version of this plugin
    pub channels: u32,                                                          // Number of channels. Effects should set this to 0 and process any number of input/output channels they get in the process callback. Generator elements should specify a >0 value here.
    pub numparameters: u32,                                                     // The number of parameters exposed by this plugin.
    pub flags: u64,                                                             // Various capabilities and requirements of the plugin.
    pub name: [u8; 32],                                                         // Name used for registration of the effect. This name will also be displayed in the GUI.
    // Option<T> is safe and transparent to use for ffi when T is a pointer type.
    // https://doc.rust-lang.org/nomicon/other-reprs.html
    pub create: Option<UnityAudioEffect_CreateCallback>,                        // The create callback is called when DSP unit is created and can be null.
    pub release: Option<UnityAudioEffect_ReleaseCallback>,                      // The release callback is called just before the plugin is freed and should free any data associated with this specific instance of the plugin. No further callbacks related to the instance will happen after this function has been called.
    pub reset: Option<UnityAudioEffect_ResetCallback>,                          // The reset callback is called by the user to bring back the plugin instance into its initial state. Use to avoid clicks or artifacts.
    pub process: Option<UnityAudioEffect_ProcessCallback>,                      // The processing callback is repeatedly called with a block of input audio to read from and an output block to write to.
    pub setposition: Option<UnityAudioEffect_SetPositionCallback>,              // The position callback can be used for implementing seek operations.
    pub paramdefs: *const UnityAudioParameterDefinition,                        // A pointer to the definitions of the parameters exposed by this plugin. This data pointed to must remain valid for the whole lifetime of the dynamic library (ideally it's static).
    pub setfloatparameter: Option<UnityAudioEffect_SetFloatParameterCallback>,  // This is called whenever one of the exposed parameters is changed.
    pub getfloatparameter: Option<UnityAudioEffect_GetFloatParameterCallback>,  // This is called to query parameter values.
    pub getfloatbuffer: Option<UnityAudioEffect_GetFloatBufferCallback>,        // Get N samples of named buffer. Used for displaying analysis data from the runtime.
}

#[repr(C)]
#[derive(Clone)]
pub struct UnityAudioParameterDefinition
{
    pub name: [u8; 16],         // Display name on the GUI
    pub unit: [u8; 16],         // Scientific unit of parameter to be appended after the value in textboxes
    pub description: *const u8, // Description of parameter (displayed in tool tips, automatically generated documentation, etc.)
    pub min: f32,               // Minimum value of the parameter
    pub max: f32,               // Maximum value of the parameter
    pub defaultval: f32,        // Default and initial value of the parameter
    pub displayscale: f32,      // Scale factor used only for the display of parameters (i.e. 100 for a percentage value ranging from 0 to 1)
    pub displayexponent: f32,   // Exponent for mapping parameters to sliders
}