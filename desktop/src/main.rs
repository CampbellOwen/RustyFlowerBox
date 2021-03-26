use std::{array, error, mem::swap};

use bindings::windows::foundation::numerics::{Vector2, Vector3};
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

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;

struct DirectX11GraphicsDevice {
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    swapchain: IDXGISwapChain,
    backbuffer_rtv: ID3D11RenderTargetView,
}

impl DirectX11GraphicsDevice {
    fn new(hwnd: HWND) -> Option<DirectX11GraphicsDevice> {
        let mut device: Option<ID3D11Device> = None;
        let mut swapchain: Option<IDXGISwapChain> = None;
        let mut device_context: Option<ID3D11DeviceContext> = None;

        let swapchain_desc = DXGI_SWAP_CHAIN_DESC {
            buffer_desc: DXGI_MODE_DESC {
                width: WIDTH as u32,
                height: HEIGHT as u32,
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
            let error_code = swapchain.GetBuffer(0, &ID3D11Resource::IID, backbuffer.set_abi());
            if error_code.is_err() {
                panic!(error_code.message());
            }
            let mut backbuffer_rtv: Option<ID3D11RenderTargetView> = None;

            let error_code =
                device.CreateRenderTargetView(backbuffer, std::ptr::null(), &mut backbuffer_rtv);
            if error_code.is_err() {
                panic!(error_code.message());
            }

            let view_port = D3D11_VIEWPORT {
                top_leftx: 0.0,
                top_lefty: 0.0,
                width: WIDTH as f32,
                height: HEIGHT as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            device_context.RSSetViewports(1, &view_port);

            let depth_stencil_desc = D3D11_DEPTH_STENCIL_DESC {
                depth_enable: BOOL::from(true),
                depth_write_mask: D3D11_DEPTH_WRITE_MASK::D3D11_DEPTH_WRITE_MASK_ALL,
                depth_func: D3D11_COMPARISON_FUNC::D3D11_COMPARISON_LESS_EQUAL,
                // Stencil Text
                stencil_enable: BOOL::from(false),
                stencil_read_mask: 0xFF,
                stencil_write_mask: 0xFF,
                front_face: D3D11_DEPTH_STENCILOP_DESC {
                    stencil_fail_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_KEEP,
                    stencil_depth_fail_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_INCR,
                    stencil_pass_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_KEEP,
                    stencil_func: D3D11_COMPARISON_FUNC::D3D11_COMPARISON_ALWAYS,
                },
                back_face: D3D11_DEPTH_STENCILOP_DESC {
                    stencil_fail_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_KEEP,
                    stencil_depth_fail_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_DECR,
                    stencil_pass_op: D3D11_STENCIL_OP::D3D11_STENCIL_OP_KEEP,
                    stencil_func: D3D11_COMPARISON_FUNC::D3D11_COMPARISON_ALWAYS,
                },
            };

            let mut depth_stencil_state: Option<ID3D11DepthStencilState> = None;
            let error_code =
                device.CreateDepthStencilState(&depth_stencil_desc, &mut depth_stencil_state);
            if error_code.is_err() {
                panic!(error_code.message());
            }

            device_context.OMSetDepthStencilState(&depth_stencil_state, 1);

            let depth_texture_desc = D3D11_TEXTURE2D_DESC {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                mip_levels: 1,
                array_size: 1,
                format: DXGI_FORMAT::DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
                sample_desc: DXGI_SAMPLE_DESC {
                    count: 1,
                    quality: 0,
                },
                usage: D3D11_USAGE::D3D11_USAGE_DEFAULT,
                bind_flags: D3D11_BIND_FLAG::D3D11_BIND_DEPTH_STENCIL.0 as u32,
                cpu_access_flags: 0,
                misc_flags: 0,
            };

            let mut depth_stencil_texture: Option<ID3D11Texture2D> = None;
            let error_code = device.CreateTexture2D(
                &depth_texture_desc,
                std::ptr::null(),
                &mut depth_stencil_texture,
            );
            if error_code.is_err() {
                panic!(error_code.message());
            }

            let depth_stencil_texture = depth_stencil_texture?;

            let depth_stencil_view_desc = D3D11_DEPTH_STENCIL_VIEW_DESC {
                format: depth_texture_desc.format,
                view_dimension: D3D11_DSV_DIMENSION::D3D11_DSV_DIMENSION_TEXTURE2DMS,
                flags: 0,
                anonymous: D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
                    texture2d: D3D11_TEX2D_DSV { mip_slice: 0 },
                },
            };

            let mut depth_stencil_view: Option<ID3D11DepthStencilView> = None;
            let error_code = device.CreateDepthStencilView(
                &depth_stencil_texture,
                &depth_stencil_view_desc,
                &mut depth_stencil_view,
            );
            if error_code.is_err() {
                panic!(error_code.message());
            }

            device_context.OMSetRenderTargets(1, &mut backbuffer_rtv, &depth_stencil_view);

            let backbuffer_rtv = backbuffer_rtv?;

            Some(DirectX11GraphicsDevice {
                device,
                device_context,
                swapchain,
                backbuffer_rtv,
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
            WIDTH,
            HEIGHT,
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

            //let _ = graphics_device.swapchain.Present(1, 0);
        }
    }
}
