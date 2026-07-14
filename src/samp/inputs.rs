use super::{handle, BOOL};
use super::version::{Version, version};

use std::ffi::c_void;

pub type CMDPROC = Option<unsafe extern "C" fn(arg1: *const std::os::raw::c_char)>;

const MAX_CLIENT_CMDS: usize = 144;
const MAX_CMD_LENGTH: usize = 32;

#[repr(C, packed)]
pub struct Input {
    pub m_pDevice: *mut (),  // IDirect3DDevice9
    pub m_pGameUi: *mut (),  // CDXUTDialog
    pub m_pEditbox: *mut (), // CDXUTEditBox
    pub m_pCommandProc: [CMDPROC; MAX_CLIENT_CMDS],
    pub m_szCommandName: [[std::os::raw::c_char; MAX_CMD_LENGTH + 1]; MAX_CLIENT_CMDS],
    pub m_nCommandCount: std::os::raw::c_int,
    pub m_bEnabled: BOOL,
    pub m_szInput: [std::os::raw::c_char; 129],
    pub m_szRecallBufffer: [[std::os::raw::c_char; 129]; 10],
    pub m_szCurrentBuffer: [std::os::raw::c_char; 129],
    pub m_nCurrentRecall: std::os::raw::c_int,
    pub m_nTotalRecall: std::os::raw::c_int,
    pub m_pDefaultCommand: CMDPROC,
}

impl Input {
    pub fn get<'a>() -> Option<&'a mut Input> {
        let input_addr = match version() {
            Version::V03DL => super::v03dl::CINPUT,
            Version::V037 => super::v037::CINPUT,
            Version::V037R3 => super::v037r3::CINPUT,
            _ => return None,
        };

        let ptr = unsafe { (handle().add(input_addr) as *mut *mut Input).read() };

        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    }

    pub fn is_active() -> bool {
        Self::get()
            .map(|input| input.m_bEnabled == 1)
            .unwrap_or(false)
    }
}

#[repr(C)]
pub struct DXUTControl {
    vftable: *mut (),
    visible: bool,    // Shown/hidden flag
    mouse_over: bool, // Mouse pointer is above control
    has_focus: bool,  // Control has input focus
    is_default: bool, // Is the default control
}

#[repr(C, packed)]
pub struct Dialog {
    pub m_pDevice: *mut (), // IDirect3DDevice9
    pub m_position: [std::os::raw::c_ulong; 2],
    pub m_size: [std::os::raw::c_ulong; 2],
    pub m_buttonOffset: [std::os::raw::c_ulong; 2],
    pub m_pDialog: *mut (),           // CDXUTDialog
    pub m_pListbox: *mut (),          // CDXUTListBox
    pub m_pEditbox: *mut DXUTControl, // CDXUTIMEEditBox
    pub m_bIsActive: BOOL,
    pub m_nType: std::os::raw::c_int,
    pub m_nId: std::os::raw::c_int,
    pub m_szText: *mut std::os::raw::c_char,
    pub m_textSize: [std::os::raw::c_int; 2],
    pub m_szCaption: [std::os::raw::c_char; 65],
    pub m_bServerside: BOOL,
}

impl Dialog {
    pub fn get<'a>() -> Option<&'a Dialog> {
        let dialog_addr = match version() {
            Version::V03DL => super::v03dl::CDIALOG,
            Version::V037 => super::v037::CDIALOG,
            Version::V037R3 => super::v037r3::CDIALOG,
            _ => return None,
        };

        let samp_base = handle();

        unsafe {
            let ptr = samp_base.add(dialog_addr) as *mut *mut Dialog;
            let cdialog = ptr.read();

            if !cdialog.is_null() {
                Some(&*cdialog)
            } else {
                None
            }
        }
    }

    pub fn is_input_focused() -> bool {
        Dialog::get()
            .filter(|cdialog| !cdialog.m_pEditbox.is_null())
            .map(|cdialog| {
                let has_focus = unsafe { cdialog.m_pEditbox.read().has_focus };
                let is_active = if cdialog.m_bIsActive == 1 {
                    true
                } else {
                    false
                };
                is_active && has_focus
            })
            .unwrap_or(false)
    }
}

pub fn show_cursor(show: bool) {
    if Input::is_active() {
        return;
    }

    unsafe {
        let cgame_addr = match version() {
            Version::V03DL => super::v03dl::CGAME,
            Version::V037 => super::v037::CGAME,
            Version::V037R3 => super::v037r3::CGAME,
            _ => return,
        };

        let setcursor_addr = match version() {
            Version::V03DL => super::v03dl::CGAME_SETCURSORMODE,
            Version::V037 => super::v037::CGAME_SETCURSORMODE,
            Version::V037R3 => super::v037r3::CGAME_SETCURSORMODE,
            _ => return,
        };

        let process_addr = match version() {
            Version::V03DL => super::v03dl::CGAME_PROCESSINPUTENABLING,
            Version::V037 => super::v037::CGAME_PROCESSINPUTENABLING,
            Version::V037R3 => super::v037r3::CGAME_PROCESSINPUTENABLING,
            _ => return,
        };

        let samp_base = handle();
        let ptr = samp_base.add(cgame_addr) as *mut *mut c_void;
        let cgame = ptr.read();

        if cgame.is_null() {
            return;
        }

        let set_cursor_mode: extern "thiscall" fn(*mut c_void, i32, BOOL) =
            std::mem::transmute(samp_base.add(setcursor_addr));
        let process_input_enabling: extern "thiscall" fn(*mut c_void) =
            std::mem::transmute(samp_base.add(process_addr));

        let mode = if show { 2 } else { 0 };
        let force_hide = if show { 0 } else { 1 };

        set_cursor_mode(cgame, mode, force_hide);

        if !show {
            process_input_enabling(cgame);
        }
    }
}
