#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CSimpleTransform {
    pub m_vPosn: CVector,
    pub m_fHeading: f32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CPlaceable {
    pub vtable_: *const (),
    pub m_placement: CSimpleTransform,
    pub m_matrix: *mut RwMatrix,
}

impl CPlaceable {
    pub fn matrix(&self) -> Option<&RwMatrix> {
        if self.m_matrix.is_null() {
            None
        } else {
            Some(unsafe { &*self.m_matrix })
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct CQuaternion {
    pub imag: CVector,
    pub real: f32,
}

impl CVector {
    pub fn new(x: f32, y: f32, z: f32) -> CVector {
        CVector { x, y, z }
    }

    pub fn zero() -> CVector {
        CVector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn with_offset_z(&self, z: f32) -> CVector {
        CVector {
            x: self.x,
            y: self.y,
            z: self.z + z,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RwMatrix {
    pub right: CVector,
    pub flags: std::os::raw::c_ulong,
    pub up: CVector,
    pub pad_u: f32,
    pub at: CVector,
    pub pad_a: f32,
    pub pos: CVector,
    pub pad_p: f32,
}

#[derive(Debug, Clone)]
pub struct Place {
    pub matrix: RwMatrix,
    pub position: CVector,
    pub velocity: CVector,
    pub direction: CVector,
}

#[repr(C)]
#[derive(Debug)]
pub struct CVector2D {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct CRect {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl CRect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> CRect {
        CRect {
            left,
            top,
            right,
            bottom,
        }
    }
}
