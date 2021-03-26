use std::{error, mem::swap};

use bindings::windows::foundation::numerics::{Vector2, Vector3};
use bindings::windows::win32::com::HRESULT;
use bindings::windows::win32::direct3d11::*;
use bindings::windows::win32::dxgi::*;
use bindings::windows::win32::menus_and_resources::HMENU;
use bindings::windows::win32::system_services::{GetModuleHandleA, BOOL, HINSTANCE, LRESULT, PSTR};
use bindings::windows::win32::windows_and_messaging::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, PeekMessageA, PeekMessage_wRemoveMsg,
    PostQuitMessage, RegisterClassA, ShowWindow, TranslateMessage, HWND, LPARAM, MSG,
    SHOW_WINDOW_CMD, WINDOWS_EX_STYLE, WINDOWS_STYLE, WM_DESTROY, WM_QUIT, WNDCLASSA,
    WNDCLASS_STYLES, WPARAM,
};

use flower_box::GraphicsDevice;
use windows::{Abi, Interface};

struct DirectX11GraphicsDevice {
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    swapchain: IDXGISwapChain,
}

impl DirectX11GraphicsDevice {
    fn new(hwnd: HWND) -> Option<DirectX11GraphicsDevice> {
        let mut device: Option<ID3D11Device> = None;
        let mut swapchain: Option<IDXGISwapChain> = None;
        let mut device_context: Option<ID3D11DeviceContext> = None;

        let swapchain_desc = DXGI_SWAP_CHAIN_DESC {
            buffer_desc: DXGI_MODE_DESC {
                width: 640,
                height: 480,
                format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            sample_desc: DXGI_SAMPLE_DESC {
                count: 1,
                ..Default::default()
            },
            buffer_usage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            buffer_count: 2,
            output_window: hwnd,
            swap_effect: DXGI_SWAP_EFFECT::DXGI_SWAP_EFFECT_DISCARD,
            flags: 0,
            windowed: BOOL::from(true),
        };

        unsafe {
            let error_code = D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_HARDWARE,
                0,
                D3D11_CREATE_DEVICE_FLAG::D3D11_CREATE_DEVICE_DEBUG,
                std::ptr::null_mut(),
                0,
                D3D11_SDK_VERSION,
                &swapchain_desc,
                &mut swapchain,
                &mut device,
                std::ptr::null_mut(),
                &mut device_context,
            );
            if error_code.is_err() {
                panic!(error_code.message())
            }

            let device = device?;
            let device_context = device_context?;
            let swapchain = swapchain?;

            let mut backbuffer: Option<ID3D11Resource> = None;
            let error_code = swapchain.GetBuffer(0, &IDXGISurface::IID, backbuffer.set_abi());
            if error_code.is_err() {
                panic!(error_code.message());
            }
            let mut render_target_view: Option<ID3D11RenderTargetView> = None;

            let error_code = device.CreateRenderTargetView(
                backbuffer,
                std::ptr::null(),
                &mut render_target_view,
            );
            if error_code.is_err() {
                println!("{}", error_code.message());
                panic!(error_code.message());
            }

            let render_target_view = render_target_view?;

            Some(DirectX11GraphicsDevice {
                device,
                device_context,
                swapchain,
            })
        }
    }
}

impl GraphicsDevice for DirectX11GraphicsDevice {
    fn set_vertex_buffer() {}
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
        }

        DefWindowProcA(hwnd, msg, w_param, l_param)
    }
}

fn create_window() -> Option<HWND> {
    unsafe {
        let instance = HINSTANCE(GetModuleHandleA(PSTR::default()));
        debug_assert!(instance.0 != 0);

        let window_class = WNDCLASSA {
            h_instance: HINSTANCE(0),
            lpsz_class_name: PSTR(b"WindowClass\0".as_ptr() as _),
            style: WNDCLASS_STYLES::CS_HREDRAW | WNDCLASS_STYLES::CS_VREDRAW,
            lpfn_wnd_proc: Some(window_proc),
            ..Default::default()
        };

        let atom = RegisterClassA(&window_class);
        debug_assert!(atom != 0);

        let hwnd_window = CreateWindowExA(
            WINDOWS_EX_STYLE(0),
            "WindowClass",
            "FlowerBox",
            WINDOWS_STYLE::WS_OVERLAPPEDWINDOW,
            0,
            0,
            640,
            480,
            HWND(0),
            HMENU(0),
            HINSTANCE(0),
            std::ptr::null_mut(),
        );
        debug_assert!(hwnd_window != (HWND(0)), "failed to open the window");

        ShowWindow(hwnd_window, SHOW_WINDOW_CMD::SW_SHOW);

        Some(hwnd_window)
    }
}

fn main() {
    //let mut window = Window::new();
    //window.run()

    let hwnd = create_window().unwrap();

    let graphics_device = DirectX11GraphicsDevice::new(hwnd).unwrap();

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if PeekMessageA(&mut msg, HWND(0), 0, 0, PeekMessage_wRemoveMsg::PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);

                if msg.message == WM_QUIT {
                    return ();
                }
            }

            let _ = graphics_device.swapchain.Present(1, 0);
        }
    }
}
