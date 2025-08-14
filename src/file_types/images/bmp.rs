//! BMP (Bitmap) 文件
//! https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage
//! https://en.wikipedia.org/wiki/BMP_file_format
use std::io::Write;

use super::common::{
    ColorDepth,
    image::{BinaryInfo, ColorPixel, RawImage},
    utils::{get_padding_num, padding_to_base},
};

const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;

fn is_indexed_image(image: &RawImage) -> bool {
    matches!(
        image.depth,
        ColorDepth::Depth1 | ColorDepth::Depth4 | ColorDepth::Depth8
    )
}

impl BinaryInfo for RawImage {
    fn to_bytes(&self) -> Vec<u8> {
        if is_indexed_image(self) {
            let mut lines_buf = vec![];
            let mut palette_buf = vec![];
            let (palette, indexed_colors) = self.get_indexed_info();
            for palette_color in palette {
                palette_buf.extend(RgbQuad::from(palette_color).to_bytes());
            }
            for indexed_line in indexed_colors.chunks(self.width as usize) {
                let mut line_buf = vec![];
                let colors_per_pixel = match self.depth {
                    ColorDepth::Depth1 => 8,
                    ColorDepth::Depth4 => 2,
                    ColorDepth::Depth8 => 1,
                    depth => panic!("Not indexed color for depth {depth:?}"),
                };
                for indexed_colors_in_byte in indexed_line.chunks(colors_per_pixel) {
                    let mut byte_buf: u8 = 0;
                    if (self.depth as u8) < 8u8 {
                        for indexed_color in indexed_colors_in_byte {
                            byte_buf = (byte_buf << self.depth as u8) | (*indexed_color as u8);
                        }
                    } else {
                        byte_buf = indexed_colors_in_byte[0] as u8;
                    }

                    if indexed_colors_in_byte.len() < colors_per_pixel {
                        // align last colors to a full byte
                        byte_buf <<=
                            (colors_per_pixel - indexed_colors_in_byte.len()) * self.depth as usize;
                    }

                    line_buf.push(byte_buf);
                }
                // padding to multiple of 4
                line_buf.extend(vec![0; get_padding_num(line_buf.len() as u32, 4) as usize]);
                lines_buf.push(line_buf);
            }
            lines_buf.reverse();
            [palette_buf, lines_buf.concat()].concat()
        } else {
            let mut lines_buf = vec![];
            for pixel_line in self.data.chunks(self.width as usize) {
                let mut line_buf = vec![];
                for pixel in pixel_line {
                    if matches!(self.depth, ColorDepth::Depth16) {
                        // RGB order, RGB555
                        // high endian for red
                        let mut pixel16: u16 = 0;
                        // 5 bits for r
                        pixel16 |= pixel.r as u16 / 8;
                        pixel16 <<= 5;
                        // 5 bits for g
                        pixel16 |= pixel.g as u16 / 8;
                        pixel16 <<= 5;
                        // 5 bits for b
                        pixel16 |= pixel.b as u16 / 8;
                        line_buf.extend(pixel16.to_le_bytes());
                    } else {
                        // BGR order，BGR888
                        line_buf.push(pixel.b);
                        line_buf.push(pixel.g);
                        line_buf.push(pixel.r);
                        if self.depth == ColorDepth::Depth32 {
                            line_buf.push(pixel.a);
                        }
                    }
                }
                // padding to multiple of 4
                line_buf.extend(vec![0; get_padding_num(line_buf.len() as u32, 4) as usize]);
                lines_buf.push(line_buf);
            }
            lines_buf.reverse();
            lines_buf.concat()
        }
    }

    fn get_byte_size(&self) -> u32 {
        match self.depth {
            ColorDepth::Depth1 => {
                padding_to_base(self.width / 8, 4) * self.height + RgbQuad::get_byte_size() * 2
            }
            ColorDepth::Depth4 => {
                padding_to_base(self.width / 2, 4) * self.height + RgbQuad::get_byte_size() * 4
            }
            ColorDepth::Depth8 => {
                padding_to_base(self.width, 4) * self.height + RgbQuad::get_byte_size() * 8
            }
            ColorDepth::Depth16 => padding_to_base(self.width * 2, 4) * self.height,
            ColorDepth::Depth24 => padding_to_base(self.width * 3, 4) * self.height,
            ColorDepth::Depth32 => self.width * 4 * self.height,
        }
    }
}

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader
pub struct BitmapFileHeader {
    /// The file type; must be 0x4d42 (the ASCII string "BM").
    bf_type: u16,
    /// The size, in bytes, of the bitmap file.
    bf_size: u32,
    /// Reserved; must be zero.
    bf_reserved1: u16,
    /// Reserved; must be zero.
    bf_reserved2: u16,
    /// The offset, in bytes, from the beginning of the BITMAPFILEHEADER structure to the bitmap bits.
    bf_off_bits: u32,
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
    bi_size: u32,
    /// Specifies the width of the bitmap, in pixels. For information about calculating the stride of the bitmap, see Remarks.
    bi_width: u32,
    /// Specifies the height of the bitmap, in pixels.
    /// - For uncompressed RGB bitmaps, if `biHeight` is positive, the bitmap is a bottom-up DIB with the origin at the lower left corner. If `biHeight` is negative, the bitmap is a top-down DIB with the origin at the upper left corner.
    /// - For YUV bitmaps, the bitmap is always top-down, regardless of the sign of `biHeight`. Decoders should offer YUV formats with positive `biHeight`, but for backward compatibility they should accept YUV formats with either positive or negative `biHeight`.
    /// - For compressed formats, `biHeight` must be positive, regardless of image orientation.
    bi_height: u32,
    /// Specifies the number of planes for the target device. This value must be set to 1.
    bi_planes: u16,
    /// Specifies the number of bits per pixel (bpp). For uncompressed formats, this value is the average number of bits per pixel. For compressed formats, this value is the implied bit depth of the uncompressed image, after the image has been decoded.
    bi_bit_count: u16,
    /// For compressed video and YUV formats, this member is a FOURCC code, specified as a `u32` in little-endian order. For example, YUYV video has the FOURCC 'VYUY' or 0x56595559. For more information, see [FOURCC Codes](https://learn.microsoft.com/en-us/windows/desktop/DirectShow/fourcc-codes).
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
    bi_compression: u32,
    /// Specifies the size, in bytes, of the image. This can be set to 0 for uncompressed RGB bitmaps.
    bi_size_image: u32,
    /// Specifies the horizontal resolution, in pixels per meter, of the target device for the bitmap.
    bi_x_pels_per_meter: u32,
    /// Specifies the vertical resolution, in pixels per meter, of the target device for the bitmap.
    bi_y_pels_per_meter: u32,
    /// Specifies the number of color indices in the color table that are actually used by the bitmap. See Remarks for more information.
    bi_clr_used: u32,
    /// Specifies the number of color indices that are considered important for displaying the bitmap. If this value is zero, all colors are important.
    bi_clr_important: u32,
}

impl BitmapInfoHeader {
    pub fn new(color_depth: ColorDepth, width: u32, height: u32) -> Self {
        Self {
            bi_size: BitmapInfoHeader::get_byte_size(),
            bi_width: width,
            bi_height: height,
            bi_planes: 1,
            bi_bit_count: color_depth as u16,
            bi_compression: BI_RGB,
            bi_size_image: 0,
            // windows default (96 dpi)
            bi_x_pels_per_meter: 3780,
            bi_y_pels_per_meter: 3780,
            bi_clr_used: match color_depth {
                ColorDepth::Depth1 => 2,
                ColorDepth::Depth4 => 16,
                ColorDepth::Depth8 => 256,
                _ => 0,
            },
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

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-rgbquad
pub struct RgbQuad {
    /// The intensity of blue in the color.
    blue: u8,
    /// The intensity of green in the color.
    green: u8,
    /// The intensity of red in the color.
    red: u8,
    /// This member is reserved and must be zero.
    reserved: u8,
}

impl RgbQuad {
    fn to_bytes(&self) -> Vec<u8> {
        vec![self.blue, self.green, self.red, self.reserved]
    }
    pub fn get_byte_size() -> u32 {
        4
    }
}

impl From<ColorPixel> for RgbQuad {
    fn from(value: ColorPixel) -> Self {
        Self {
            blue: value.b,
            green: value.g,
            red: value.r,
            reserved: Default::default(),
        }
    }
}

pub struct Bitmap {
    file_header: BitmapFileHeader,
    info_header: BitmapInfoHeader,
    pub color_data: RawImage,
}

impl Bitmap {
    pub fn new(image: RawImage) -> Self {
        let mut file_header = BitmapFileHeader::new(image.width * image.height);
        let info_header = BitmapInfoHeader::new(image.depth, image.width, image.height);
        file_header.set_image_data_info(
            BitmapFileHeader::get_byte_size() + BitmapInfoHeader::get_byte_size(),
            image.get_byte_size(),
        );

        Bitmap {
            file_header,
            info_header,
            color_data: image,
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

    use rand::Rng;

    use crate::file_types::images::common::{
        ColorDepth,
        image::{ColorPixel, RawImage},
    };

    use super::Bitmap;

    pub fn generate_random_color() -> ColorPixel {
        let mut rng = rand::rng();
        ColorPixel::new_rgba(
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
        )
    }

    #[test]
    pub fn test_generate_24_depth() {
        let mut image = RawImage::new(4096, 4096, ColorDepth::Depth24);

        for r in 0u16..256 {
            for g in 0u16..256 {
                for b in 0u16..256 {
                    let bx = (b & 0x0F) as u32;
                    let by = (b >> 4) as u32;
                    let x = r as u32 + 256 * bx;
                    let y = g as u32 + 256 * by;
                    image.set(x, y, ColorPixel::new_rgb(r as u8, g as u8, b as u8));
                }
            }
        }

        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/24.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_generate_32_depth() {
        let mut image = RawImage::new(4096, 4096, ColorDepth::Depth32);

        for r in 0u16..256 {
            for g in 0u16..256 {
                for b in 0u16..256 {
                    let bx = (b & 0x0F) as u32;
                    let by = (b >> 4) as u32;
                    let x = r as u32 + 256 * bx;
                    let y = g as u32 + 256 * by;
                    image.set(x, y, ColorPixel::new_rgb(r as u8, g as u8, b as u8));
                }
            }
        }

        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/32.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_generate_1_depth() {
        let width: usize = 100;
        let height: usize = 100;
        let mut image = RawImage::new(width as u32, height as u32, ColorDepth::Depth1);

        for index in 0..(width * height) {
            image.data[index] = if (index / width + index % width) % 2 == 0 {
                ColorPixel::new_rgb(255, 0, 0)
            } else {
                ColorPixel::new_rgb(0, 255, 0)
            };
        }
        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/1.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_generate_4_depth() {
        let width: usize = 100;
        let height: usize = 100;
        let mut image = RawImage::new(width as u32, height as u32, ColorDepth::Depth4);

        let mut mock_palette = vec![];
        for _ in 0..16 {
            mock_palette.push(generate_random_color());
        }

        for index in 0..(width * height) {
            image.data[index] = mock_palette[index % 16];
        }
        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/4.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_generate_8_depth() {
        let width: usize = 100;
        let height: usize = 100;
        let mut image = RawImage::new(width as u32, height as u32, ColorDepth::Depth8);

        let mut mock_palette = vec![];
        for _ in 0..256 {
            mock_palette.push(generate_random_color());
        }

        for index in 0..(width * height) {
            image.data[index] = mock_palette[index % 256];
        }
        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/8.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_generate_16_depth() {
        let mut image = RawImage::new(4096, 4096, ColorDepth::Depth16);

        for r in 0u16..256 {
            for g in 0u16..256 {
                for b in 0u16..256 {
                    let bx = (b & 0x0F) as u32;
                    let by = (b >> 4) as u32;
                    let x = r as u32 + 256 * bx;
                    let y = g as u32 + 256 * by;
                    image.set(x, y, ColorPixel::new_rgb(r as u8, g as u8, b as u8));
                }
            }
        }

        let bmp = Bitmap::new(image);

        let path = Path::new("test/images");
        fs::create_dir_all(path).unwrap();
        let file = File::create("test/images/16.bmp").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&bmp.to_bytes()).unwrap();
    }
}
