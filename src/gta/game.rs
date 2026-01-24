use retour::GenericDetour;

const SHUTDOWN: usize = 0x53C900;

pub fn shutdown() -> bool {
    shutdown_func()()
}

pub fn shutdown_func() -> extern "C" fn() -> bool {
    unsafe {
        std::mem::transmute(SHUTDOWN)
    }
}

pub fn on_shutdown<F: FnMut() + 'static>(callback: F) {
    let func = shutdown_func();

    unsafe {
        GenericDetour::new(func, cgame_destroy)
            .map(|hook| {
                let _ = hook.enable();

                HOOK = Some(CGameDestroyHook {
                    hook,
                    callback: Box::new(callback),
                });
            });
    }
}

struct CGameDestroyHook {
    hook: GenericDetour<extern "C" fn() -> bool>,
    callback: Box<dyn FnMut()>,
}

static mut HOOK: Option<CGameDestroyHook> = None;

extern "C" fn cgame_destroy() -> bool {
    unsafe {
        if let Some(hook) = HOOK.as_mut() {
            (hook.callback)();

            return hook.hook.call();
        }
    }

    true
}