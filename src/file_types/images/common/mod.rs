pub mod image;
pub mod utils;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ColorDepth {
    Depth1 = 1,
    Depth4 = 4,
    Depth8 = 8,
    Depth16 = 16,
    Depth24 = 24,
    Depth32 = 32,
}
