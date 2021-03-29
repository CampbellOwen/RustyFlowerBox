fn main() {
    windows::build!(
        windows::win32::direct3d11::*,
        windows::win32::dxgi::*,
        windows::win32::system_services::{BOOL, GetModuleHandleA, HINSTANCE, LRESULT, PSTR, PWSTR},
        windows::win32::menus_and_resources::HMENU,
        windows::win32::windows_and_messaging::{
          CreateWindowExA,
          HWND,
          RegisterClassA,
          WINDOWS_EX_STYLE,
          WINDOWS_STYLE,
          WNDCLASSA,
          WNDCLASS_STYLES,
          WPARAM,
          LPARAM,
          WM_DESTROY,
          WM_QUIT,
          PostQuitMessage,
          DefWindowProcA,
          ShowWindow,
          SHOW_WINDOW_CMD,
          PeekMessageA,
          PeekMessage_wRemoveMsg,
          TranslateMessage,
          DispatchMessageA,
          MSG,
        },
        windows::foundation::numerics::{Vector2, Vector3},
        windows::win32::com::HRESULT,
        windows::win32::direct3d_hlsl::*
    );
}
