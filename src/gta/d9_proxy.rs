use winapi::ctypes::*;
use winapi::shared::d3d9::*;
use winapi::shared::d3d9caps::*;
use winapi::shared::d3d9types::*;
use winapi::shared::guiddef::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::{HMONITOR, HWND, POINT, RECT};
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
use winapi::um::wingdi::{PALETTEENTRY, RGNDATA};
use winapi::um::winnt::{HANDLE, HRESULT, VOID};

use retour::GenericDetour;

type CreateDevice = extern "stdcall" fn(u32) -> *mut IDirect3D9;

static mut DEVICE_HOOK: Option<GenericDetour<CreateDevice>> = None;
static mut ON_CREATE: Option<OnCreate> = None;
static mut ON_RENDER: Option<OnRender> = None;
static mut ON_RESET: Option<OnReset> = None;
static mut ON_DESTROY: Option<OnDestroy> = None;

pub type OnCreate = fn();
pub type OnRender = fn(&mut IDirect3DDevice9);
pub type OnReset = fn(&mut IDirect3DDevice9, u8);
pub type OnDestroy = fn(&mut IDirect3DDevice9);

#[repr(C)]
struct Direct3D {
    vftable: *mut IDirect3D9Vtbl,
    origin: *mut IDirect3D9,
    on_create: OnCreate,
}

impl Direct3D {
    fn restore<'a>(raw: *mut IDirect3D9) -> &'a mut Direct3D {
        unsafe { &mut *(raw as *mut Direct3D) }
    }

    fn origin<'a>(raw: *mut IDirect3D9) -> &'a mut IDirect3D9 {
        let origin = Direct3D::restore(raw).origin;
        unsafe { &mut *origin }
    }
}

pub fn set_proxy(
    on_create: OnCreate,
    on_render: OnRender,
    on_reset: OnReset,
    on_destroy: OnDestroy,
) {
    unsafe {
        let func: CreateDevice = std::mem::transmute(0x807C2B);
        log::trace!("finding function and hooking ... (d9_proxy::set_proxy)");
        let mut hook = GenericDetour::new(func, hook_direct3d_create9).unwrap();
        log::trace!("done enable it ... (d9_proxy::set_proxy)");
        hook.enable().unwrap();

        ON_CREATE = Some(on_create);
        ON_RENDER = Some(on_render);
        ON_RESET = Some(on_reset);
        ON_DESTROY = Some(on_destroy);

        DEVICE_HOOK = Some(hook);
        log::trace!("done ... (d9_proxy::set_proxy)");
    }
}

pub fn leak<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

extern "stdcall" fn hook_direct3d_create9(sdk: u32) -> *mut IDirect3D9 {
    unsafe {
        if let Some(hook) = DEVICE_HOOK.as_mut() {
            log::trace!("hook_direct3d_create9({})", sdk);

            let origin = hook.call(sdk);

            log::trace!("origin ptr: {:#?}", origin);

            let table = create_vftable();

            let direct = Direct3D {
                origin,
                vftable: leak(table),
                on_create: ON_CREATE.unwrap(),
            };

            return leak(direct) as *mut _;
        }
    }

    std::ptr::null_mut()
}

fn delete(obj: *mut Direct3D) {
    unsafe {
        log::trace!("delete Direct3D object");

        let device = Box::from_raw(obj);
        let vftable = Box::from_raw(device.vftable);
        drop(vftable);
        drop(device);
    }
}

fn create_vftable() -> IDirect3D9Vtbl {
    let parent = IUnknownVtbl {
        QueryInterface,
        AddRef,
        Release,
    };

    IDirect3D9Vtbl {
        parent,
        RegisterSoftwareDevice,
        GetAdapterCount,
        GetAdapterIdentifier,
        GetAdapterModeCount,
        EnumAdapterModes,
        GetAdapterDisplayMode,
        CheckDeviceType,
        CheckDeviceFormat,
        CheckDeviceMultiSampleType,
        CheckDepthStencilMatch,
        CheckDeviceFormatConversion,
        GetDeviceCaps,
        GetAdapterMonitor,
        CreateDevice,
    }
}

// IUNKOWN ==================

unsafe extern "system" fn QueryInterface(
    this: *mut IUnknown,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let origin = Direct3D::origin(this as *mut _);
    let result = origin.QueryInterface(riid, ppvObject);

    if result == 0 {
        *ppvObject = this as *mut c_void;
    }

    result
}

unsafe extern "system" fn AddRef(this: *mut IUnknown) -> ULONG {
    Direct3D::origin(this as *mut _).AddRef()
}

unsafe extern "system" fn Release(this: *mut IUnknown) -> ULONG {
    let origin = Direct3D::origin(this as *mut _);
    origin.AddRef();
    let result = origin.Release();

    if result == 1 {
        delete(this as *mut Direct3D);
    }

    return origin.Release();
}

// IDIRECT3D9 =======================================

unsafe extern "system" fn RegisterSoftwareDevice(
    this: *mut IDirect3D9,
    pInitializeFunction: *mut VOID,
) -> HRESULT {
    Direct3D::origin(this).RegisterSoftwareDevice(pInitializeFunction)
}

unsafe extern "system" fn GetAdapterCount(this: *mut IDirect3D9) -> UINT {
    Direct3D::origin(this).GetAdapterCount()
}

unsafe extern "system" fn GetAdapterIdentifier(
    this: *mut IDirect3D9,
    Adapter: UINT,
    Flags: DWORD,
    pIdentifier: *mut D3DADAPTER_IDENTIFIER9,
) -> HRESULT {
    Direct3D::origin(this).GetAdapterIdentifier(Adapter, Flags, pIdentifier)
}

unsafe extern "system" fn GetAdapterModeCount(
    this: *mut IDirect3D9,
    Adapter: UINT,
    Format: D3DFORMAT,
) -> UINT {
    Direct3D::origin(this).GetAdapterModeCount(Adapter, Format)
}

unsafe extern "system" fn EnumAdapterModes(
    this: *mut IDirect3D9,
    Adapter: UINT,
    Format: D3DFORMAT,
    Mode: UINT,
    pMode: *mut D3DDISPLAYMODE,
) -> HRESULT {
    Direct3D::origin(this).EnumAdapterModes(Adapter, Format, Mode, pMode)
}

unsafe extern "system" fn GetAdapterDisplayMode(
    this: *mut IDirect3D9,
    Adapter: UINT,
    pMode: *mut D3DDISPLAYMODE,
) -> HRESULT {
    Direct3D::origin(this).GetAdapterDisplayMode(Adapter, pMode)
}

unsafe extern "system" fn CheckDeviceType(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DevType: D3DDEVTYPE,
    AdapterFormat: D3DFORMAT,
    BackBufferFormat: D3DFORMAT,
    bWindowed: BOOL,
) -> HRESULT {
    Direct3D::origin(this).CheckDeviceType(
        Adapter,
        DevType,
        AdapterFormat,
        BackBufferFormat,
        bWindowed,
    )
}

unsafe extern "system" fn CheckDeviceFormat(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    AdapterFormat: D3DFORMAT,
    Usage: DWORD,
    RType: D3DRESOURCETYPE,
    CheckFormat: D3DFORMAT,
) -> HRESULT {
    Direct3D::origin(this).CheckDeviceFormat(
        Adapter,
        DeviceType,
        AdapterFormat,
        Usage,
        RType,
        CheckFormat,
    )
}

unsafe extern "system" fn CheckDeviceMultiSampleType(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    SurfaceFormat: D3DFORMAT,
    Windowed: BOOL,
    MultiSampleType: D3DMULTISAMPLE_TYPE,
    pQualityLevels: *mut DWORD,
) -> HRESULT {
    Direct3D::origin(this).CheckDeviceMultiSampleType(
        Adapter,
        DeviceType,
        SurfaceFormat,
        Windowed,
        MultiSampleType,
        pQualityLevels,
    )
}

unsafe extern "system" fn CheckDepthStencilMatch(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    AdapterFormat: D3DFORMAT,
    RenderTargetFormat: D3DFORMAT,
    DepthStencilFormat: D3DFORMAT,
) -> HRESULT {
    Direct3D::origin(this).CheckDepthStencilMatch(
        Adapter,
        DeviceType,
        AdapterFormat,
        RenderTargetFormat,
        DepthStencilFormat,
    )
}

unsafe extern "system" fn CheckDeviceFormatConversion(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    SourceFormat: D3DFORMAT,
    TargetFormat: D3DFORMAT,
) -> HRESULT {
    Direct3D::origin(this).CheckDeviceFormatConversion(
        Adapter,
        DeviceType,
        SourceFormat,
        TargetFormat,
    )
}

unsafe extern "system" fn GetDeviceCaps(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    pCaps: *mut D3DCAPS9,
) -> HRESULT {
    Direct3D::origin(this).GetDeviceCaps(Adapter, DeviceType, pCaps)
}

unsafe extern "system" fn GetAdapterMonitor(this: *mut IDirect3D9, Adapter: UINT) -> HMONITOR {
    Direct3D::origin(this).GetAdapterMonitor(Adapter)
}

unsafe extern "system" fn CreateDevice(
    this: *mut IDirect3D9,
    Adapter: UINT,
    DeviceType: D3DDEVTYPE,
    hFocusWindow: HWND,
    BehaviorFlags: DWORD,
    pPresentationParameters: *mut D3DPRESENT_PARAMETERS,
    ppReturnedDeviceInterface: *mut *mut IDirect3DDevice9,
) -> HRESULT {
    log::trace!("Direct3D::CreateDevice");

    let result = Direct3D::origin(this).CreateDevice(
        Adapter,
        DeviceType,
        hFocusWindow,
        BehaviorFlags,
        pPresentationParameters,
        ppReturnedDeviceInterface,
    );

    if result == 0 {
        let proxy = Direct3D::restore(this);

        log::trace!("super::device_proxy::set_proxy()");

        let device = super::device_proxy::set_proxy(
            *ppReturnedDeviceInterface,
            ON_RENDER,
            ON_RESET,
            ON_DESTROY,
        );

        *ppReturnedDeviceInterface = device;

        log::trace!("(proxy.on_create)()");

        (proxy.on_create)();
    }

    return result;
}
