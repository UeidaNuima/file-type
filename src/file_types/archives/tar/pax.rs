use std::{cmp::max, collections::HashMap};

use crate::file_types::archives::tar::{
    TarEntry, TarHeader,
    header::TarFileType,
    utils::{split_path, to_field, to_fixed},
};

#[derive(strum::Display, Eq, PartialEq, Hash)]
#[strum(serialize_all = "lowercase")]
#[expect(dead_code)]
pub enum PaxKey {
    Atime,
    Charset,
    Comment,
    Gid,
    Gname,
    Hdrcharset,
    Linkpath,
    Mtime,
    Path,
    Size,
    Uid,
    Uname,
}

pub enum PaxHeaderType {
    PerEntry,
    #[expect(dead_code)]
    Global,
}

pub struct PaxEntry {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub mtime: i64,
    pub uname: String,
    pub gname: String,
    pub header_type: PaxHeaderType,
    pub entries: HashMap<PaxKey, String>,
}

impl PaxEntry {
    fn entries_to_content(&self) -> String {
        let mut content = String::new();
        for (key, value) in self.entries.iter() {
            let line_content = format!("{}={}\n", key, value);
            // 计算 line_content 的长度
            let content_len = line_content.len();
            // 初始化一个长度为1的字符串(只包含数字本身长度)
            let mut line_length = "1".to_string();
            loop {
                // 计算当前完整行的长度 = line_length长度 + content长度 + 1个空格
                let total_len = line_length.len() + content_len + 1;
                // 将total_len转为字符串
                let new_length = total_len.to_string();
                // 如果新的长度字符串和之前的相同，说明已经稳定，跳出循环
                if new_length == line_length {
                    break;
                }
                line_length = new_length;
            }
            // 组合最终的行，格式为: "长度 实际内容"
            content.push_str(&format!("{} {}", line_length, line_content));
        }
        content
    }
    pub fn to_tar_entry(&self) -> TarEntry {
        let content = self.entries_to_content();
        // 使用 Path 分割路径名
        let path = std::path::Path::new(&self.name);
        // 获取文件名和父目录路径
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or_default();

        // 构建新的路径名：在原路径中间插入 PaxHeader/
        let name = if parent.is_empty() {
            format!("PaxHeader/{}", file_name)
        } else {
            format!("{}/PaxHeader/{}", parent, file_name)
        };

        let splitted_path = split_path(name.as_str());
        let mut tar_header = TarHeader {
            name: to_fixed(splitted_path.filename.as_bytes()),
            mode: to_field(0o0644, 8, " \0").try_into().unwrap(),
            uid: to_field(self.uid as u64, 8, " \0").try_into().unwrap(),
            gid: to_field(self.gid as u64, 8, " \0").try_into().unwrap(),
            mtime: to_field(max(self.mtime, 0) as u64, 12, " ")
                .try_into()
                .unwrap(),
            typeflag: match self.header_type {
                PaxHeaderType::Global => TarFileType::XglType,
                PaxHeaderType::PerEntry => TarFileType::XhdType,
            } as u8,
            size: to_field(content.len() as u64, 12, " ").try_into().unwrap(),
            uname: to_fixed(self.uname.as_bytes()),
            gname: to_fixed(self.gname.as_bytes()),
            prefix: to_fixed(splitted_path.prefix.as_bytes()),
            ..Default::default()
        };

        tar_header.calc_checksum();
        TarEntry {
            header: tar_header,
            content: content.as_bytes().to_vec(),
        }
    }
}
