use core::panic;

use super::ColorDepth;

#[derive(Clone, PartialEq, Copy)]
pub struct ColorPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorPixel {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new_rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new_rgb(0, 0, 0)
    }

    pub fn red() -> Self {
        Self::new_rgb(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new_rgb(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new_rgb(0, 0, 255)
    }

    pub fn white_transparent() -> Self {
        Self::new_rgba(255, 255, 255, 0)
    }

    pub fn black_transparent() -> Self {
        Self::new_rgba(0, 0, 0, 0)
    }
}

pub struct RawImage {
    pub data: Vec<ColorPixel>,
    pub width: u32,
    pub height: u32,
    pub depth: ColorDepth,
}

impl RawImage {
    pub fn new(width: u32, height: u32, color_depth: ColorDepth) -> Self {
        Self {
            data: vec![ColorPixel::black(); (width * height) as usize],
            width,
            height,
            depth: color_depth,
        }
    }

    pub fn set(&mut self, x: u32, y: u32, pixel: ColorPixel) {
        if x > self.width || y > self.height {
            panic!("Point overflow");
        }

        self.data[(self.width * y + x) as usize] = pixel;
    }

    pub fn get_indexed_info(&self) -> (Vec<ColorPixel>, Vec<usize>) {
        let mut palette: Vec<ColorPixel> = vec![];
        let mut indexed_data = vec![];
        for pixel in &self.data {
            if let Some(index) = palette
                .iter()
                .enumerate()
                .find(|(_, pc)| *pc == pixel)
                .map(|(idx, _)| idx)
            {
                indexed_data.push(index);
            } else {
                palette.push(*pixel);
                indexed_data.push(palette.len() - 1);
            }
        }
        let palette_num = 1 << (self.depth as u8);
        if palette.len() > palette_num {
            panic!(
                "Palette number overflow for depth {:?}: {}",
                self.depth,
                palette.len(),
            );
        }
        if palette.len() < palette_num {
            palette.extend(vec![ColorPixel::white(); palette_num - palette.len()]);
        }
        (palette, indexed_data)
    }
}

pub trait BinaryInfo {
    fn to_bytes(&self) -> Vec<u8>;
    fn get_byte_size(&self) -> u32;
}
