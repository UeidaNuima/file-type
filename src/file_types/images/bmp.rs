//! BMP (Bitmap) 文件
//! https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage
use std::io::Write;

use crate::consts::{DWORD, LONG, WORD};

const BI_RGB: DWORD = 0;
const BI_BITFIELDS: DWORD = 3;

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader
pub struct BitmapFileHeader {
    /// The file type; must be 0x4d42 (the ASCII string "BM").
    bf_type: WORD,
    /// The size, in bytes, of the bitmap file.
    bf_size: DWORD,
    /// Reserved; must be zero.
    bf_reserved1: WORD,
    /// Reserved; must be zero.
    bf_reserved2: WORD,
    /// The offset, in bytes, from the beginning of the BITMAPFILEHEADER structure to the bitmap bits.
    bf_off_bits: DWORD,
}

impl BitmapFileHeader {
    pub fn new(size: u32) -> Self {
        Self {
            bf_type: 0x4d42,
            bf_size: size,
            bf_reserved1: Default::default(),
            bf_reserved2: Default::default(),
            bf_off_bits: Default::default(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_all(&self.bf_type.to_le_bytes()).unwrap();
        buf.write_all(&self.bf_size.to_le_bytes()).unwrap();
        buf.write_all(&self.bf_reserved1.to_le_bytes()).unwrap();
        buf.write_all(&self.bf_reserved2.to_le_bytes()).unwrap();
        buf.write_all(&self.bf_off_bits.to_le_bytes()).unwrap();
        buf
    }

    pub fn set_image_data_info(&mut self, offset: u32, size: u32) {
        self.bf_off_bits = offset;
        self.bf_size = offset + size;
    }

    pub fn get_byte_size() -> u32 {
        14
    }
}

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader
pub struct BitmapInfoHeader {
    /// Specifies the number of bytes required by the structure. This value does not include the size of the color table or the size of the color masks, if they are appended to the end of structure. See Remarks.
    bi_size: DWORD,
    /// Specifies the width of the bitmap, in pixels. For information about calculating the stride of the bitmap, see Remarks.
    bi_width: LONG,
    /// Specifies the height of the bitmap, in pixels.
    /// - For uncompressed RGB bitmaps, if `biHeight` is positive, the bitmap is a bottom-up DIB with the origin at the lower left corner. If `biHeight` is negative, the bitmap is a top-down DIB with the origin at the upper left corner.
    /// - For YUV bitmaps, the bitmap is always top-down, regardless of the sign of `biHeight`. Decoders should offer YUV formats with positive `biHeight`, but for backward compatibility they should accept YUV formats with either positive or negative `biHeight`.
    /// - For compressed formats, `biHeight` must be positive, regardless of image orientation.
    bi_height: LONG,
    /// Specifies the number of planes for the target device. This value must be set to 1.
    bi_planes: WORD,
    /// Specifies the number of bits per pixel (bpp). For uncompressed formats, this value is the average number of bits per pixel. For compressed formats, this value is the implied bit depth of the uncompressed image, after the image has been decoded.
    bi_bit_count: WORD,
    /// For compressed video and YUV formats, this member is a FOURCC code, specified as a `DWORD` in little-endian order. For example, YUYV video has the FOURCC 'VYUY' or 0x56595559. For more information, see [FOURCC Codes](https://learn.microsoft.com/en-us/windows/desktop/DirectShow/fourcc-codes).
    ///
    /// For uncompressed RGB formats, the following values are possible:
    ///
    /// |Value|Meaning|
    /// |-----|-------|
    /// |BI_RGB|Uncompressed RGB.|
    /// |BI_BITFIELDS|Uncompressed RGB with color masks. Valid for 16-bpp and 32-bpp bitmaps.|
    /// See Remarks for more information. Note that `BI_JPG` and `BI_PNG` are not valid video formats.
    ///
    /// For 16-bpp bitmaps, if `biCompression` equals `BI_RGB`, the format is always RGB 555. If `biCompression` equals `BI_BITFIELDS`, the format is either RGB 555 or RGB 565. Use the subtype GUID in the [AM_MEDIA_TYPE](https://learn.microsoft.com/en-us/windows/desktop/api/strmif/ns-strmif-am_media_type) structure to determine the specific RGB type.
    bi_compression: DWORD,
    /// Specifies the size, in bytes, of the image. This can be set to 0 for uncompressed RGB bitmaps.
    bi_size_image: DWORD,
    /// Specifies the horizontal resolution, in pixels per meter, of the target device for the bitmap.
    bi_x_pels_per_meter: LONG,
    /// Specifies the vertical resolution, in pixels per meter, of the target device for the bitmap.
    bi_y_pels_per_meter: LONG,
    /// Specifies the number of color indices in the color table that are actually used by the bitmap. See Remarks for more information.
    bi_clr_used: DWORD,
    /// Specifies the number of color indices that are considered important for displaying the bitmap. If this value is zero, all colors are important.
    bi_clr_important: DWORD,
}

impl BitmapInfoHeader {
    pub fn new(bit_count: u16, width: u32, height: u32) -> Self {
        if !matches!(bit_count, 1 | 4 | 8 | 16 | 24 | 32) {
            panic!("Unknown bit count (depth) for bmp: {bit_count}");
        }
        Self {
            bi_size: BitmapInfoHeader::get_byte_size(),
            bi_width: width as i32,
            bi_height: height as i32,
            bi_planes: 1,
            bi_bit_count: bit_count,
            bi_compression: BI_RGB,
            bi_size_image: 0,
            // windows default (96 dpi)
            bi_x_pels_per_meter: 3780,
            bi_y_pels_per_meter: 3780,
            // not used when no color palette is provided
            bi_clr_used: Default::default(),
            // not used in modern display
            bi_clr_important: Default::default(),
        }
    }

    pub fn get_byte_size() -> u32 {
        40
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_all(&self.bi_size.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_width.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_height.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_planes.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_bit_count.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_compression.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_size_image.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_x_pels_per_meter.to_le_bytes())
            .unwrap();
        buf.write_all(&self.bi_y_pels_per_meter.to_le_bytes())
            .unwrap();
        buf.write_all(&self.bi_clr_used.to_le_bytes()).unwrap();
        buf.write_all(&self.bi_clr_important.to_le_bytes()).unwrap();
        buf
    }
}

#[derive(Clone)]
pub struct ColorPixel {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorPixel {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn white() -> Self {
        Self::new_rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new_rgb(0, 0, 0)
    }
}

pub struct ColorMatrix {
    pub data: Vec<ColorPixel>,
    pub width: u32,
    pub height: u32,
    push_cursor: usize,
}

impl ColorMatrix {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![ColorPixel::black(); (width * height) as usize],
            width,
            height,
            push_cursor: 0,
        }
    }

    pub fn push(&mut self, pixel: ColorPixel) {
        if self.push_cursor > (self.width * self.height) as usize {
            panic!("Push overflow");
        }
        self.data[self.push_cursor] = pixel;
        self.push_cursor += 1;
    }

    pub fn set(&mut self, x: u32, y: u32, pixel: ColorPixel) {
        if x > self.width || y > self.height {
            panic!("Point overflow");
        }

        self.data[(self.width * y + x) as usize] = pixel;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        for pixel_line in self.data.chunks(self.width as usize) {
            let mut line_buf = vec![];
            for pixel in pixel_line {
                // BGR order
                line_buf.push(pixel.b);
                line_buf.push(pixel.g);
                line_buf.push(pixel.r);
            }
            // padding to multiple of 4
            let padding = (4 - pixel_line.len() % 4) % 4;
            line_buf.extend(vec![0; padding]);
            line_buf.reverse();
            buf.extend(line_buf);
        }

        buf.reverse();

        buf
    }

    pub fn get_byte_size(&self) -> u32 {
        (self.width / 4 + (4 - self.width % 4) % 4) * self.height
    }
}

pub struct Bitmap {
    file_header: BitmapFileHeader,
    info_header: BitmapInfoHeader,
    pub color_data: ColorMatrix,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        let mut file_header = BitmapFileHeader::new(width * height);
        let info_header = BitmapInfoHeader::new(24, width, height);
        let color_data = ColorMatrix::new(width, height);
        file_header.set_image_data_info(
            BitmapFileHeader::get_byte_size() + BitmapInfoHeader::get_byte_size(),
            color_data.get_byte_size(),
        );

        Bitmap {
            file_header,
            info_header,
            color_data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(self.file_header.to_bytes());
        buf.extend(self.info_header.to_bytes());
        buf.extend(self.color_data.to_bytes());

        buf
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs::{self, File},
        io::{BufWriter, Write},
        path::Path,
    };

    use super::{Bitmap, ColorPixel};

    #[test]
    pub fn test_generate_24_depth_uncompressed() {
        let mut bmp = Bitmap::new(4096, 4096);

        for r in 0u16..256 {
            for g in 0u16..256 {
                for b in 0u16..256 {
                    let bx = (b & 0x0F) as u32;
                    let by = (b >> 4) as u32;
                    let x = r as u32 + 256 * bx;
                    let y = g as u32 + 256 * by;
                    bmp.color_data.set(
                        x,
                        y,
                        ColorPixel {
                            r: r as u8,
                            g: g as u8,
                            b: b as u8,
                        },
                    );
                }
            }
        }

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/24_uncompressed.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }
}
