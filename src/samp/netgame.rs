use std::ptr::NonNull;
use std::net::SocketAddr;

use super::{v037, v037r3};
use super::version::{Version, version};
use retour::GenericDetour;
use crate::samp::Gamestate;

pub struct NetGame<'a> {
    netgame_v1: Option<&'a mut v037::CNetGame>,
    netgame_v3: Option<&'a mut v037r3::CNetGame>,
}

impl<'a> NetGame<'a> {
    pub fn get() -> NetGame<'a> {
        match version() {
            Version::V037 => NetGame {
                netgame_v1: v037::CNetGame::get(),
                netgame_v3: None,
            },

            Version::V037R3 => NetGame {
                netgame_v1: None,
                netgame_v3: v037r3::CNetGame::get(),
            },

            _ => panic!("Unknown SA:MP version"),
        }
    }

    pub fn addr(&self) -> Option<SocketAddr> {
        match version() {
            Version::V037 => self.netgame_v1.as_ref().and_then(|netgame| netgame.addr()),
            Version::V037R3 => self.netgame_v3.as_ref().and_then(|netgame| netgame.addr()),
            _ => None,
        }
    }

    pub fn on_destroy<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0x9380,
            Version::V037R3 => 0x9510,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(this: *mut ()) = std::mem::transmute(ptr);

            GenericDetour::new(func, cnetgame_destroy)
                .map(|hook| {
                    let _ = hook.enable();

                    DESTROY_HOOK = Some(CNetGameDestroyHook {
                        hook,
                        callback: Box::new(callback),
                    });
                });
        }
    }

    pub fn on_reconnect<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0xA060,
            Version::V037R3 => 0xA1E0,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(*mut ()) = std::mem::transmute(ptr);

            GenericDetour::new(func, cnetgame_reconnect)
                .map(|hook| {
                    let _ = hook.enable();

                    RECONNECT_HOOK = Some(CNetGameReconnectHook {
                        hook,
                        callback: Box::new(callback),
                    });
                });
        }
    }

    pub fn on_connected<F: FnMut() + 'static>(callback: F) {
        let address = match version() {
            Version::V037 => 0xA890,
            Version::V037R3 => 0xAA20,
            _ => return,
        };

        unsafe {
            let ptr = super::handle().add(address);
            let func: extern "thiscall" fn(*mut (), *mut ()) = std::mem::transmute(ptr);

            GenericDetour::new(func, cnetgame_connect)
                .map(|hook| {
                    let _ = hook.enable();

                    STATE_HOOK = Some(CNetGameStateHook {
                        hook,
                        callback: Box::new(callback),
                    });
                });
        }
    }
}

struct CNetGameDestroyHook {
    hook: GenericDetour<extern "thiscall" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut DESTROY_HOOK: Option<CNetGameDestroyHook> = None;

extern "thiscall" fn cnetgame_destroy(this: *mut ()) {
    unsafe {
        if let Some(hook) = DESTROY_HOOK.as_mut() {
            (hook.callback)();
            hook.hook.call(this);
        }
    }
}

struct CNetGameStateHook {
    hook: GenericDetour<extern "thiscall" fn(*mut (), *mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut STATE_HOOK: Option<CNetGameStateHook> = None;

extern "thiscall" fn cnetgame_connect(this: *mut (), packet: *mut ()) {
    unsafe {
        if let Some(hook) = STATE_HOOK.as_mut() {
            (hook.callback)();
            hook.hook.call(this, packet);
        }
    }
}

struct CNetGameReconnectHook {
    hook: GenericDetour<extern "thiscall" fn(*mut ())>,
    callback: Box<dyn FnMut()>,
}

static mut RECONNECT_HOOK: Option<CNetGameReconnectHook> = None;

extern "thiscall" fn cnetgame_reconnect(this: *mut ()) {
    unsafe {
        if let Some(hook) = RECONNECT_HOOK.as_mut() {
            (hook.callback)();
            hook.hook.call(this);
        }
    }
}