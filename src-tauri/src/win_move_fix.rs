//! Windows：消除窗口拖动 / 缩放的起手迟滞。
//!
//! 根因（tauri#10767）：wry 给父窗口装的子类在 WM_ENTERSIZEMOVE 上同步跨进程
//! 调用 ICoreWebView2Controller::MoveFocus，实测耗时 22–58ms；而系统要等这条
//! 消息返回才开始移动窗口——表现为"快速一点一甩，窗口先卡一下再跟上"，拖边框
//! 缩放同样中招。上游已由 wry PR #1910 修复（把这一项从 wry 子类里删掉），
//! 但本项目锁定的 wry 0.55.1 尚未包含。
//!
//! 做法（该 issue 里实测过：拖拽起手 64ms → 21ms，与纯 Win32 窗口的 19ms
//! 基本持平）：在 wry 之后**再装一层子类**，由它抢先应答并吞掉
//! WM_ENTERSIZEMOVE，内层的 wry / tao 都收不到——对 wry 的效果与上游修复
//! 等效，且不需要改动依赖。
//!
//! 已知取舍：tao 原本靠这条消息置"移动中"标志，现在拖动期间不会置位。该标志
//! 在 tao 0.35.3 里只有一处读取，位于 WM_DPICHANGED 的 Windows 10 分支
//! （build < 22000，跨不同缩放比例的显示器拖动时锚住光标）；Win11 不受影响。

use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::WM_ENTERSIZEMOVE;

/// 子类标识：只需与同窗口上其它子类（wry / tao）不同即可
const SUBCLASS_ID: usize = 0x524D_4D46; // "RMMF" = RustMusic Move Fix

unsafe extern "system" fn move_fix_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid_subclass: usize,
    _dw_ref_data: usize,
) -> LRESULT {
    match msg {
        WM_ENTERSIZEMOVE => LRESULT(0),
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}

/// 给主窗口装上拖动修复子类。必须在网页视图创建之后调用（setup 阶段即
/// 满足）：子类按"后装的先执行"串联，晚于 wry 安装才会排在它前面。
/// 窗口销毁时系统自动摘除子类，无需手动清理
pub fn install(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    let Ok(hwnd) = w.hwnd() else {
        return;
    };
    unsafe {
        let _ = SetWindowSubclass(HWND(hwnd.0 as *mut _), Some(move_fix_proc), SUBCLASS_ID, 0);
    }
}
