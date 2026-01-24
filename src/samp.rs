pub mod audio_stream;
pub mod deathwindow;
pub mod inputs;
pub mod label_pool;
pub mod netgame;
pub mod objects;
pub mod packets;
pub mod players;
pub mod spawnscreen;
pub mod v037;
pub mod v037r3;
pub mod version;

use version::{version, Version};

pub type TICK = std::os::raw::c_ulong;
pub type BOOL = std::os::raw::c_int;
pub type GTAREF = std::os::raw::c_int;
pub type ID = std::os::raw::c_ushort;
pub type NUMBER = std::os::raw::c_uchar;
pub type D3DCOLOR = std::os::raw::c_ulong;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CStdString {
    bytes: [u8; 16], // it's like union
    len: u32,
    capacity: u32,
}

impl CStdString {
    unsafe fn bytes_from_ptr<'a>(ptr: *const CStdString) -> &'a [u8] {
        let len = std::ptr::read_unaligned(std::ptr::addr_of!((*ptr).len)) as usize;
        let capacity = std::ptr::read_unaligned(std::ptr::addr_of!((*ptr).capacity));
        let bytes_ptr = std::ptr::addr_of!((*ptr).bytes) as *const u8;

        if capacity <= 0xF {
            std::slice::from_raw_parts(bytes_ptr, len)
        } else {
            let ptr_ptr = bytes_ptr as *const usize;
            let heap_ptr = std::ptr::read_unaligned(ptr_ptr) as *const u8;
            std::slice::from_raw_parts(heap_ptr, len)
        }
    }

    pub unsafe fn as_str_from_ptr<'a>(
        ptr: *const CStdString,
    ) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(Self::bytes_from_ptr(ptr))
    }

    pub unsafe fn as_str_unchecked_from_ptr<'a>(ptr: *const CStdString) -> &'a str {
        std::str::from_utf8_unchecked(Self::bytes_from_ptr(ptr))
    }

    fn bytes(&self) -> &[u8] {
        let len = self.len as usize;

        if self.capacity <= 0xF {
            &self.bytes[0..len]
        } else {
            let ptr = unsafe { (&self.bytes as *const _ as *const usize).read() };
            let ptr = ptr as *const u8;

            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.bytes())
    }

    pub fn as_str_unchecked(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.bytes()) }
    }

    pub fn to_string(&self) -> String {
        self.as_str_unchecked().to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Gamestate {
    None,
    WaitConnect,
    Connecting,
    AwaitJoin,
    Connected,
    Restarting,
}

impl From<v037r3::Gamestate> for Gamestate {
    fn from(state: v037r3::Gamestate) -> Gamestate {
        match state {
            v037r3::Gamestate::None => Gamestate::None,
            v037r3::Gamestate::WaitConnect => Gamestate::WaitConnect,
            v037r3::Gamestate::Connecting => Gamestate::Connecting,
            v037r3::Gamestate::Connected => Gamestate::Connected,
            v037r3::Gamestate::AwaitJoin => Gamestate::AwaitJoin,
            v037r3::Gamestate::Restarting => Gamestate::Restarting,
        }
    }
}

impl From<v037::Gamestate> for Gamestate {
    fn from(state: v037::Gamestate) -> Gamestate {
        match state {
            v037::Gamestate::None => Gamestate::None,
            v037::Gamestate::WaitConnect => Gamestate::WaitConnect,
            v037::Gamestate::Connecting => Gamestate::Connecting,
            v037::Gamestate::Connected => Gamestate::Connected,
            v037::Gamestate::AwaitJoin => Gamestate::AwaitJoin,
            v037::Gamestate::Restarting => Gamestate::Restarting,
        }
    }
}

pub fn gamestate() -> Gamestate {
    match version() {
        Version::V037 => v037::CNetGame::get()
            .map(|netgame| netgame.gamestate().into())
            .unwrap_or(Gamestate::None),

        Version::V037R3 => v037r3::CNetGame::get()
            .map(|netgame| netgame.gamestate().into())
            .unwrap_or(Gamestate::None),

        _ => Gamestate::None,
    }
}

pub fn is_loaded() -> bool {
    !handle().is_null()
}

pub fn handle() -> *mut u8 {
    crate::utils::module_handle("samp.dll") as *mut u8
}
