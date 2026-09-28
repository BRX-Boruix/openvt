//! BORUIX `openvt`：运行期开新终端的用户入口（B3-C3，ADR-048 owner 裁决）。
//!
//! **语义**：向 init 递交「开一个新终端」的请求——create 请求文件
//! `/system/console-requests/<n>`（n 从 1 向上试到 CONSOLES_MAX-1）。
//! init 的 supervisor 巡检消费请求：spawn consoled(N)+login(N)、双账本
//! 登记、unlink 请求文件（B3-C2）。
//!
//! **id 试错协议（S21）**：create 成功 = 预约成功（该实例 id 已被本次请求
//! 占用；init 侧幂等去重保证重复预约无害）；AlreadyExists = 该号已被其它
//! 请求/实例占用 → 试下一号。用户不挑号（openvt 惯例），分配的最终真值
//! 由 init 的串口日志给出（"openvt: instance N created"）。
//!
//! **失败模式（S09 如实）**：
//! - 全部 1..63 都被占 → "no free console slot"，退出非零（上限诚实）；
//! - 请求目录不存在（老内核/目录未建）→ 如实报错退出；
//! - create 的其它错误 → 如实报错退出。
//!
//! 上限 64 与内核 `vfs::console::CONSOLES_MAX` / init `CONSOLES_MAX`
//! 同值（S13 注释链；内核侧被 test_console_runtime_create 钉死）。
#![no_std]
#![no_main]

extern crate alloc;

use libsys::{close, open, write, OpenFlags, Permissions, STDOUT};

/// 实例 id 试错上界（不含）——与内核/ init 的 CONSOLES_MAX=64 同值。
const CONSOLES_MAX: usize = 64;

fn out(b: &[u8]) {
    let _ = write(STDOUT, b);
}

fn out_u64(mut v: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if v == 0 {
        i -= 1;
        buf[i] = b'0';
    }
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    out(&buf[i..]);
}

/// 请求文件路径（/system/console-requests/<id>）：栈缓冲拼接（零堆，S17）。
/// 48 = 前缀 26 + 2 位数字（id<64）+ NUL 余量。
fn request_path(id: usize, buf: &mut [u8]) -> &str {
    const PREFIX: &[u8] = b"/system/console-requests/";
    let mut len = 0usize;
    for b in PREFIX.iter() {
        buf[len] = *b;
        len += 1;
    }
    let mut digits = [0u8; 2];
    let mut n = 0usize;
    let mut v = id;
    while v > 0 {
        digits[n] = b'0' + (v % 10) as u8;
        n += 1;
        v /= 10;
    }
    let mut k = n;
    while k > 0 {
        k -= 1;
        buf[len] = digits[k];
        len += 1;
    }
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

/// Device node path (/devices/consoles/<id>): stack buffer join, zero-heap.
/// Used to skip boot-time pre-created instances (node exists = in use).
fn device_path(id: usize, buf: &mut [u8]) -> &str {
    const PREFIX: &[u8] = b"/devices/consoles/";
    let mut len = 0usize;
    for b in PREFIX.iter() {
        buf[len] = *b;
        len += 1;
    }
    let mut digits = [0u8; 2];
    let mut n = 0usize;
    let mut v = id;
    while v > 0 {
        digits[n] = b'0' + (v % 10) as u8;
        n += 1;
        v /= 10;
    }
    let mut k = n;
    while k > 0 {
        k -= 1;
        buf[len] = digits[k];
        len += 1;
    }
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

#[unsafe(no_mangle)]
pub extern "C" fn user_main(_argc: isize, _argv: *const *const u8) -> i32 {
    out(b"[openvt] requesting a new console instance\n");
    let mut id = 1usize;
    while id < CONSOLES_MAX {
        // Boot-time pre-created instances (0..CONSOLES_N-1) are already in
        // use: skip ids whose device node exists (S13: node table = truth).
        let mut devbuf = [0u8; 32];
        let devpath = device_path(id, &mut devbuf);
        match open(devpath, OpenFlags::READ_ONLY, Permissions::readonly()) {
            Ok(fd) => {
                let _ = close(fd);
                id += 1;
                continue;
            }
            Err(libsys::Error::NotFound) => {}
            Err(_) => {
                out(b"[openvt] FATAL: cannot probe /devices/consoles\n");
                return 1;
            }
        }
        let mut buf = [0u8; 48];
        let path = request_path(id, &mut buf);
        // probe-then-create (S09 boundary): no O_EXCL bit in OpenFlags.
        // Existing file = pending request for this id: skip, never
        // CREATE_OR_TRUNCATE over someone else pending request.
        // The init patrol ledger dedup remains authoritative (S13).
        match open(path, OpenFlags::READ_ONLY, Permissions::readonly()) {
            Ok(fd) => {
                let _ = close(fd);
                id += 1;
                continue;
            }
            Err(libsys::Error::NotFound) => {}
            Err(_) => {
                out(b"[openvt] FATAL: cannot probe /system/console-requests\n");
                return 1;
            }
        }
        match open(path, OpenFlags::CREATE_OR_TRUNCATE, Permissions::all()) {
            Ok(fd) => {
                let _ = close(fd);
                out(b"[openvt] request queued for instance ");
                out_u64(id as u64);
                out(b"; wait for init patrol log (instance N created)\n");
                return 0;
            }
            Err(_) => {
                out(b"[openvt] FATAL: cannot write request under /system/console-requests\n");
                return 1;
            }
        }
    }
    out(b"[openvt] no free console slot (cap 64)\n");
    1
}
