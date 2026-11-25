use eframe::egui;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use windows_sys::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, ERROR_SUCCESS, GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETFOCUS,
    NIM_SETVERSION, NINF_KEY, NIN_SELECT, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu,
    DestroyWindow, DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW, IMAGE_ICON,
    LoadImageW, MF_SEPARATOR, MF_STRING, MSG, PostMessageW, PostQuitMessage, RegisterClassW,
    RegisterWindowMessageW, SM_CXSMICON, SM_CYSMICON, SetForegroundWindow, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
    WM_APP, WM_CONTEXTMENU, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WNDCLASSW,
    WS_EX_TOOLWINDOW, WS_POPUP,
};

const APP_ICON_ID: usize = 1;
const TRAY_ID: u32 = 1;
const MSG_TRAY: u32 = WM_APP + 1;
const MSG_ACTIVATE: u32 = WM_APP + 2;
const MSG_QUIT_LOOP: u32 = WM_APP + 3;
const MSG_EXIT_CMD: u32 = WM_APP + 4;
const NIN_KEYSELECT: u32 = NIN_SELECT | NINF_KEY;

static TASKBAR_CREATED_MSG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

const MENU_OPEN: u32 = 1001;
const MENU_SETTINGS: u32 = 1002;
const MENU_TOGGLE_THEME: u32 = 1003;
const MENU_EXIT: u32 = 1004;

#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    OpenMain,
    OpenSettings,
    ToggleTheme,
    Exit,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn control_class_name() -> String {
    if let Ok(custom) = std::env::var("UISHOWCASE_MUTEX_NAME")
        && !custom.trim().is_empty()
    {
        return format!("{custom}_Control");
    }
    "UiShowcase_Control".to_owned()
}

pub struct SingleInstanceGuard {
    _handle: windows_sys::Win32::Foundation::HANDLE,
}

impl SingleInstanceGuard {
    pub fn try_acquire() -> Option<Self> {
        let base = std::env::var("UISHOWCASE_MUTEX_NAME")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "UiShowcase_SingleInstance".to_owned());
        let mutex_name = to_wide(&format!("Local\\{base}_Mutex"));
        let handle = unsafe { CreateMutexW(null(), 1, mutex_name.as_ptr()) };
        if handle.is_null() {
            return None;
        }
        let err = unsafe { GetLastError() };
        if err == ERROR_ALREADY_EXISTS {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(handle);
                let cls = to_wide(&control_class_name());
                let hwnd = FindWindowW(cls.as_ptr(), cls.as_ptr());
                if !hwnd.is_null() {
                    PostMessageW(hwnd, MSG_ACTIVATE, 0, 0);
                }
            }
            return None;
        }
        Some(Self { _handle: handle })
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self._handle.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self._handle);
            }
        }
    }
}

struct TrayShared {
    tx: mpsc::Sender<TrayEvent>,
    ctx: Arc<Mutex<Option<egui::Context>>>,
}

static TRAY_SHARED: Mutex<Option<Arc<TrayShared>>> = Mutex::new(None);

pub struct TrayController {
    control_hwnd: Arc<Mutex<isize>>,
    ctx_holder: Arc<Mutex<Option<egui::Context>>>,
    tray_registered: Arc<std::sync::atomic::AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl TrayController {
    pub fn start() -> (Self, mpsc::Receiver<TrayEvent>) {
        let (tx, rx) = mpsc::channel();
        let ctx_holder: Arc<Mutex<Option<egui::Context>>> = Arc::new(Mutex::new(None));
        let control_hwnd = Arc::new(Mutex::new(0isize));
        let tray_registered = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let shared = Arc::new(TrayShared {
            tx,
            ctx: ctx_holder.clone(),
        });
        *TRAY_SHARED.lock().unwrap() = Some(shared);

        let hwnd_slot = control_hwnd.clone();
        let reg_flag = tray_registered.clone();
        let class_name_str = control_class_name();

        let join = thread::spawn(move || unsafe {
            let instance = GetModuleHandleW(null());
            let class_name = to_wide(&class_name_str);
            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: instance,
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                class_name.as_ptr(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
                instance,
                null(),
            );
            if hwnd.is_null() {
                return;
            }
            *hwnd_slot.lock().unwrap() = hwnd as isize;

            let added = add_tray_icon(hwnd);
            reg_flag.store(added, std::sync::atomic::Ordering::Relaxed);

            let tb_msg = RegisterWindowMessageW(to_wide("TaskbarCreated").as_ptr());
            TASKBAR_CREATED_MSG.store(tb_msg, std::sync::atomic::Ordering::Relaxed);

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if reg_flag.swap(false, std::sync::atomic::Ordering::Relaxed) {
                remove_tray_icon(hwnd);
            }
            DestroyWindow(hwnd);
        });

        (
            Self {
                control_hwnd,
                ctx_holder,
                tray_registered,
                join: Some(join),
            },
            rx,
        )
    }

    pub fn bind_context(&self, ctx: egui::Context) {
        *self.ctx_holder.lock().unwrap() = Some(ctx);
    }

    pub fn is_registered(&self) -> bool {
        self.tray_registered
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn shutdown(&mut self) {
        let raw = *self.control_hwnd.lock().unwrap();
        if raw != 0 {
            unsafe {
                PostMessageW(raw as HWND, MSG_QUIT_LOOP, 0, 0);
            }
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
        *TRAY_SHARED.lock().unwrap() = None;
    }
}

impl Drop for TrayController {
    fn drop(&mut self) {
        self.shutdown();
    }
}

unsafe fn add_tray_icon(hwnd: HWND) -> bool {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ID;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
    data.uCallbackMessage = MSG_TRAY;

    let tip = to_wide("澄明界面展示");
    let copy_len = tip.len().min(data.szTip.len());
    data.szTip[..copy_len].copy_from_slice(&tip[..copy_len]);

    let dpi = unsafe { GetDpiForSystem() };
    let hicon = unsafe {
        LoadImageW(
            GetModuleHandleW(null()),
            APP_ICON_ID as *const u16,
            IMAGE_ICON,
            GetSystemMetricsForDpi(SM_CXSMICON, dpi),
            GetSystemMetricsForDpi(SM_CYSMICON, dpi),
            0,
        )
    };
    if hicon.is_null() {
        return false;
    }
    data.hIcon = hicon;
    let added = unsafe {
        Shell_NotifyIconW(NIM_ADD, &data) != 0 || Shell_NotifyIconW(NIM_MODIFY, &data) != 0
    };
    if added {
        let mut ver: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        ver.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        ver.hWnd = hwnd;
        ver.uID = TRAY_ID;
        ver.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        unsafe {
            Shell_NotifyIconW(NIM_SETVERSION, &ver);
        }
    }
    unsafe {
        DestroyIcon(hicon);
    }
    added
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ID;
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

fn emit_event(ev: TrayEvent) {
    if let Some(shared) = TRAY_SHARED.lock().unwrap().as_ref().cloned() {
        let _ = shared.tx.send(ev);
        if let Some(ctx) = shared.ctx.lock().unwrap().as_ref() {
            ctx.request_repaint();
        }
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        MSG_ACTIVATE => {
            emit_event(TrayEvent::OpenMain);
            0
        }
        MSG_EXIT_CMD => {
            emit_event(TrayEvent::Exit);
            0
        }
        MSG_QUIT_LOOP => {
            unsafe { PostQuitMessage(0) };
            0
        }
        MSG_TRAY => {
            let event = (lparam as u32) & 0xFFFF;
            match event {
                WM_LBUTTONDBLCLK | WM_LBUTTONUP | NIN_SELECT | NIN_KEYSELECT => {
                    emit_event(TrayEvent::OpenMain);
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => unsafe {
                    show_tray_menu(hwnd);
                },
                _ => {}
            }
            0
        }
        other => {
            let tb_msg = TASKBAR_CREATED_MSG.load(std::sync::atomic::Ordering::Relaxed);
            if tb_msg != 0 && other == tb_msg {
                unsafe {
                    let _ = add_tray_icon(hwnd);
                }
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }
    let open_txt = to_wide("打开主窗口");
    let set_txt = to_wide("打开设置");
    let theme_txt = to_wide("切换浅色 / 深色主题");
    let exit_txt = to_wide("退出");

    unsafe {
        AppendMenuW(menu, MF_STRING, MENU_OPEN as usize, open_txt.as_ptr());
        AppendMenuW(menu, MF_STRING, MENU_SETTINGS as usize, set_txt.as_ptr());
        AppendMenuW(
            menu,
            MF_STRING,
            MENU_TOGGLE_THEME as usize,
            theme_txt.as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, MENU_EXIT as usize, exit_txt.as_ptr());

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
            pt.x,
            pt.y,
            0,
            hwnd,
            null(),
        ) as u32;
        DestroyMenu(menu);
        // Win32 KB135788：TrackPopupMenu 结束后须投递 WM_NULL，防止托盘菜单粘滞或阻塞后续退出
        PostMessageW(hwnd, WM_NULL, 0, 0);

        match cmd {
            MENU_OPEN => emit_event(TrayEvent::OpenMain),
            MENU_SETTINGS => emit_event(TrayEvent::OpenSettings),
            MENU_TOGGLE_THEME => emit_event(TrayEvent::ToggleTheme),
            MENU_EXIT => emit_event(TrayEvent::Exit),
            _ => {
                let mut data: NOTIFYICONDATAW = std::mem::zeroed();
                data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                data.hWnd = hwnd;
                data.uID = TRAY_ID;
                Shell_NotifyIconW(NIM_SETFOCUS, &data);
            }
        }
    }
}

pub fn apply_titlebar_theme(hwnd_raw: isize, dark: bool) {
    if hwnd_raw == 0 {
        return;
    }
    let val: i32 = if dark { 1 } else { 0 };
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd_raw as HWND,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            (&val as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }
}

pub fn is_windows_dark_theme() -> bool {
    let subkey = to_wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let value = to_wide("AppsUseLightTheme");
    let mut data: u32 = 1;
    let mut size: u32 = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            0x0000_0010, // RRF_RT_REG_DWORD
            null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    status == ERROR_SUCCESS && data == 0
}

pub fn autostart_value_name() -> String {
    std::env::var("UISHOWCASE_AUTORUN_NAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "澄明界面展示".to_owned())
}

pub fn is_autostart_enabled() -> bool {
    let subkey = to_wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let name = to_wide(&autostart_value_name());
    let mut size: u32 = 0;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut size,
        )
    };
    status == ERROR_SUCCESS && size > 2
}

pub fn get_autostart_command() -> String {
    let subkey = to_wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let name = to_wide(&autostart_value_name());
    let mut size: u32 = 0;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS || size == 0 {
        return "未注册".to_owned();
    }
    let mut buf = vec![0u16; (size as usize / 2) + 1];
    let status2 = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status2 != ERROR_SUCCESS {
        return "未注册".to_owned();
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

pub fn set_autostart_enabled(enable: bool, start_minimized: bool) -> Result<(), String> {
    let subkey = to_wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let name = to_wide(&autostart_value_name());
    if enable {
        let exe = std::env::current_exe().map_err(|e| format!("无法获取程序路径：{e}"))?;
        let cmd = if start_minimized {
            format!("\"{}\" --minimized", exe.display())
        } else {
            format!("\"{}\"", exe.display())
        };
        let wide_cmd = to_wide(&cmd);
        let bytes_len = (wide_cmd.len() * 2) as u32;
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                wide_cmd.as_ptr().cast(),
                bytes_len,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("写入登录启动项失败（错误码：{status}）。"));
        }
    } else {
        unsafe {
            let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, subkey.as_ptr(), name.as_ptr());
        }
    }
    Ok(())
}

pub fn load_window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory_with_format(
        include_bytes!("../assets/app.ico"),
        image::ImageFormat::Ico,
    )
    .ok()?
    .into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

pub fn load_icon_rgba_for_texture() -> Option<(usize, usize, Vec<u8>)> {
    let img = image::load_from_memory_with_format(
        include_bytes!("../assets/app.ico"),
        image::ImageFormat::Ico,
    )
    .ok()?
    .into_rgba8();
    let (w, h) = img.dimensions();
    Some((w as usize, h as usize, img.into_raw()))
}
