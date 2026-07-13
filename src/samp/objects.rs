use super::{v037r3 as r3, v037 as r1};
use super::version::{Version, version};

use crate::gta::object::CObject;
use crate::gta::matrix::CVector;
use crate::gta::rw::{self, rpworld::*, rwplcore::*};

use std::ffi::c_void;

pub struct Object<'a> {
    object_v1: Option<&'a r1::CObject>,
    object_v3: Option<&'a r3::CObject>,
}

impl<'a> Object<'a> {
    fn new_v1(object: &'a r1::CObject) -> Object<'a> {
        Object {
            object_v1: Some(object),
            object_v3: None,
        }
    }

    fn new_v3(object: &'a r3::CObject) -> Object<'a> {
        Object {
            object_v3: Some(object),
            object_v1: None,
        }
    }

    pub fn entity(&self) -> Option<&'a mut CObject> {
        let v1 = self.object_v1.map(|obj| obj._base.m_pGameEntity as *mut CObject);
        let v3 = self.object_v3.map(|obj| obj._base.m_pGameEntity as *mut CObject);

        v1.or(v3)
            .filter(|ptr| !ptr.is_null())
            .map(|ptr| unsafe { &mut *ptr })
    }

    pub fn position(&self) -> CVector {
        self.entity()
            .and_then(|entity| entity.physical().entity().placeable().matrix())
            .map(|matrix| matrix.pos)
            .unwrap_or_else(|| CVector::zero())
    }

    pub fn velocity(&self) -> CVector {
        self.entity()
            .map(|entity| entity.physical().m_vecMoveSpeed)
            .unwrap_or_else(|| CVector::zero())
    }

    pub fn heading(&self) -> CVector {
        self.entity()
            .and_then(|entity| entity.physical().entity().placeable().matrix())
            .map(|matrix| matrix.at)
            .unwrap_or_else(|| CVector::zero())
    }

    /// Returns the RenderWare atomics that actually draw this object.
    ///
    /// SA:MP R3 applies `SetObjectMaterial` immediately before invoking an
    /// atomic's render callback, so consumers that need the final material
    /// state must observe this path instead of only hooking `CEntity::Render`.
    pub fn render_atomics(&self) -> Vec<*mut RpAtomic> {
        let Some(entity) = self.entity() else {
            return Vec::new();
        };

        let rwobject = entity._base._base.rw_entity as *mut RwObject;

        if rwobject.is_null() {
            return Vec::new();
        }

        let mut atomics = Vec::new();

        unsafe {
            if (*rwobject).obj_type == rpCLUMP {
                rw::rpclump_for_all_atomics(
                    rwobject as *mut _,
                    Some(collect_render_atomic),
                    &mut atomics as *mut _ as *mut c_void,
                );
            } else {
                atomics.push(rwobject as *mut _);
            }
        }

        atomics
    }

    pub fn get(object_id: i32) -> Option<Object<'a>> {
        match version() {
            Version::V037 => r1::find_object(object_id).map(|obj| Object::new_v1(obj)),
            Version::V037R3 => r3::find_object(object_id).map(|obj| Object::new_v3(obj)),
            _ => None,
        }
    }
}

unsafe extern "C" fn collect_render_atomic(
    atomic: *mut RpAtomic,
    data: *mut c_void,
) -> *mut RpAtomic {
    unsafe {
        (&mut *(data as *mut Vec<*mut RpAtomic>)).push(atomic);
    }

    atomic
}
