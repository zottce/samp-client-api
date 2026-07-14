use super::{BOOL, CStdString, GTAREF, ID, TICK, handle};
use crate::gta::matrix::{CVector, RwMatrix};

use std::net::{Ipv4Addr, SocketAddr};

pub const CNETGAME: usize = 0x2ACA24;
pub const CINPUT: usize = 0x2ACA14;
pub const CGAME: usize = 0x2ACA3C;
pub const CGAME_SETCURSORMODE: usize = 0xA0530;
pub const CGAME_PROCESSINPUTENABLING: usize = 0xA0410;
pub const CDIALOG: usize = 0x2AC9E0;
pub const CDEATHWINDOW_DRAW: usize = 0x6A2E0;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Gamestate {
    None,
    WaitConnect,
    Connecting,
    AwaitJoin,
    Connected,
    Restarting,
}

impl From<i32> for Gamestate {
    fn from(state: i32) -> Self {
        match state {
            0x1 => Self::WaitConnect,
            0x2 => Self::Connecting,
            0x5 => Self::Connected,
            0x6 => Self::AwaitJoin,
            0xB => Self::Restarting,
            _ => Self::None,
        }
    }
}

#[repr(C, packed)]
pub struct CNetGame {
    pad_0: [u8; 44],
    rakclient: *mut (),
    ip: [u8; 257],
    hostname: [u8; 257],
    disable_collision: bool,
    update_camera_target: bool,
    nametag_status: bool,
    port: i32,
    lanmode: BOOL,
    map_icons: [GTAREF; 100],
    gamestate: i32,
    last_connect_attempt: TICK,
    settings: *mut (),
    pad_2: [u8; 5],
    pools: *mut CNetGamePools,
}

impl CNetGame {
    pub fn get<'a>() -> Option<&'a mut Self> {
        let ptr = netgame();
        (!ptr.is_null()).then(|| unsafe { &mut *ptr })
    }

    pub fn addr(&self) -> Option<SocketAddr> {
        let ip = unsafe { std::ptr::addr_of!(self.ip).read_unaligned() };
        let end = ip.iter().position(|&byte| byte == 0).unwrap_or(ip.len());
        let addr: Ipv4Addr = std::str::from_utf8(&ip[..end]).ok()?.parse().ok()?;
        let port = unsafe { std::ptr::addr_of!(self.port).read_unaligned() };
        Some(SocketAddr::from((addr, port as u16)))
    }

    pub fn gamestate(&self) -> Gamestate {
        Gamestate::from(unsafe { std::ptr::addr_of!(self.gamestate).read_unaligned() })
    }
}

#[repr(C, packed)]
pub struct CNetGamePools {
    menu: *mut (),
    actor: *mut (),
    player: *mut CPlayerPool,
    vehicle: *mut (),
    pickup: *mut (),
    object: *mut CObjectPool,
    gangzone: *mut (),
    label: *mut (),
    textdraw: *mut (),
}

#[repr(C, packed)]
pub struct CPlayerPool {
    local_player_id: ID,
    align: i32,
    local_player_name: CStdString,
    local_player: *mut CLocalPlayer,
    largest_id: i32,
    players: [*mut CPlayerInfo; 1004],
    not_empty: [BOOL; 1004],
    previous_collision: [BOOL; 1004],
    local_player_ping: i32,
    local_player_score: i32,
}

#[repr(C, packed)]
pub struct CPlayerInfo {
    score: i32,
    is_npc: BOOL,
    remote_player: *mut CRemotePlayer,
    ping: u32,
    align: u32,
    nickname: CStdString,
}

impl CPlayerInfo {
    pub fn remote_player(&self) -> Option<&CRemotePlayer> {
        let ptr = unsafe { std::ptr::addr_of!(self.remote_player).read_unaligned() };
        (!ptr.is_null()).then(|| unsafe { &*ptr })
    }

    pub fn gta_ped(&self) -> Option<&super::players::GamePed> {
        self.remote_player()?.game_ped()
    }

    pub fn is_in_stream(&self) -> bool {
        self.gta_ped().is_some()
    }

    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        self.name()
            .map(|name| {
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish()
            })
            .unwrap_or(0)
    }

    pub fn name(&self) -> Option<&str> {
        unsafe { CStdString::as_str_from_ptr(std::ptr::addr_of!(self.nickname)).ok() }
    }

    pub fn name_with_id(&self) -> String {
        self.name()
            .zip(self.remote_player())
            .map(|(name, remote)| format!("[ID: {}] {}", remote.id(), name))
            .unwrap_or_else(|| "[ID: -1] bugged name".to_owned())
    }
}

#[repr(C, packed)]
pub struct CRemotePlayer {
    id: ID,
    vehicle_id: ID,
    ped: *mut CPed,
}

impl CRemotePlayer {
    fn game_ped(&self) -> Option<&super::players::GamePed> {
        let ped = unsafe { std::ptr::addr_of!(self.ped).read_unaligned() };
        if ped.is_null() {
            return None;
        }

        let game_ped = unsafe { std::ptr::addr_of!((*ped).game_ped).read_unaligned() };
        (!game_ped.is_null()).then(|| unsafe { &*game_ped })
    }

    pub fn matrix(&self) -> Option<RwMatrix> {
        let matrix = self.game_ped()?.matrix;
        (!matrix.is_null()).then(|| unsafe { matrix.read() })
    }

    pub fn ped_position(&self) -> CVector {
        self.matrix()
            .map(|matrix| matrix.pos)
            .unwrap_or_else(CVector::zero)
    }

    pub fn velocity(&self) -> CVector {
        game_ped_velocity(self.game_ped())
    }

    pub fn head_direction(&self) -> CVector {
        self.matrix()
            .map(|matrix| matrix.at)
            .unwrap_or_else(CVector::zero)
    }

    pub fn id(&self) -> ID {
        unsafe { std::ptr::addr_of!(self.id).read_unaligned() }
    }
}

#[repr(C, packed)]
pub struct CLocalPlayer {
    pub ped: *mut CPed,
}

impl CLocalPlayer {
    fn game_ped(&self) -> Option<&super::players::GamePed> {
        let ped = unsafe { std::ptr::addr_of!(self.ped).read_unaligned() };
        if ped.is_null() {
            return None;
        }

        let game_ped = unsafe { std::ptr::addr_of!((*ped).game_ped).read_unaligned() };
        (!game_ped.is_null()).then(|| unsafe { &*game_ped })
    }

    pub fn matrix(&self) -> Option<RwMatrix> {
        let matrix = self.game_ped()?.matrix;
        (!matrix.is_null()).then(|| unsafe { matrix.read() })
    }

    pub fn ped_position(&self) -> CVector {
        self.matrix()
            .map(|matrix| matrix.pos)
            .unwrap_or_else(CVector::zero)
    }

    pub fn velocity(&self) -> CVector {
        game_ped_velocity(self.game_ped())
    }

    pub fn game_ped_ptr(&self) -> *const () {
        self.game_ped()
            .map(|ped| ped as *const _ as *const ())
            .unwrap_or(std::ptr::null())
    }

    pub fn name(&self) -> Option<&str> {
        let pool = player_pool()?;
        unsafe { CStdString::as_str_from_ptr(std::ptr::addr_of!(pool.local_player_name)).ok() }
    }

    pub fn id(&self) -> Option<i32> {
        player_pool()
            .map(|pool| unsafe { std::ptr::addr_of!(pool.local_player_id).read_unaligned() as i32 })
    }
}

#[repr(C, packed)]
pub struct CPed {
    base: CEntity,
    using_cellphone: BOOL,
    accessories: [u8; 600],
    game_ped: *mut super::players::GamePed,
}

#[repr(C, packed)]
pub struct CEntity {
    vtable: *const u8,
    pad_4: [u8; 60],
    pub game_entity: *mut crate::gta::entity::CEntity,
    handle: GTAREF,
}

#[repr(C, packed)]
pub struct CObjectPool {
    largest_id: i32,
    not_empty: [BOOL; 2100],
    objects: [*mut CObject; 2100],
}

#[repr(C, packed)]
pub struct CObject {
    pub base: CEntity,
}

fn game_ped_velocity(ped: Option<&super::players::GamePed>) -> CVector {
    ped.map(|ped| unsafe {
        let physical = ped as *const _ as *const crate::gta::physical::CPhysical;
        std::ptr::addr_of!((*physical).m_vecMoveSpeed).read_unaligned()
    })
    .unwrap_or_else(CVector::zero)
}

pub fn netgame() -> *mut CNetGame {
    unsafe { (handle().add(CNETGAME) as *mut *mut CNetGame).read() }
}

pub fn local_player<'a>() -> Option<&'a mut CLocalPlayer> {
    let ptr = player_pool()
        .map(|pool| unsafe { std::ptr::addr_of!(pool.local_player).read_unaligned() })?;
    (!ptr.is_null()).then(|| unsafe { &mut *ptr })
}

pub fn find_player<'a>(player_id: i32) -> Option<&'a CPlayerInfo> {
    let index = usize::try_from(player_id).ok().filter(|&id| id < 1000)?;
    let players = unsafe { std::ptr::addr_of!(player_pool()?.players).read_unaligned() };
    let ptr = players[index];
    (!ptr.is_null()).then(|| unsafe { &*ptr })
}

pub fn player_pool() -> Option<&'static mut CPlayerPool> {
    let ptr = unsafe { std::ptr::addr_of!(pools()?.player).read_unaligned() };
    (!ptr.is_null()).then(|| unsafe { &mut *ptr })
}

pub(crate) fn player_slots() -> Option<[*mut CPlayerInfo; 1004]> {
    player_pool().map(|pool| unsafe { std::ptr::addr_of!(pool.players).read_unaligned() })
}

pub fn find_object<'a>(object_id: i32) -> Option<&'a mut CObject> {
    let index = usize::try_from(object_id).ok().filter(|&id| id < 2100)?;
    let objects = unsafe { std::ptr::addr_of!(object_pool()?.objects).read_unaligned() };
    let ptr = objects[index];
    (!ptr.is_null()).then(|| unsafe { &mut *ptr })
}

fn object_pool() -> Option<&'static mut CObjectPool> {
    let ptr = unsafe { std::ptr::addr_of!(pools()?.object).read_unaligned() };
    (!ptr.is_null()).then(|| unsafe { &mut *ptr })
}

fn pools() -> Option<&'static mut CNetGamePools> {
    let netgame = CNetGame::get()?;
    let ptr = unsafe { std::ptr::addr_of!(netgame.pools).read_unaligned() };
    (!ptr.is_null()).then(|| unsafe { &mut *ptr })
}
