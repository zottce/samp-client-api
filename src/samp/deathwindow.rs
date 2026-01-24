use retour::GenericDetour;

use super::version::{version, Version};

struct DeathWindowDrawHook {
    hook: GenericDetour<extern "thiscall" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut DRAW_HOOK: Option<DeathWindowDrawHook> = None;

extern "thiscall" fn deathwindow_draw(this: *mut ()) {
    unsafe {
        if let Some(hook) = DRAW_HOOK.as_mut() {
            hook.hook.call(this);
            (hook.callback)();
        }
    }
}

#[repr(C)]
pub struct DeathWindow;

impl DeathWindow {
    pub fn on_draw<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => crate::samp::v037::CDEATHWINDOW_DRAW,
            Version::V037R3 => crate::samp::v037r3::CDEATHWINDOW_DRAW,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(*mut ()) = std::mem::transmute(ptr);

            if let Ok(hook) = GenericDetour::new(func, deathwindow_draw) {
                let _ = hook.enable();

                DRAW_HOOK = Some(DeathWindowDrawHook {
                    hook,
                    callback: Box::new(callback),
                });
            };
        }
    }
}
