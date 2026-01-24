use super::packets;
use super::players::GamePed;
use super::{handle, CStdString, BOOL, D3DCOLOR, GTAREF, ID, NUMBER, TICK};
use crate::gta::matrix::{CVector, RwMatrix};

use std::ffi::{c_void, CStr};
use std::net::{Ipv4Addr, SocketAddr};

pub const CNETGAME: usize = 0x26E8DC;
pub const CINPUT: usize = 0x26E8CC;
pub const CGAME: usize = 0x26E8F4;
pub const CGAME_SETCURSORMODE: usize = 0x9FFE0;
pub const CGAME_PROCESSINPUTENABLING: usize = 0x9FEC0;
pub const CDIALOG: usize = 0x26E898;
pub const CDEATHWINDOW_DRAW: usize = 0x69B70;

const SPEC_MODE_VEHICLE: i8 = 3;
const SPEC_MODE_PLAYER: i8 = 4;
const SPEC_MODE_FIXED: i8 = 15;
const SPEC_MODE_SIDE: i8 = 14;

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
    fn from(state: i32) -> Gamestate {
        match state {
            0x01 => Gamestate::WaitConnect,
            0x02 => Gamestate::Connecting,
            0x06 => Gamestate::AwaitJoin,
            0x05 => Gamestate::Connected,
            0x0B => Gamestate::Restarting,
            _ => Gamestate::None,
        }
    }
}

#[repr(C, packed)]
pub struct CNetGame {
    pub pad_0: [::std::os::raw::c_char; 44usize],
    pub m_pRakClient: *mut (),
    pub m_szHostAddress: [::std::os::raw::c_uchar; 257usize],
    pub m_szHostname: [::std::os::raw::c_char; 257usize],
    pub m_bDisableCollision: bool,
    pub m_bUpdateCameraTarget: bool,
    pub m_bNametagStatus: bool,
    pub m_nPort: ::std::os::raw::c_int,
    pub m_bLanMode: BOOL,
    pub m_aMapIcons: [GTAREF; 100usize],
    pub m_nGameState: ::std::os::raw::c_int,
    pub m_lastConnectAttempt: TICK,
    pub m_pSettings: *mut (),
    pub pad_2: [::std::os::raw::c_char; 5usize],
    pub m_pPools: *mut CNetGame_Pools,
}

impl CNetGame {
    pub fn get<'a>() -> Option<&'a mut CNetGame> {
        let ptr = netgame();

        if ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *ptr) }
        }
    }

    pub fn addr(&self) -> Option<SocketAddr> {
        let iter = self.m_szHostAddress.iter().take_while(|&&byte| byte != 0);
        let addr = String::from_utf8(iter.cloned().collect()).ok()?;
        let addr: Ipv4Addr = addr.parse().ok()?;

        Some(SocketAddr::from((addr, self.m_nPort as u16)))
    }

    pub fn gamestate(&self) -> Gamestate {
        Gamestate::from(self.m_nGameState)
    }
}

#[repr(C, packed)]
pub struct CNetGame_Pools {
    pub m_pMenu: *mut (),
    pub m_pActor: *mut (),
    pub m_pPlayer: *mut CPlayerPool,
    pub m_pVehicle: *mut CVehiclePool,
    pub m_pPickup: *mut (),
    pub m_pObject: *mut CObjectPool,
    pub m_pGangZone: *mut (),
    pub m_pLabel: *mut (),
    pub m_pTextDraw: *mut (),
}

#[repr(C, packed)]
pub struct CPlayerPool {
    pub m_nLargestId: ::std::os::raw::c_int,
    pub m_pObject: [*mut CPlayerInfo; 1004usize],
    pub m_bNotEmpty: [BOOL; 1004usize],
    pub m_bPrevCollisionFlag: [BOOL; 1004usize],
    pub m_localInfo: CPlayerPool__bindgen_ty_1,
}

#[repr(C, packed)]
pub struct CPlayerPool__bindgen_ty_1 {
    pub m_nPing: ::std::os::raw::c_int,
    pub m_nScore: ::std::os::raw::c_int,
    pub m_nId: ID,
    pub __align: ::std::os::raw::c_int,
    pub m_szName: CStdString,
    pub m_pObject: *mut CLocalPlayer,
}

#[repr(C, packed)]
pub struct CPlayerInfo {
    pub m_pPlayer: *mut CRemotePlayer,
    pub m_nPing: ::std::os::raw::c_int,
    pub __aling: ::std::os::raw::c_int,
    pub m_szNick: CStdString,
    pub m_nScore: ::std::os::raw::c_int,
    pub m_bIsNPC: BOOL,
}

impl CPlayerInfo {
    pub fn remote_player_mut(&mut self) -> Option<&mut CRemotePlayer> {
        if self.m_pPlayer.is_null() {
            return None;
        } else {
            return Some(unsafe { &mut *self.m_pPlayer });
        }
    }

    pub fn remote_player(&self) -> Option<&CRemotePlayer> {
        if self.m_pPlayer.is_null() {
            return None;
        } else {
            return Some(unsafe { &*self.m_pPlayer });
        }
    }

    pub fn gta_ped(&self) -> Option<&super::players::GamePed> {
        self.remote_player()
            .filter(|remote| !remote.m_pPed.is_null())
            .filter(|remote| unsafe { !remote.m_pPed.read().m_pGamePed.is_null() })
            .map(|remote| unsafe { &*remote.m_pPed.read().m_pGamePed })
    }

    pub fn is_in_stream(&self) -> bool {
        self.remote_player()
            .filter(|remote| !remote.m_pPed.is_null())
            .map(|remote| unsafe { !remote.m_pPed.read().m_pGamePed.is_null() })
            .unwrap_or(false)
    }

    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        unsafe {
            CStdString::as_str_from_ptr(std::ptr::addr_of!(self.m_szNick))
        }
        .map(|name| {
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            hasher.finish()
        })
        .unwrap_or(0)
    }

    pub fn name(&self) -> Option<&str> {
        unsafe { CStdString::as_str_from_ptr(std::ptr::addr_of!(self.m_szNick)).ok() }
    }

    pub fn name_with_id(&self) -> String {
        unsafe { CStdString::as_str_from_ptr(std::ptr::addr_of!(self.m_szNick)).ok() }
            .and_then(|name| {
                let remote = self.remote_player()?;
                let id = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(remote.m_nId)) };
                Some(format!("[ID: {}] {}", id, name))
            })
            .unwrap_or_else(|| "[ID: -1] bugged name".to_owned())
    }
}

#[repr(C, packed)]
pub struct CRemotePlayer {
    pub m_pPed: *mut CPed,         // CPed
    pub m_pVehicle: *mut CVehicle, // CVehicle
    pub m_nId: ID,
    pub m_nVehicleId: ID,
    pub field_1: ::std::os::raw::c_int,
    pub m_bDrawLabels: BOOL,
    pub m_bHasJetpack: BOOL,
    pub m_nSpecialAction: ::std::os::raw::c_uchar,
    pub m_incarData: packets::IncarData,
    pub m_trailerData: packets::TrailerData,
    pub m_aimData: packets::AimData,
    pub m_passengerData: packets::PassengerData,
    pub m_onfootData: packets::OnfootData,
    pub m_nTeam: ::std::os::raw::c_uchar,
    pub m_nState: ::std::os::raw::c_uchar,
    pub m_nSeatId: ::std::os::raw::c_uchar,
    pub field_3: ::std::os::raw::c_int,
    pub m_bPassengerDriveBy: BOOL,
    pub m_onfootTargetPosition: CVector,
    pub m_onfootTargetSpeed: CVector,
    pub m_incarTargetPosition: CVector,
    pub m_incarTargetSpeed: CVector,
    pub pad_1: [::std::os::raw::c_char; 76usize],
    pub m_positionDifference: CVector,
    pub m_incarTargetRotation: CRemotePlayer__bindgen_ty_1,
    pub m_fReportedArmour: f32,
    pub m_fReportedHealth: f32,
    pub pad_2: [::std::os::raw::c_char; 12usize],
    pub m_animation: Animation,
    pub m_nUpdateType: ::std::os::raw::c_uchar,
    pub m_lastUpdate: TICK,
    pub m_lastTimestamp: TICK,
    pub m_bPerformingCustomAnimation: BOOL,
    pub m_nStatus: ::std::os::raw::c_int,
    pub m_head: CRemotePlayer__bindgen_ty_2,
    pub m_bMarkerState: BOOL,
    pub m_markerPosition: CRemotePlayer__bindgen_ty_3,
    pub m_marker: GTAREF,
}

impl CRemotePlayer {
    pub fn matrix(&self) -> Option<RwMatrix> {
        unsafe {
            if self.m_pPed.is_null() {
                return None;
            }

            let matrix = self.m_pPed.read().m_pGamePed.read().matrix.read();

            Some(matrix)
        }
    }

    pub fn position(&self) -> CVector {
        match self.m_nState {
            17 => self.m_onfootData.m_position.clone(),    // onfoot
            18 => self.m_passengerData.m_position.clone(), // passenger
            19 => self.m_incarData.m_position.clone(),     // driver
            _ => CVector::zero(),                          // none
        }
    }

    pub fn ped_position(&self) -> CVector {
        if self.m_pPed.is_null() {
            return CVector::zero();
        }

        let ped = unsafe { &*self.m_pPed };

        if ped.m_pGamePed.is_null() {
            return CVector::zero();
        }

        unsafe { ped.m_pGamePed.read().matrix.read().pos }
    }

    pub fn velocity(&self) -> CVector {
        match self.m_nState {
            17 => self.m_onfootData.m_speed.clone(), // onfoot
            19 => self.m_incarData.m_speed.clone(),  // driver
            _ => CVector::zero(),
        }
    }

    pub fn head_direction(&self) -> CVector {
        self.m_head.m_direction.clone()
    }

    pub fn id(&self) -> ID {
        self.m_nId
    }
}

#[repr(C, packed)]
pub struct CRemotePlayer__bindgen_ty_1 {
    pub real: f32,
    pub imag: CVector,
}

#[repr(C, packed)]
pub struct CRemotePlayer__bindgen_ty_2 {
    pub m_direction: CVector,
    pub m_lastUpdate: TICK,
    pub m_lastLook: TICK,
}

#[repr(C, packed)]
pub struct CRemotePlayer__bindgen_ty_3 {
    pub x: ::std::os::raw::c_int,
    pub y: ::std::os::raw::c_int,
    pub z: ::std::os::raw::c_int,
}

#[repr(C, packed)]
pub struct Animation {
    pub shit: u32,
}

#[repr(C, packed)]
pub struct CLocalPlayer {
    pub m_pPed: *mut CPed, // CPed
    pub m_incarData: packets::IncarData,
    pub m_aimData: packets::AimData,
    pub m_trailerData: packets::TrailerData,
    pub m_onfootData: packets::OnfootData,
    pub m_passengerData: packets::PassengerData,
    pub m_bIsActive: BOOL,
    pub m_bIsWasted: BOOL,
    pub m_nCurrentVehicle: ID,
    pub m_nLastVehicle: ID,
    pub m_animation: Animation,
    pub field_1: ::std::os::raw::c_int,
    pub m_bDoesSpectating: BOOL,
    pub m_nTeam: NUMBER,
    pub field_10d: ::std::os::raw::c_short,
    pub m_lastUpdate: TICK,
    pub m_lastSpecUpdate: TICK,
    pub m_lastAimUpdate: TICK,
    pub m_lastStatsUpdate: TICK,
    pub m_cameraTarget: CLocalPlayer_CameraTarget,
    pub m_lastCameraTargetUpdate: TICK,
    pub m_head: CLocalPlayer__bindgen_ty_1,
    pub m_lastAnyUpdate: TICK,
    pub m_bClearedToSpawn: BOOL,
    pub m_lastSelectionTick: TICK,
    pub m_initialSelectionTick: TICK,
    pub m_spawnInfo: CLocalPlayer_SpawnInfo,
    pub m_bHasSpawnInfo: BOOL,
    pub m_lastWeaponsUpdate: TICK,
    pub m_weaponsData: CLocalPlayer__bindgen_ty_2,
    pub m_bPassengerDriveBy: BOOL,
    pub m_nCurrentInterior: ::std::os::raw::c_char,
    pub m_bInRCMode: BOOL,
    pub m_szName: [::std::os::raw::c_char; 256usize],
    pub m_surfing: CLocalPlayer__bindgen_ty_3,
    pub m_classSelection: CLocalPlayer__bindgen_ty_4,
    pub m_zoneDisplayingEnd: TICK,
    pub m_spectating: CLocalPlayer__bindgen_ty_5,
    pub m_damage: CLocalPlayer__bindgen_ty_6,
}

impl CLocalPlayer {
    pub fn matrix(&self) -> Option<RwMatrix> {
        unsafe {
            if self.m_pPed.is_null() {
                return None;
            }

            let matrix = self.m_pPed.read().m_pGamePed.read().matrix.read();

            Some(matrix)
        }
    }

    pub fn position(&self) -> CVector {
        if self.m_spectating.m_nMode != 0 {
            return self.spec_position();
        }

        if self.m_nCurrentVehicle != u16::max_value() {
            if self.m_passengerData.m_nSeatId > 0 {
                self.m_passengerData.m_position.clone() // passenger
            } else {
                self.m_incarData.m_position.clone() // driver
            }
        } else {
            self.m_onfootData.m_position.clone() // onfoot
        }
    }

    pub fn ped_position(&self) -> CVector {
        if self.m_spectating.m_nMode != 0 && self.m_spectating.m_nObject != -1 {
            return self.spec_position();
        }

        if self.m_pPed.is_null() {
            return CVector::zero();
        }

        let ped = unsafe { &*self.m_pPed };

        if ped.m_pGamePed.is_null() {
            return CVector::zero();
        }

        unsafe { ped.m_pGamePed.read().matrix.read().pos }
    }

    pub fn spec_position(&self) -> CVector {
        let null_vec = CVector::zero();

        match self.m_spectating.m_nMode {
            SPEC_MODE_PLAYER => {
                let player_id = self.m_spectating.m_nObject;
                let position = find_player(player_id)
                    .and_then(|player| player.remote_player())
                    .map(|player| player.ped_position())
                    .unwrap_or(null_vec);

                return position;
            }

            SPEC_MODE_VEHICLE => {
                let vehicle_id = self.m_spectating.m_nObject;
                let position = find_vehicle(vehicle_id)
                    .map(|vehicle| vehicle.position())
                    .unwrap_or(null_vec);

                return position;
            }

            _ => (),
        }

        return null_vec;
    }

    pub fn velocity(&self) -> CVector {
        if self.m_nCurrentVehicle != u16::max_value() {
            self.m_incarData.m_speed.clone() // driver
        } else {
            self.m_onfootData.m_speed.clone() // onfoot
        }
    }

    pub fn name(&self) -> Option<&str> {
        player_pool().and_then(|players| unsafe {
            CStdString::as_str_from_ptr(std::ptr::addr_of!(players.m_localInfo.m_szName)).ok()
        })
    }

    pub fn id(&self) -> Option<i32> {
        player_pool().map(|players| unsafe {
            std::ptr::read_unaligned(std::ptr::addr_of!(players.m_localInfo.m_nId)) as i32
        })
    }
}

#[repr(C, packed)]
pub struct CLocalPlayer_CameraTarget {
    pub m_nObject: ID,
    pub m_nVehicle: ID,
    pub m_nPlayer: ID,
    pub m_nActor: ID,
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_1 {
    pub m_direction: CVector,
    pub m_lastUpdate: TICK,
    pub m_lastLook: TICK,
}

#[repr(C, packed)]
pub struct CLocalPlayer_SpawnInfo {
    pub m_nTeam: NUMBER,
    pub m_nSkin: ::std::os::raw::c_int,
    pub field_c: ::std::os::raw::c_char,
    pub m_position: CVector,
    pub m_fRotation: f32,
    pub m_aWeapon: [::std::os::raw::c_int; 3usize],
    pub m_aAmmo: [::std::os::raw::c_int; 3usize],
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_2 {
    pub m_nAimedPlayer: ID,
    pub m_nAimedActor: ID,
    pub m_nCurrentWeapon: NUMBER,
    pub m_aLastWeapon: [NUMBER; 13usize],
    pub m_aLastWeaponAmmo: [::std::os::raw::c_int; 13usize],
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_3 {
    pub m_nEntityId: ID,
    pub m_lastUpdate: TICK,
    pub __bindgen_anon_1: CLocalPlayer__bindgen_ty_3__bindgen_ty_1,
    pub m_bStuck: BOOL,
    pub m_bIsActive: BOOL,
    pub m_position: CVector,
    pub field_: ::std::os::raw::c_int,
    pub m_nMode: ::std::os::raw::c_int,
}

#[repr(C, packed)]
pub union CLocalPlayer__bindgen_ty_3__bindgen_ty_1 {
    pub m_pVehicle: *mut CVehicle, // CVehicle
    pub m_pObject: *mut (),
    _bindgen_union_align: [u8; 4usize],
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_4 {
    pub m_bEnableAfterDeath: BOOL,
    pub m_nSelected: ::std::os::raw::c_int,
    pub m_bWaitingForSpawnRequestReply: BOOL,
    pub m_bIsActive: BOOL,
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_5 {
    pub m_nMode: ::std::os::raw::c_char,
    pub m_nType: ::std::os::raw::c_char,
    pub m_nObject: ::std::os::raw::c_int,
    pub m_bProcessed: BOOL,
}

#[repr(C, packed)]
pub struct CLocalPlayer__bindgen_ty_6 {
    pub m_nVehicleUpdating: ID,
    pub m_nBumper: ::std::os::raw::c_int,
    pub m_nDoor: ::std::os::raw::c_int,
    pub m_bLight: bool,
    pub m_bWheel: bool,
}

#[repr(C, packed)]
pub struct CVehiclePool {
    pub m_nCount: ::std::os::raw::c_int,
    pub m_waitingList: CVehiclePool__bindgen_ty_1,
    pub m_pObject: [*mut CVehicle; 2000usize],
    pub m_bNotEmpty: [BOOL; 2000usize],
    pub m_pGameObject: [*mut GamePed; 2000usize], // GTA::CVehicle
    pub pad_6ef4: [::std::os::raw::c_uint; 2000usize],
    pub m_nLastUndrivenId: [ID; 2000usize],
    pub m_lastUndrivenProcessTick: [TICK; 2000usize],
    pub m_bIsActive: [BOOL; 2000usize],
    pub m_bIsDestroyed: [BOOL; 2000usize],
    pub m_tickWhenDestroyed: [TICK; 2000usize],
    pub m_spawnedAt: [CVector; 2000usize],
    pub m_bNeedsToInitializeLicensePlates: BOOL,
}

#[repr(C, packed)]
pub struct CVehiclePool_Info {
    pub m_nId: ID,
    pub m_nType: ::std::os::raw::c_int,
    pub m_position: CVector,
    pub m_fRotation: f32,
    pub m_nPrimaryColor: NUMBER,
    pub m_nSecondaryColor: NUMBER,
    pub m_fHealth: f32,
    pub m_nInterior: ::std::os::raw::c_char,
    pub m_nDoorDamageStatus: ::std::os::raw::c_int,
    pub m_nPanelDamageStatus: ::std::os::raw::c_int,
    pub m_nLightDamageStatus: ::std::os::raw::c_char,
    pub m_bDoorsLocked: bool,
    pub m_bHasSiren: bool,
}

#[repr(C, packed)]
pub struct CVehiclePool__bindgen_ty_1 {
    pub m_entry: [CVehiclePool_Info; 100usize],
    pub m_bNotEmpty: [BOOL; 100usize],
}

#[repr(C, packed)]
pub struct CVehicle {
    pub _base: CEntity,
    pub m_pTrailer: *mut CVehicle,
    pub m_pGameVehicle: *mut GamePed, // GTA::CVehicle
    pub pad_50: [::std::os::raw::c_char; 8usize],
    pub m_bIsInvulnerable: BOOL,
    pub m_bIsLightsOn: BOOL,
    pub m_bIsLocked: BOOL,
    pub m_bIsObjective: bool,
    pub m_bObjectiveBlipCreated: BOOL,
    pub m_timeSinceLastDriven: TICK,
    pub m_bHasBeenDriven: BOOL,
    pub pad_71: [::std::os::raw::c_char; 4usize],
    pub m_bEngineState: BOOL,
    pub m_nPrimaryColor: ::std::os::raw::c_uchar,
    pub m_nSecondaryColor: ::std::os::raw::c_uchar,
    pub m_bNeedsToUpdateColor: BOOL,
    pub m_bUnoccupiedSync: BOOL,
    pub m_bRemoteUnocSync: BOOL,
    pub m_bKeepModelLoaded: BOOL,
    pub m_bHasSiren: ::std::os::raw::c_int,
    pub m_pLicensePlate: *mut (), // IDirect3DTexture9
    pub m_szLicensePlateText: [::std::os::raw::c_char; 33usize],
    pub m_marker: GTAREF,
}

impl CVehicle {
    pub fn position(&self) -> CVector {
        if self.m_pGameVehicle.is_null() {
            return CVector::zero();
        }

        let placeable = unsafe { &self.m_pGameVehicle.read() };
        unsafe { placeable.matrix.read().pos }
    }
}

#[repr(C, packed)]
pub struct CEntity {
    pub vtable_: *const u8,
    pub pad_4: [std::os::raw::c_char; 60],
    pub m_pGameEntity: *mut *mut crate::gta::entity::CEntity,
    pub m_handle: GTAREF,
}

#[repr(C, packed)]
pub struct CPed {
    pub _base: CEntity,
    pub m_bUsingCellphone: BOOL,
    pub m_accessories: CPed__bindgen_ty_1,
    pub m_pGamePed: *mut GamePed, // GTA::CPed
    pub pad_2a8: [::std::os::raw::c_uint; 2usize],
    pub m_nPlayerNumber: ::std::os::raw::c_uchar,
    pub pad_2b1: [::std::os::raw::c_uint; 2usize],
    pub m_parachuteObject: GTAREF,
    pub m_urinatingParticle: GTAREF,
    pub m_stuff: CPed__bindgen_ty_2,
    pub m_arrow: GTAREF,
    pub field_2de: ::std::os::raw::c_uchar,
    pub m_bDoesDancing: BOOL,
    pub m_nDanceStyle: ::std::os::raw::c_uint,
    pub m_nLastDanceMove: ::std::os::raw::c_uint,
    pub pad_2de: [::std::os::raw::c_uchar; 20usize],
    pub m_bDoesUrinating: BOOL,
    pub pad: [::std::os::raw::c_uchar; 55usize],
}

#[repr(C, packed)]
pub struct CPed__bindgen_ty_1 {
    pub m_bNotEmpty: [BOOL; 10usize],
    pub m_info: [Accessory; 10usize],
    pub m_pObject: [*mut (); 10usize], // CObject
}

#[repr(C, packed)]
pub struct CPed__bindgen_ty_2 {
    pub m_nType: ::std::os::raw::c_int,
    pub m_object: GTAREF,
    pub m_nDrunkLevel: ::std::os::raw::c_uint,
}

#[repr(C, packed)]
pub struct Accessory {
    pub m_nModel: ::std::os::raw::c_int,
    pub m_nBone: ::std::os::raw::c_int,
    pub m_offset: CVector,
    pub m_rotation: CVector,
    pub m_scale: CVector,
    pub m_firstMaterialColor: D3DCOLOR,
    pub m_secondMaterialColor: D3DCOLOR,
}

#[repr(C, packed)]
pub struct CObjectPool {
    pub m_nLargestId: ::std::os::raw::c_int,
    pub m_bNotEmpty: [BOOL; 1000usize],
    pub m_pObject: [*mut CObject; 1000usize],
}

#[repr(C, packed)]
pub struct CObject {
    pub _base: CEntity,
    pub pad_0: [::std::os::raw::c_char; 6usize],
    pub m_nModel: ::std::os::raw::c_int,
    pub pad_1: ::std::os::raw::c_char,
    pub m_bDontCollideWithCamera: bool,
    pub m_fDrawDistance: f32,
    pub field_0: f32,
    pub m_position: CVector,
    pub m_fDistanceToCamera: f32,
    pub m_bDrawLast: bool,
    pub pad_2: [::std::os::raw::c_char; 64usize],
    pub m_rotation: CVector,
    pub pad_3: [::std::os::raw::c_char; 5usize],
    pub m_nAttachedToVehicle: ID,
    pub m_nAttachedToObject: ID,
    pub m_attachOffset: CVector,
    pub m_attachRotation: CVector,
    pub field_1: ::std::os::raw::c_char,
    pub m_targetMatrix: RwMatrix,
    pub pad_4: [::std::os::raw::c_char; 148usize],
    pub m_bMoving: ::std::os::raw::c_char,
    pub m_fSpeed: f32,
    pub pad_5: [::std::os::raw::c_char; 99usize],
    pub m_material: CObject__bindgen_ty_1,
    pub m_bHasCustomMaterial: BOOL,
    pub pad_9: [::std::os::raw::c_char; 13usize],
}

#[repr(C, packed)]
pub struct CObject__bindgen_ty_1 {
    pub __bindgen_anon_1: CObject__bindgen_ty_1__bindgen_ty_1,
    pub m_color: [D3DCOLOR; 16usize],
    pub pad_6: [::std::os::raw::c_char; 68usize],
    pub m_nType: [::std::os::raw::c_int; 16usize],
    pub m_text: CObject__bindgen_ty_1__bindgen_ty_2,
}

#[repr(C, packed)]
pub struct CObject__bindgen_ty_1__bindgen_ty_1 {
    pub bindgen_union_field: [u8; 64usize],
}

#[repr(C, packed)]
pub struct CObject__bindgen_ty_1__bindgen_ty_2 {
    pub m_bTextureWasCreated: [BOOL; 16usize],
    pub m_textInfo: [CObject__bindgen_ty_1__bindgen_ty_2__bindgen_ty_1; 16usize],
    pub m_szData: [*mut ::std::os::raw::c_char; 16usize],
    pub m_pBackgroundTexture: [*mut (); 16usize], // IDirect3DTexture9
    pub m_pTexture: [*mut (); 16usize],           // IDirect3DTexture9
}

#[repr(C, packed)]
pub struct CObject__bindgen_ty_1__bindgen_ty_2__bindgen_ty_1 {
    pub m_nMaterialIndex: ::std::os::raw::c_char,
    pub pad_0: [::std::os::raw::c_char; 137usize],
    pub m_nMaterialSize: ::std::os::raw::c_char,
    pub m_szFont: [::std::os::raw::c_char; 65usize],
    pub m_nFontSize: ::std::os::raw::c_char,
    pub m_bBold: bool,
    pub m_fontColor: D3DCOLOR,
    pub m_backgroundColor: D3DCOLOR,
    pub m_align: ::std::os::raw::c_char,
}

pub fn netgame() -> *mut CNetGame {
    unsafe { *(handle().add(CNETGAME) as *mut *mut CNetGame) }
}

pub fn players<'a>() -> Option<impl Iterator<Item = &'a mut CPlayerInfo>> {
    player_pool().map(|pool| {
        let players = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(pool.m_pObject)) };
        players
            .into_iter()
            .filter(|player| !player.is_null())
            .map(|player| unsafe { &mut *player })
    })
}

pub fn find_player<'a>(player_id: i32) -> Option<&'a CPlayerInfo> {
    if player_id < 0 || player_id >= 1000 {
        return None;
    }

    unsafe {
        let player_id = player_id as usize;

        if let Some(players) = player_pool() {
            let list = std::ptr::read_unaligned(std::ptr::addr_of!(players.m_pObject));

            if list[player_id].is_null() {
                return None;
            }

            Some(&mut *list[player_id])
        } else {
            None
        }
    }
}

pub fn find_vehicle<'a>(vehicle_id: i32) -> Option<&'a mut CVehicle> {
    if vehicle_id < 0 || vehicle_id >= 2000 {
        return None;
    }

    unsafe {
        let vehicle_id = vehicle_id as usize;

        if let Some(vehicles) = vehicle_pool() {
            let list = std::ptr::read_unaligned(std::ptr::addr_of!(vehicles.m_pObject));

            if list[vehicle_id].is_null() {
                return None;
            }

            Some(&mut *list[vehicle_id])
        } else {
            None
        }
    }
}

pub fn local_player<'a>() -> Option<&'a mut CLocalPlayer> {
    player_pool().and_then(|pool| unsafe {
        let local_ptr = std::ptr::read_unaligned(std::ptr::addr_of!(pool.m_localInfo.m_pObject));

        if local_ptr.is_null() {
            None
        } else {
            Some(&mut *local_ptr)
        }
    })
}

pub fn player_pool() -> Option<&'static mut CPlayerPool> {
    unsafe {
        let pools = pools()?;

        if pools.m_pPlayer.is_null() {
            return None;
        }

        Some(&mut *(*pools).m_pPlayer)
    }
}

pub fn vehicle_pool() -> Option<&'static mut CVehiclePool> {
    unsafe {
        let pools = pools()?;

        if pools.m_pVehicle.is_null() {
            return None;
        }

        Some(&mut *(*pools).m_pVehicle)
    }
}

pub fn object_pool() -> Option<&'static mut CObjectPool> {
    unsafe {
        let pools = pools()?;

        if pools.m_pObject.is_null() {
            return None;
        }

        Some(&mut *(*pools).m_pObject)
    }
}

pub fn find_object<'a>(object_id: i32) -> Option<&'a mut CObject> {
    if object_id < 0 || object_id >= 1000 {
        return None;
    }

    unsafe {
        let object_id = object_id as usize;

        if let Some(objects) = object_pool() {
            if objects.m_pObject[object_id].is_null() {
                return None;
            }

            Some(&mut *objects.m_pObject[object_id])
        } else {
            None
        }
    }
}

fn pools() -> Option<&'static mut CNetGame_Pools> {
    unsafe {
        let netgame = netgame();

        if netgame.is_null() {
            return None;
        }

        if (*netgame).m_pPools.is_null() {
            return None;
        }

        let pools = &mut *(*netgame).m_pPools;

        Some(pools)
    }
}
