use super::version::{version, Version};
use super::{v037 as r1, v037r3 as r3};
use crate::gta::matrix::{CVector, RwMatrix};

#[repr(C, packed)]
pub struct GamePed {
    pad: [u8; 20],
    pub matrix: *mut RwMatrix,
}

pub struct LocalPlayer<'a> {
    player_v1: Option<&'a mut r1::CLocalPlayer>,
    player_v3: Option<&'a mut r3::CLocalPlayer>,
}

impl<'a> LocalPlayer<'a> {
    pub fn matrix(&self) -> Option<RwMatrix> {
        if let Some(player) = self.player_v1.as_ref() {
            return player.matrix();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.matrix();
        }

        None
    }

    pub fn position(&self) -> CVector {
        if let Some(player) = self.player_v1.as_ref() {
            return player.ped_position();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.ped_position();
        }

        CVector::zero()
    }

    pub fn velocity(&self) -> CVector {
        if let Some(player) = self.player_v1.as_ref() {
            return player.velocity();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.velocity();
        }

        CVector::zero()
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(player) = self.player_v1.as_ref() {
            return player.name();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.name();
        }

        None
    }

    pub fn id(&self) -> Option<i32> {
        if let Some(player) = self.player_v1.as_ref() {
            return player.id();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.id();
        }

        None
    }

    pub fn ped(&self) -> *const () {
        if let Some(player) = self.player_v1.as_ref() {
            return unsafe { (*player.m_pPed).m_pGamePed as *mut _ };
        }

        if let Some(player) = self.player_v3.as_ref() {
            return unsafe { (*player.m_pPed).m_pGamePed as *mut _ };
        }

        std::ptr::null()
    }
}

pub struct PlayerPool<'a> {
    pool_v1: Option<&'a mut r1::CPlayerPool>,
    pool_v3: Option<&'a mut r3::CPlayerPool>,
}

pub struct Player<'a> {
    player_v1: Option<&'a r1::CPlayerInfo>,
    player_v3: Option<&'a r3::CPlayerInfo>,
}

impl<'a> Player<'a> {
    fn new_v1(player: &'a r1::CPlayerInfo) -> Player<'a> {
        Player {
            player_v1: Some(player),
            player_v3: None,
        }
    }

    fn new_v3(player: &'a r3::CPlayerInfo) -> Player<'a> {
        Player {
            player_v3: Some(player),
            player_v1: None,
        }
    }

    pub fn remote_player(&self) -> Option<RemotePlayer> {
        let remote_v1 = self
            .player_v1
            .as_ref()
            .and_then(|player| player.remote_player());
        let remote_v3 = self
            .player_v3
            .as_ref()
            .and_then(|player| player.remote_player());

        if remote_v1.is_none() && remote_v3.is_none() {
            return None;
        }

        Some(RemotePlayer {
            remote_v1,
            remote_v3,
        })
    }

    pub fn gta_ped(&self) -> Option<&GamePed> {
        if let Some(player) = self.player_v1.as_ref() {
            return player.gta_ped();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.gta_ped();
        }

        None
    }

    pub fn is_in_stream(&self) -> bool {
        if let Some(player) = self.player_v1.as_ref() {
            return player.is_in_stream();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.is_in_stream();
        }

        false
    }

    pub fn hash(&self) -> u64 {
        if let Some(player) = self.player_v1.as_ref() {
            return player.hash();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.hash();
        }

        u64::max_value()
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(player) = self.player_v1.as_ref() {
            return player.name();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.name();
        }

        None
    }

    pub fn name_with_id(&self) -> String {
        if let Some(player) = self.player_v1.as_ref() {
            return player.name_with_id();
        }

        if let Some(player) = self.player_v3.as_ref() {
            return player.name_with_id();
        }

        "[ID: -1] noname".to_string()
    }
}

pub struct RemotePlayer<'a> {
    remote_v1: Option<&'a r1::CRemotePlayer>,
    remote_v3: Option<&'a r3::CRemotePlayer>,
}

impl<'a> RemotePlayer<'a> {
    pub fn matrix(&self) -> Option<RwMatrix> {
        if let Some(remote) = self.remote_v1.as_ref() {
            return remote.matrix();
        }

        if let Some(remote) = self.remote_v3.as_ref() {
            return remote.matrix();
        }

        None
    }

    pub fn position(&self) -> CVector {
        if let Some(remote) = self.remote_v1.as_ref() {
            return remote.ped_position();
        }

        if let Some(remote) = self.remote_v3.as_ref() {
            return remote.ped_position();
        }

        CVector::zero()
    }

    pub fn velocity(&self) -> CVector {
        if let Some(remote) = self.remote_v1.as_ref() {
            return remote.velocity();
        }

        if let Some(remote) = self.remote_v3.as_ref() {
            return remote.velocity();
        }

        CVector::zero()
    }

    pub fn head_direction(&self) -> CVector {
        if let Some(remote) = self.remote_v1.as_ref() {
            return remote.head_direction();
        }

        if let Some(remote) = self.remote_v3.as_ref() {
            return remote.head_direction();
        }

        CVector::zero()
    }

    pub fn id(&self) -> u16 {
        if let Some(remote) = self.remote_v1.as_ref() {
            return remote.id();
        }

        if let Some(remote) = self.remote_v3.as_ref() {
            return remote.id();
        }

        u16::max_value()
    }
}

pub fn local_player<'a>() -> Option<LocalPlayer<'a>> {
    match version() {
        Version::V037 => Some(LocalPlayer {
            player_v1: r1::local_player(),
            player_v3: None,
        }),
        Version::V037R3 => Some(LocalPlayer {
            player_v1: None,
            player_v3: r3::local_player(),
        }),
        _ => None,
    }
}

pub fn find_player<'a>(id: i32) -> Option<Player<'a>> {
    match version() {
        Version::V037 => r1::find_player(id).map(|player| Player {
            player_v1: Some(player),
            player_v3: None,
        }),
        Version::V037R3 => r3::find_player(id).map(|player| Player {
            player_v1: None,
            player_v3: Some(player),
        }),
        _ => None,
    }
}

pub fn players<'a>() -> Option<PlayersIterator<'a>> {
    match version() {
        Version::V037 => Some(PlayersIterator {
            players_v1: r1::player_pool().map(|pool| unsafe {
                std::ptr::read_unaligned(std::ptr::addr_of!(pool.m_pObject))
            }),
            players_v3: None,
            index: 0,
            _marker: std::marker::PhantomData,
        }),

        Version::V037R3 => Some(PlayersIterator {
            players_v3: r3::player_pool().map(|pool| unsafe {
                std::ptr::read_unaligned(std::ptr::addr_of!(pool.m_pObject))
            }),
            players_v1: None,
            index: 0,
            _marker: std::marker::PhantomData,
        }),

        _ => None,
    }
}

pub struct PlayersIterator<'a> {
    players_v1: Option<[*mut r1::CPlayerInfo; 1004]>,
    players_v3: Option<[*mut r3::CPlayerInfo; 1004]>,
    index: usize,
    _marker: std::marker::PhantomData<&'a mut ()>,
}

impl<'a> Iterator for PlayersIterator<'a> {
    type Item = Player<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(players) = self.players_v1.as_ref() {
            while self.index >= 0 && self.index < 1000 {
                if let Some(player) = players.get(self.index).filter(|player| !player.is_null()) {
                    self.index += 1;
                    return Some(Player::new_v1(unsafe { &mut **player }));
                }

                self.index += 1;
            }
        }

        if let Some(players) = self.players_v3.as_ref() {
            while self.index >= 0 && self.index < 1000 {
                if let Some(player) = players.get(self.index).filter(|player| !player.is_null()) {
                    self.index += 1;
                    return Some(Player::new_v3(unsafe { &mut **player }));
                }

                self.index += 1;
            }
        }

        None
    }
}
