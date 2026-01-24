use retour::GenericDetour;

use super::version::{version, Version};

struct LabelPoolDrawHook {
    hook: GenericDetour<extern "thiscall" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut DRAW_HOOK: Option<LabelPoolDrawHook> = None;

extern "thiscall" fn labelpool_draw(this: *mut ()) {
    unsafe {
        if let Some(hook) = DRAW_HOOK.as_mut() {
            hook.hook.call(this);
            (hook.callback)();
        }
    }
}

#[repr(C)]
pub struct LabelPool;

impl LabelPool {
    pub fn on_draw<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0x1340,
            Version::V037R3 => 0x1340,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(*mut ()) = std::mem::transmute(ptr);

            if let Ok(hook) = GenericDetour::new(func, labelpool_draw) {
                let _ = hook.enable();

                DRAW_HOOK = Some(LabelPoolDrawHook {
                    hook,
                    callback: Box::new(callback),
                });
            };
        }
    }
}
