//! Wrapper library for RISC-V Supervisor Binary Interface (WIP)

use core::arch::asm;

/// SBI extension ID
#[repr(transparent)]
pub struct Eid(pub usize);

impl Eid {
    pub const CONSOLE_PUTCHAR: Self = Self(1);
    pub const CONSOLE_GETCHAR: Self = Self(2);
    pub const SHUTDOWN: Self = Self(8);
}

#[inline]
pub fn putchar(ch: u8) {
    unsafe {
        asm!("ecall", in("a7") Eid::CONSOLE_PUTCHAR.0, in("a0") ch as usize);
    }
}

pub struct StdOut;

impl core::fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            putchar(c);
        }
        Ok(())
    }
}

#[inline]
pub fn getchar() -> Option<u8> {
    unsafe {
        let c: usize;
        asm!("ecall", in("a7") Eid::CONSOLE_GETCHAR.0, lateout("a0") c);
        match c {
            0..=255 => Some(c as u8),
            _ => None,
        }
    }
}

#[inline]
pub fn shutdown() -> ! {
    unsafe {
        asm!("ecall", in("a7") Eid::SHUTDOWN.0, options(noreturn));
    }
}
