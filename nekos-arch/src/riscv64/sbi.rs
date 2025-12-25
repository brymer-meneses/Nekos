#[repr(C)]
struct SibReturn {
    error: i32,
    value: i32,
}

unsafe fn call(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    fid: i32,
    eid: i32,
) -> SibReturn {
    let mut error;
    let mut value;

    unsafe {
        core::arch::asm!(
            "ecall",
            inout("a0") arg0 => error,
            inout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid,
        );
    }

    SibReturn { error, value }
}

pub struct SbiWriter;

use core::fmt;

impl fmt::Write for SbiWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            unsafe { call(c as i32, 0, 0, 0, 0, 0, 0, 1) };
        }

        Ok(())
    }
}

pub fn log(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = SbiWriter {};
    writer.write_fmt(args).unwrap();
}
