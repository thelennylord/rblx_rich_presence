use std::{
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    mem::size_of,
    ptr::null_mut,
    sync::{
        mpsc::{
            Sender, channel
        }
    }
};
use winapi::{
    ctypes::{
        c_ulong, c_ushort
    },
    shared::{
        minwindef::{
            LRESULT, UINT, WPARAM, LPARAM, HINSTANCE, DWORD
        },
        windef::{
            HWND, POINT, HBRUSH, HMENU, HICON
        },
        guiddef::GUID
    },
    um::{
        winuser,
        wincon::GetConsoleWindow,
        shellapi::{
            NOTIFYICONDATAW, NIF_MESSAGE, Shell_NotifyIconW, NIM_ADD, NIF_ICON, NIM_DELETE, NIF_TIP
        },
        winuser::{
            WM_APP, WM_LBUTTONUP, WM_RBUTTONUP, WM_GETICON, GetCursorPos, CreatePopupMenu, AppendMenuW, DefWindowProcW, WNDCLASSEXW,
            MF_STRING, MF_GRAYED, GetMessageW, TPM_RETURNCMD, TPM_TOPALIGN, TPM_LEFTALIGN, MF_SEPARATOR, MF_POPUP, MF_CHECKED, MF_UNCHECKED,
            SendMessageW, ICON_SMALL2, MF_BYCOMMAND, SC_CLOSE
        }
    }
};
use crate::utils::{set_config, get_config};


struct SafeNID {
    nid: NOTIFYICONDATAW,
}

unsafe impl Send for SafeNID {}

pub fn wide_str(str: &str) -> Vec<u16> {
    OsStr::new(str)
    .encode_wide()
    .chain(Some(0).into_iter())
    .collect::<Vec<_>>()
}

unsafe extern "system" fn window_proc(hwnd: HWND, u_msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // item ids
    const TRAY_ICON: u32 = WM_APP + 1;
    const MENU_LABEL: u32 = WM_APP + 2;
    const MENU_OPTIONS: u32 = WM_APP + 3;
    const OPTIONS_USERNAME: u32 = MENU_OPTIONS + 1;
    const OPTIONS_GAME: u32 = MENU_OPTIONS + 2;
    const OPTIONS_PRESENCE: u32 = MENU_OPTIONS + 3;
    const MENU_DEBUG: u32 = WM_APP + 7;
    const MENU_EXIT: u32 = WM_APP + 8;
    
    
    match u_msg {
        TRAY_ICON => {
            if l_param as UINT == WM_LBUTTONUP || l_param as UINT == WM_RBUTTONUP {
                let mut position = POINT {
                    x: 0,
                    y: 0
                };
                
                if GetCursorPos(&mut position as *mut POINT) == 0 {
                    return 1;
                }

                let mut config = get_config().unwrap();
                
                let con = GetConsoleWindow();
                let main_popup = CreatePopupMenu();
                
                AppendMenuW(main_popup, MF_STRING | MF_GRAYED, MENU_LABEL as usize, wide_str("Roblox Rich Presence").as_ptr());
                AppendMenuW(main_popup, MF_SEPARATOR, 0, null_mut());
                
                let options_popup = CreatePopupMenu();
                
                AppendMenuW(options_popup, MF_STRING| (if config.presence.show_username {MF_CHECKED} else {MF_UNCHECKED}), OPTIONS_USERNAME as usize, wide_str("Show username").as_ptr());
                AppendMenuW(options_popup, MF_STRING| (if config.presence.show_game {MF_CHECKED} else {MF_UNCHECKED}), OPTIONS_GAME as usize, wide_str("Show game").as_ptr());
                AppendMenuW(options_popup, MF_STRING| (if config.presence.show_presence {MF_CHECKED} else {MF_UNCHECKED}), OPTIONS_PRESENCE as usize, wide_str("Show presence").as_ptr());
                AppendMenuW(main_popup, MF_POPUP, options_popup as usize, wide_str("Options...").as_ptr());
                
                AppendMenuW(main_popup, MF_STRING | (if winuser::IsWindowVisible(con) == 0 {MF_UNCHECKED} else {MF_CHECKED}), MENU_DEBUG as usize, wide_str("Debug").as_ptr());
                
                AppendMenuW(main_popup, MF_SEPARATOR, 0, null_mut());
                AppendMenuW(main_popup, MF_STRING, MENU_EXIT as usize, wide_str("Quit").as_ptr());
                
                winuser::SetForegroundWindow(hwnd);

                // load config

                let result = winuser::TrackPopupMenuEx(main_popup, (TPM_RETURNCMD | TPM_TOPALIGN | TPM_LEFTALIGN) as u32, position.x, position.y, hwnd, null_mut());
                match result as u32 {
                    OPTIONS_USERNAME => {
                        config.presence.show_username = !config.presence.show_username;
                        set_config(&config).unwrap();
                    },
                    OPTIONS_GAME => {
                        config.presence.show_game = !config.presence.show_game;
                        set_config(&config).unwrap();
                    },
                    OPTIONS_PRESENCE => {
                        config.presence.show_presence = !config.presence.show_presence;
                        set_config(&config).unwrap();
                    },
                    MENU_DEBUG => {
                        if winuser::IsWindowVisible(con) == 0 {
                            winuser::ShowWindow(con, 1);
                        } else {
                            winuser::ShowWindow(con, 0);
                        }
                    },
                    MENU_EXIT => {
                        let mut nid = NOTIFYICONDATAW {
                            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
                            hWnd: hwnd,
                            uID: 0x1 as UINT,
                            uFlags: NIF_MESSAGE | NIF_ICON,
                            uCallbackMessage: WM_APP + 1 as UINT,
                            hIcon: winuser::LoadIconW(null_mut(), winuser::IDI_APPLICATION),
                            szTip: [0 as u16; 128],
                            dwState: 0 as DWORD,
                            dwStateMask: 0 as DWORD,
                            szInfo: [0 as u16; 256],
                            u: Default::default(),
                            szInfoTitle: [0 as u16; 64],
                            dwInfoFlags: 0 as UINT,
                            guidItem: GUID {
                                Data1: 0 as c_ulong,
                                Data2: 0 as c_ushort,
                                Data3: 0 as c_ushort,
                                Data4: [0; 8],
                            },
                            hBalloonIcon: 0 as HICON,
                        };
                        Shell_NotifyIconW(NIM_DELETE, &mut nid as *mut NOTIFYICONDATAW);
                        winuser::PostQuitMessage(0);
                        std::process::exit(0);
                    }
                    _ => {
                        return 1;
                    }
                }
                return 1;
            }
        },
        _ => {
            return DefWindowProcW(hwnd, u_msg, w_param, l_param);
        }
    }
    
    return 0;
}

pub unsafe fn start(tx: Sender<Sender<bool>>) {
    let con = GetConsoleWindow();
    let wcex = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0 as HINSTANCE,
        hIcon: null_mut(), //winuser::LoadIconW(0 as HINSTANCE, winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(0 as HINSTANCE, winuser::IDI_APPLICATION),
        hbrBackground: 16 as HBRUSH,
        lpszMenuName: null_mut(),
        lpszClassName: wide_str("rblx_rp_class").as_ptr(),
        hIconSm: null_mut()
    };

    if winuser::RegisterClassExW(&wcex) == 0 {
        panic!("Error creating class");
    };

    let hwnd = winuser::CreateWindowExW(
        0,
        wide_str("rblx_rp_class").as_ptr(),
        wide_str("Roblox Rich Presence").as_ptr(),
        winuser::WS_OVERLAPPEDWINDOW,
        winuser::CW_USEDEFAULT,
        0,
        winuser::CW_USEDEFAULT,
        0,
        0 as HWND,
        0 as HMENU,
        0 as HINSTANCE,
        null_mut(),
    );

    if hwnd == null_mut() {
        panic!("hwnd is a null pointer");
    }
    
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
        hWnd: hwnd,
        uID: 0x1 as UINT,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_APP + 1 as UINT,
        hIcon: winuser::LoadIconW(null_mut(), winuser::IDI_APPLICATION),
        szTip: [0 as u16; 128],
        dwState: 0 as DWORD,
        dwStateMask: 0 as DWORD,
        szInfo: [0 as u16; 256],
        u: Default::default(),
        szInfoTitle: [0 as u16; 64],
        dwInfoFlags: 0 as UINT,
        guidItem: GUID {
            Data1: 0 as c_ulong,
            Data2: 0 as c_ushort,
            Data3: 0 as c_ushort,
            Data4: [0; 8],
        },
        hBalloonIcon: 0 as HICON,
    };
    
    let hicon = SendMessageW(con, WM_GETICON, ICON_SMALL2 as usize, 0 as isize);
    if hicon != 0 {
        nid.hIcon = hicon as HICON;
    }

    let tooltip = b"Roblox Rich Presence".clone();
    for i in 0..tooltip.len() {
        nid.szTip[i] = tooltip[i] as u16;
    }

    if Shell_NotifyIconW(NIM_ADD, &mut nid as *mut NOTIFYICONDATAW) == 0 {
        panic!("Error while creating icon in tray");
    }

    let mut msg = winuser::MSG {
        hwnd: 0 as HWND,
        message: 0 as UINT,
        wParam: 0 as WPARAM,
        lParam: 0 as LPARAM,
        time: 0 as DWORD,
        pt: POINT { x: 0, y: 0 },
    };

    let (send, rx) = channel::<bool>();
    let send1 = send.clone();
    tx.send(send1).unwrap();

    let mut safe_nid = SafeNID {
        nid
    };

    let hmenu = winuser::GetSystemMenu(con, 0);
    winuser::DeleteMenu(hmenu, SC_CLOSE as u32, MF_BYCOMMAND);
    winuser::ShowWindow(con, 0);

    std::thread::spawn(move || loop {
        if rx.try_recv().is_ok() {
            Shell_NotifyIconW(NIM_DELETE, &mut safe_nid.nid as *mut NOTIFYICONDATAW);
            winuser::PostQuitMessage(0);
            tx.send(send.clone()).unwrap();
            break;
        };
        std::thread::sleep(std::time::Duration::from_secs(1))
    });

    while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
        
        if msg.message == winuser::WM_QUIT {
            break;
        }
        
        winuser::TranslateMessage(&mut msg);
        winuser::DispatchMessageW(&mut msg);
    }
        
}