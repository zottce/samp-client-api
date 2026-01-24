use retour::GenericDetour;

use super::version::{version, Version};

struct SpawnScreenDrawHook {
    hook: GenericDetour<extern "thiscall" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut DRAW_HOOK: Option<SpawnScreenDrawHook> = None;

extern "thiscall" fn spawnscreen_draw(this: *mut ()) {
    unsafe {
        if let Some(hook) = DRAW_HOOK.as_mut() {
            hook.hook.call(this);
            (hook.callback)();
        }
    }
}

#[repr(C)]
pub struct SpawnScreen;

impl SpawnScreen {
    pub fn on_draw<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0x6C9B0,
            Version::V037R3 => 0x708A0,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(*mut ()) = std::mem::transmute(ptr);

            if let Ok(hook) = GenericDetour::new(func, spawnscreen_draw) {
                let _ = hook.enable();

                DRAW_HOOK = Some(SpawnScreenDrawHook {
                    hook,
                    callback: Box::new(callback),
                });
            };
        }
    }
}
