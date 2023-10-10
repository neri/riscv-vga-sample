#![no_std]

pub mod sbi;
pub mod vga;

use core::{arch::asm, fmt::Write, panic::PanicInfo};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = write!(crate::stdout(), $($arg)*);
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        let _ = writeln!(crate::stdout(), $($arg)*);
    };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("\n\x1b[m#### KERNEL PANIC ####\n{}", _info);
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[macro_export]
macro_rules! arch_entry {
    ($start:ident) => {
        use core::arch::{asm, global_asm};

        extern "C" {
            static __bss: *mut u8;
            static __bss_end: *const u8;
            static __stack_top: *mut u8;
        }

        global_asm!(
            "
        .section .text.boot
        .global boot
        .align 4
        boot:
            la sp, __stack_top
            j _arch_riscv_start
        "
        );

        #[no_mangle]
        fn _arch_riscv_start(_hartid: usize, _dtb: usize) -> ! {
            unsafe {
                __bss.write_bytes(0, __bss_end as usize - __bss as usize);
            };

            $start();

            #[allow(unreachable_code)]
            loop {
                unsafe {
                    asm!("wfi");
                }
            }
        }
    };
}

#[inline]
pub fn stdout() -> impl Write {
    sbi::StdOut
}
