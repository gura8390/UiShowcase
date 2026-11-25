use std::cell::Cell;
use std::time::Instant;

use eframe::{AppCreator, EframeWinitApplication, NativeOptions, UserEvent};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{IsIconic, IsWindowVisible};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

/// 适配 Windows 隐藏窗口（关闭到托盘 / --minimized / 最小化）的 eframe 事件循环包装器。
/// Win32 不会为隐藏或最小化的 HWND 派发 WM_PAINT；此处在 about_to_wait 中直接向 eframe
/// 投递 RedrawRequested，使托盘「退出」「打开主窗口」「打开设置」「切换主题」以及单实例唤回
/// 在窗口隐藏状态下也能立即响应，且静止时保持零轮询占用。
pub fn run(title: &str, options: NativeOptions, create: AppCreator<'_>) -> eframe::Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    let window = Cell::new(None);
    let app = eframe::create_native(
        title,
        options,
        Box::new(|cc| {
            if let Ok(handle) = cc.window_handle()
                && let RawWindowHandle::Win32(h) = handle.as_raw()
            {
                window.set(Some(h.hwnd.get()));
            }
            create(cc)
        }),
        &event_loop,
    );

    event_loop
        .run_app(&mut WindowsEventLoop {
            app,
            window: &window,
            repaint_at: None,
        })
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    Ok(())
}

struct WindowsEventLoop<'a> {
    app: EframeWinitApplication<'a>,
    window: &'a Cell<Option<isize>>,
    repaint_at: Option<Instant>,
}

impl ApplicationHandler<UserEvent> for WindowsEventLoop<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.app.new_events(event_loop, cause);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app.resumed(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        if let UserEvent::RequestRepaint {
            viewport_id, when, ..
        } = &event
            && *viewport_id == eframe::egui::ViewportId::ROOT
        {
            self.repaint_at = Some(self.repaint_at.map_or(*when, |pending| pending.min(*when)));
        }
        self.app.user_event(event_loop, event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::RedrawRequested)
            && self
                .window
                .get()
                .is_some_and(|handle| u64::from(id) == handle as u64)
        {
            self.repaint_at = None;
        }
        self.app.window_event(event_loop, id, event);
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, id: DeviceId, event: DeviceEvent) {
        self.app.device_event(event_loop, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.about_to_wait(event_loop);
        if let Some(handle) = self.window.get()
            && unsafe {
                let hwnd = handle as HWND;
                IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0
            }
            && let Some(when) = self.repaint_at
        {
            if when <= Instant::now() {
                self.repaint_at = None;
                self.app.window_event(
                    event_loop,
                    WindowId::from(handle as u64),
                    WindowEvent::RedrawRequested,
                );
            } else {
                let deadline = match event_loop.control_flow() {
                    ControlFlow::WaitUntil(deadline) => deadline.min(when),
                    _ => when,
                };
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
        }
        if event_loop.control_flow() == ControlFlow::Poll {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.app.suspended(event_loop);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.app.exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.app.memory_warning(event_loop);
    }
}
