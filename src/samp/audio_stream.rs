use retour::GenericDetour;

use super::version::{version, Version};

struct AudioStreamProcessHook {
    hook: GenericDetour<extern "C" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut PROCESS_HOOK: Option<AudioStreamProcessHook> = None;

extern "C" fn audio_stream_process(this: *mut ()) {
    unsafe {
        if let Some(hook) = PROCESS_HOOK.as_mut() {
            hook.hook.call(this);
            (hook.callback)();
        }
    }
}

#[repr(C)]
pub struct AudioStream;

impl AudioStream {
    pub fn on_process<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0x62B40,
            Version::V037R3 => 0x65F90,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "C" fn(*mut ()) = std::mem::transmute(ptr);

            if let Ok(hook) = GenericDetour::new(func, audio_stream_process) {
                let _ = hook.enable();

                PROCESS_HOOK = Some(AudioStreamProcessHook {
                    hook,
                    callback: Box::new(callback),
                });
            };
        }
    }
}
