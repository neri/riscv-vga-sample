//!
#![no_std]
#![no_main]

use core::{convert::Infallible, fmt::Write};
use embedded_graphics::{
    mono_font::{ascii::FONT_7X14, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use kernel::*;

arch_entry!(start);

fn start() -> ! {
    let fb_base = 0x5000_0000 as *mut u8;

    unsafe {
        // NOTE: In qemu, `pci_base` is 0x3000_0000 and `dev` is 1, so `dev_base` is 0x3000_8000.
        let dev_base = 0x3000_8000 as *mut u32;

        // I want VGA
        let pci_class = dev_base.add(2).read_volatile() >> 8;
        if pci_class != 0x030000 {
            panic!("virtio-vga was not found");
        }

        // PCI configuration
        let io_base = {
            dev_base.add(4).write_volatile(0xFFFF_FFFF);
            let fb_size = (!dev_base.add(4).read_volatile() | 0xF) as usize + 1;
            let io_base = fb_base.add(fb_size);
            dev_base.add(6).write_volatile(0xFFFF_FFFF);
            let io_size = (!dev_base.add(6).read_volatile() | 0xF) as usize + 1;

            dev_base.add(4).write_volatile(fb_base as u32 | 8);
            dev_base.add(6).write_volatile(io_base as u32 | 8);

            let cmd = dev_base.add(1);
            cmd.write_volatile(cmd.read_volatile() | 0x0002);

            println!(
                "VGA fb_base {:08x} size {}M io_base {:08x} size {}",
                fb_base as usize,
                (fb_size + 0x80000) >> 20,
                io_base as usize,
                io_size,
            );

            io_base
        };

        vga::set_mode13(io_base);

        fb_base.write_bytes(0, 320 * 200);
    };

    let mut display = unsafe { Mode13Display::new(fb_base) };

    Text::new(
        "Hello, world!",
        Point::new(4, 18),
        MonoTextStyleBuilder::new()
            .font(&FONT_7X14)
            .text_color(Rgb888::WHITE)
            .build(),
    )
    .draw(&mut display)
    .unwrap();

    Circle::new(Point::new(60, 30), 80)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLUE, 1))
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(110, 80), 80)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(160, 30), 80)
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
        .draw(&mut display)
        .unwrap();

    loop {
        let c = sbi::getchar();
        if let Some(c) = c {
            if c != 0 {
                sbi::putchar(c);
            }
        }
    }
}

pub struct Mode13Display {
    base: *mut u8,
}

impl Mode13Display {
    #[inline]
    pub unsafe fn new(base: *mut u8) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn _color_component_to_safe_color(c: u8) -> u8 {
        const TABLE: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        TABLE[c as usize]
    }

    pub fn set_pixel(&mut self, coord: Point, color: Rgb888) -> Option<()> {
        if let Ok((x @ 0..=319, y @ 0..=199)) = coord.try_into() {
            let index = x as usize + y as usize * 320;

            let r = Self::_color_component_to_safe_color(color.r());
            let g = Self::_color_component_to_safe_color(color.g());
            let b = Self::_color_component_to_safe_color(color.b());
            let color = 16 + r + g * 6 + b * 36;

            unsafe {
                self.base.add(index).write_volatile(color);
            }

            Some(())
        } else {
            None
        }
    }
}

impl DrawTarget for Mode13Display {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            self.set_pixel(coord, color);
        }
        Ok(())
    }
}

impl OriginDimensions for Mode13Display {
    fn size(&self) -> Size {
        Size::new(320, 200)
    }
}
