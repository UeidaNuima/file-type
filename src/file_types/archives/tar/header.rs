use std::{mem, slice};

use crate::file_types::archives::tar::utils::{to_field, to_fixed};

#[repr(u8)]
#[expect(clippy::enum_variant_names)]
pub enum TarFileType {
    /// regular file
    RegType = b'0',
    /// regular file (old)
    #[expect(dead_code)]
    ARegType = b'\0',
    /// link
    #[expect(dead_code)]
    LnkType = b'1',
    /// reserved
    /// symlink
    SymType = b'2',
    /// character special
    ChrType = b'3',
    /// block special
    BlkType = b'4',
    /// directory
    DirType = b'5',
    /// FIFO special
    FifoType = b'6',
    /// reserved
    #[expect(dead_code)]
    ContType = b'7',
    /// Extended header referring to the next file in the archive
    XhdType = b'x',
    /// Global extended header
    XglType = b'g',
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TarHeader {
    /// 名称，字符串，左对齐
    pub name: [u8; 100], // 0
    /// 文件 mode，八进制字符串，右对齐，空格 + null 结束
    pub mode: [u8; 8], // 100
    /// uid，八进制字符串，右对齐，空格 + null 结束
    pub uid: [u8; 8], // 108
    /// gid，八进制字符串，右对齐，空格 + null 结束
    pub gid: [u8; 8], // 116
    /// 文件大小，八进制字符串，右对齐，**空格**结束
    pub size: [u8; 12], // 124
    /// 文件修改时间，八进制字符串，右对齐，**空格**结束
    pub mtime: [u8; 12], // 136
    /// 校验
    ///
    /// - 先将 chksum 填充为 8 个空格，然后将整个 header 的 bytes 作为数字相加，至少保留 17 位精度
    /// - 将这个结果作为八进制字符串填充，右对齐，**null + 空格**结束（和其他字段都不一样）
    pub chksum: [u8; 8], // 148
    /// 文件类型
    pub typeflag: u8, // 156
    /// 链接文件名，当文件类型为软链或硬链时有值，字符串，左对齐
    pub linkname: [u8; 100], // 157
    /// 类型标识，当前固定为 "ustar\0"
    pub magic: [u8; 6], // 257
    /// 版本（？），两位八进制数，右对齐，当前固定为 00
    pub version: [u8; 2], // 263
    /// 用户名，字符串，左对齐
    pub uname: [u8; 32], // 265
    /// 组名，字符串，左对齐
    pub gname: [u8; 32], // 297
    /// 设备主版本号，如果文件类型为设备时填充，右对齐，默认值填充全 0 ，空格 + null 结束
    pub devmajor: [u8; 8], // 329
    /// 设备次版本号，如果文件类型为设备时填充，右对齐，默认值填充全 0 ，空格 + null 结束
    pub devminor: [u8; 8], // 337
    /// 前缀，如果填充，会和名称以  / 拼在一起做完整路径，字符串，左对齐
    pub prefix: [u8; 155], // 345
    // 填充到 512 字节，固定全 null
    pub padding: [u8; 12], // 500
} // 512

impl Default for TarHeader {
    fn default() -> Self {
        Self {
            name: [0; 100],
            mode: [0; 8],
            uid: [0; 8],
            gid: [0; 8],
            size: [0; 12],
            mtime: [0; 12],
            chksum: [b' '; 8],
            typeflag: 0,
            linkname: [0; 100],
            magic: to_fixed("ustar".as_bytes()),
            version: to_fixed("00".as_bytes()),
            uname: [0; 32],
            gname: [0; 32],
            devmajor: to_fixed("000000 ".as_bytes()),
            devminor: to_fixed("000000 ".as_bytes()),
            prefix: [0; 155],
            padding: [0; 12],
        }
    }
}

impl TarHeader {
    pub fn calc_checksum(&mut self) {
        self.chksum = to_fixed(&[b' '; 8]);
        self.chksum = to_field(
            self.as_bytes().iter().map(|&n| n as u64).sum::<u64>(),
            8,
            "\0 ",
        )
        .try_into()
        .unwrap();
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let ptr = self as *const TarHeader as *const u8;
        let size = mem::size_of::<TarHeader>();
        unsafe { slice::from_raw_parts(ptr, size).to_vec() }
    }
}
