//! Tar Format (UStar, Unix Standard TAR)
//! https://www.subspacefield.org/~vax/tar_format.html
//! https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pax.html
//! https://www.gnu.org/software/tar/manual/html_node/Standard.html

mod header;
mod pax;
mod utils;

use std::{
    borrow::Cow,
    cmp::max,
    collections::HashMap,
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::Path,
};

use users::{Group, User, get_group_by_gid, get_user_by_uid};

use crate::file_types::archives::tar::{
    header::{TarFileType, TarHeader},
    pax::PaxEntry,
    utils::{split_path, to_field, to_fixed},
};

pub struct TarEntry {
    pub header: TarHeader,
    pub content: Vec<u8>,
}

impl TarEntry {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(self.header.as_bytes());
        result.extend(&self.content);
        // 计算需要填充的字节数，使总长度为512的倍数
        let padding_size = (512 - (self.content.len() % 512)) % 512;
        // 添加填充字节
        result.extend(vec![0u8; padding_size]);

        result
    }
}

pub struct Tar {
    pub entries: Vec<TarEntry>,
}

impl Tar {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn append_entry(&mut self, file_path: &str, root_path: Option<&str>) {
        let path = Path::new(file_path);
        let file_name = path.file_name().unwrap().to_string_lossy();
        // 排除 Mac 资源文件的干扰
        if file_name == ".DS_Store" {
            return;
        }
        // 根据 root_path 生成最终的文件路径名
        let name_with_path = if let Some(root) = root_path {
            if file_path.starts_with(root) {
                // 如果 root_path 是 file_path 的前缀，则删除前缀部分
                let trimmed = file_path.trim_start_matches(root).trim_start_matches('/');
                if trimmed.is_empty() {
                    // 如果删除前缀后为空，则使用文件名
                    file_name.to_string()
                } else {
                    trimmed.to_string()
                }
            } else {
                // 如果 root_path 不是前缀，则只使用文件名
                file_name.to_string()
            }
        } else {
            // 没有 root_path 时使用文件名
            file_name.to_string()
        };

        let file_meta = fs::symlink_metadata(path).unwrap();
        let file_type = file_meta.file_type();
        let mode = file_meta.mode();
        let uid = file_meta.uid();
        let gid = file_meta.gid();
        let mtime = file_meta.mtime();
        let (tar_file_type, size) = if file_type.is_dir() {
            (TarFileType::DirType, 0)
        } else if file_type.is_symlink() {
            (TarFileType::SymType, 0)
        } else if file_type.is_char_device() {
            (TarFileType::ChrType, 0)
        } else if file_type.is_block_device() {
            (TarFileType::BlkType, 0)
        } else if file_type.is_fifo() {
            (TarFileType::FifoType, 0)
        } else {
            (TarFileType::RegType, file_meta.size())
        };

        let uname = get_user_by_uid(uid)
            .map(|u: User| u.name().to_string_lossy().into_owned())
            .unwrap_or_default();
        let gname = get_group_by_gid(gid)
            .map(|g: Group| g.name().to_string_lossy().into_owned())
            .unwrap_or_default();

        let splitted_path = split_path(name_with_path.as_str());

        let mut tar_header = TarHeader {
            name: to_fixed(splitted_path.filename.as_bytes()),
            mode: to_field(mode as u64 & 0o7777, 8, " \0").try_into().unwrap(),
            uid: to_field(uid as u64, 8, " \0").try_into().unwrap(),
            gid: to_field(gid as u64, 8, " \0").try_into().unwrap(),
            size: to_field(size, 12, " ").try_into().unwrap(),
            mtime: to_field(max(mtime, 0) as u64, 12, " ").try_into().unwrap(),
            typeflag: tar_file_type as u8,
            uname: to_fixed(uname.as_bytes()),
            gname: to_fixed(gname.as_bytes()),
            prefix: to_fixed(splitted_path.prefix.as_bytes()),
            ..Default::default()
        };

        tar_header.calc_checksum();

        let mut pax_entry = PaxEntry {
            name: name_with_path.clone(),
            uid,
            gid,
            mtime,
            uname,
            gname,
            header_type: pax::PaxHeaderType::PerEntry,
            entries: HashMap::new(),
        };

        // 只实现超长的时候写 path
        if splitted_path.is_truncated {
            pax_entry.entries.insert(pax::PaxKey::Path, name_with_path);
        }

        if !pax_entry.entries.is_empty() {
            self.entries.push(pax_entry.to_tar_entry());
        }

        self.entries.push(TarEntry {
            header: tar_header,
            content: if file_type.is_file() {
                fs::read(path).unwrap_or_default()
            } else {
                Vec::new()
            },
        });
    }

    fn append_inner(&mut self, file_path: &str, root_path: &str) {
        let path = Path::new(file_path);
        if path.is_dir() {
            let file_path = if !file_path.ends_with("/") {
                Cow::Owned(format!("{file_path}/"))
            } else {
                Cow::Borrowed(file_path)
            };
            // 如果是目录，先添加目录本身
            self.append_entry(&file_path, Some(root_path));

            // 遍历目录下的所有文件和子目录
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let sub_path = entry.path();
                // 递归调用 append
                self.append_inner(sub_path.to_str().unwrap(), root_path);
            }
        } else {
            // 如果是文件，直接添加
            self.append_entry(file_path, Some(root_path));
        }
    }

    pub fn append(&mut self, file_path: &str) {
        // 获取文件路径的父目录部分
        let root_path = Path::new(file_path)
            .parent()
            .unwrap_or_else(|| panic!("No parent found for path {file_path}"))
            .to_str()
            .unwrap_or_else(|| panic!("Unable to convert to str for {file_path}"));

        // 使用父目录作为根路径调用 append_inner
        self.append_inner(file_path, format!("{root_path}/").as_str());
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        for entry in self.entries.iter() {
            result.extend(entry.as_bytes());
        }

        // 结尾，两个 512 字节的全 0 块
        result.extend([0u8; 1024]);

        result
    }
}

#[cfg(test)]
pub mod tests {
    use std::{fs, path::Path};

    use crate::file_types::archives::tar::Tar;

    const NORMAL_FILE_CONTENT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
            nisi ut aliquip ex ea commodo consequat.";

    pub fn generate_test_text_file() {
        // 创建测试资源目录
        let test_dir = Path::new("test/archives/resources");
        fs::create_dir_all(test_dir).unwrap();

        // 创建测试文本文件并写入 Lorem ipsum 内容
        let test_file = test_dir.join("test-text.txt");
        fs::write(&test_file, NORMAL_FILE_CONTENT).unwrap();
    }

    pub fn generate_mid_long_path_file() {
        let path = format!(
            "test/archives/resources/{}/{}",
            "a".repeat(90),
            "b".repeat(90)
        );
        // 创建测试资源目录
        let test_dir = Path::new(path.as_str());
        fs::create_dir_all(test_dir).unwrap();

        // 创建测试文本文件并写入 Lorem ipsum 内容
        let test_file = test_dir.join("test-text.txt");
        fs::write(&test_file, NORMAL_FILE_CONTENT).unwrap();
    }

    pub fn generate_very_long_path_file() {
        let path = format!(
            "test/archives/resources/{}/{}/{}/{}/{}/{}/{}/{}",
            "c".repeat(40),
            "d".repeat(40),
            "e".repeat(40),
            "f".repeat(40),
            "g".repeat(40),
            "h".repeat(40),
            "i".repeat(40),
            "j".repeat(40),
        );
        // 创建测试资源目录
        let test_dir = Path::new(path.as_str());
        fs::create_dir_all(test_dir).unwrap();

        // 创建测试文本文件并写入 Lorem ipsum 内容
        let test_file = test_dir.join("test-text.txt");
        fs::write(&test_file, NORMAL_FILE_CONTENT).unwrap();
    }

    #[test]
    pub fn test_compress_simple_text_file() {
        generate_test_text_file();
        let mut tar_file = Tar::new();
        tar_file.append_entry("test/archives/resources/test-text.txt", None);
        // 将压缩后的数据写入文件
        fs::write("test/archives/tar-simple-text.tar", tar_file.to_bytes()).unwrap();
    }

    #[test]
    pub fn test_compress_dir() {
        generate_test_text_file();
        generate_mid_long_path_file();
        generate_very_long_path_file();
        let mut tar_file = Tar::new();
        tar_file.append("test/archives/resources");
        // 将压缩后的数据写入文件
        fs::write("test/archives/tar-dir.tar", tar_file.to_bytes()).unwrap();
    }
}
