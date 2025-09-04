/// 将 &[u8] 转换为定长切片
pub fn to_fixed<const N: usize>(src: &[u8]) -> [u8; N] {
    let mut arr = [0u8; N]; // 全部初始化为 0
    let len = src.len().min(N); // 取最小长度，避免越界
    arr[..len].copy_from_slice(&src[..len]);
    arr
}

fn n_sevens(n: usize) -> u64 {
    let mut result: u64 = 0;
    for _ in 0..n {
        result = (result << 3) | 7; // 左移一位八进制（3个二进制位），加上7
    }
    result
}

/// 转为 tar 头里 8 字节字段的内容：7 位八进制 + 1 个 `\0`
/// 例如 0o755 -> b"0000755\0"
pub fn to_field(num: u64, length: u8, terminator: &str) -> Vec<u8> {
    let terminator_len = terminator.len() as u8;
    let num = num & n_sevens((length - terminator_len) as usize);
    // 7 宽度、前导 0 的八进制字符串
    let s = format!(
        "{:0length$o}{terminator}",
        num,
        length = (length - terminator_len) as usize
    );
    s.as_bytes().to_vec()
}

/// 路径拆分结果
#[derive(Debug)]
pub struct PathSplit {
    pub prefix: String,
    pub filename: String,
    pub is_truncated: bool, // 是否有内容被截断
}

/// 按字节长度截断字符串，确保不会在多字节字符中间截断
fn truncate_by_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut byte_count = 0;
    for (char_idx, ch) in s.char_indices() {
        let char_bytes = ch.len_utf8();
        if byte_count + char_bytes > max_bytes {
            return s[..char_idx].to_string();
        }
        byte_count += char_bytes;
    }
    s.to_string()
}

/// 将文件路径按照 / 符号拆分成文件名和 prefix
///
/// 规则：
/// - 文件名小于等于100字节
/// - prefix 小于等于 155 字节
/// - 如果路径小于等于 100 字节，则只填充文件名，prefix 留空
/// - 如果大于 100 字节，尝试从前面开始从某个 / 分开成前后两部分
/// - 如果无论如何都没有办法切分后让前后的长度都符合预期，则考虑截断
/// - 返回值会体现是否有内容被截断
pub fn split_path(path: &str) -> PathSplit {
    const MAX_FILENAME: usize = 100;
    const MAX_PREFIX: usize = 155;

    // 如果整个路径长度能在filename内放得下，就不要切分
    if path.len() <= MAX_FILENAME {
        return PathSplit {
            prefix: String::new(),
            filename: path.to_string(),
            is_truncated: false,
        };
    }

    // 找到所有 / 的位置
    let slash_positions: Vec<usize> = path.match_indices('/').map(|(i, _)| i).collect();

    if slash_positions.is_empty() {
        // 没有 / 符号，只能截断作为文件名
        return PathSplit {
            prefix: String::new(),
            filename: truncate_by_bytes(path, MAX_FILENAME),
            is_truncated: true,
        };
    }

    // 按fragment分割路径
    let fragments: Vec<&str> = path.split('/').collect();

    // 策略：prefix从头保留尽可能多的fragment，filename从尾保留尽可能多的fragment
    // 确保两部分没有重复的fragment

    let mut best_prefix_fragments = 0;
    let mut best_filename_fragments = 0;
    let mut found_valid_split = false;

    // 尝试所有可能的分割点
    for prefix_count in 0..fragments.len() {
        for filename_count in 1..=fragments.len() {
            // 确保prefix和filename不重复fragment
            if prefix_count + filename_count > fragments.len() {
                continue;
            }

            // 构建prefix（从头取prefix_count个fragment）
            let prefix_parts: Vec<&str> = fragments.iter().take(prefix_count).cloned().collect();
            let prefix_str = if prefix_parts.is_empty() {
                String::new()
            } else {
                prefix_parts.join("/")
            };

            // 构建filename（从尾取filename_count个fragment）
            let filename_parts: Vec<&str> = fragments
                .iter()
                .rev()
                .take(filename_count)
                .rev()
                .cloned()
                .collect();
            let filename_str = filename_parts.join("/");

            // 检查长度限制
            if prefix_str.len() <= MAX_PREFIX && filename_str.len() <= MAX_FILENAME {
                // 找到了有效的分割，选择保留最多fragment的组合
                if !found_valid_split
                    || (prefix_count + filename_count
                        > best_prefix_fragments + best_filename_fragments)
                {
                    best_prefix_fragments = prefix_count;
                    best_filename_fragments = filename_count;
                    found_valid_split = true;
                }
            }
        }
    }

    if found_valid_split {
        // 构建最佳结果
        let prefix_parts: Vec<&str> = fragments
            .iter()
            .take(best_prefix_fragments)
            .cloned()
            .collect();
        let prefix_str = if prefix_parts.is_empty() {
            String::new()
        } else {
            prefix_parts.join("/")
        };

        let filename_parts: Vec<&str> = fragments
            .iter()
            .rev()
            .take(best_filename_fragments)
            .rev()
            .cloned()
            .collect();
        let filename_str = filename_parts.join("/");

        let is_truncated = best_prefix_fragments + best_filename_fragments < fragments.len();

        return PathSplit {
            prefix: prefix_str,
            filename: filename_str,
            is_truncated,
        };
    }

    // 如果没有找到有效分割，尝试截断单个fragment
    // 优先尝试截断最后一个fragment作为filename
    let last_fragment = fragments.last().unwrap();
    if last_fragment.len() > MAX_FILENAME {
        // 截断最后一个fragment
        let truncated_filename = truncate_by_bytes(last_fragment, MAX_FILENAME);

        // 尝试保留前面的fragment作为prefix
        if fragments.len() > 1 {
            let prefix_fragments: Vec<&str> = fragments
                .iter()
                .take(fragments.len() - 1)
                .cloned()
                .collect();
            let prefix_str = prefix_fragments.join("/");

            if prefix_str.len() <= MAX_PREFIX {
                return PathSplit {
                    prefix: prefix_str,
                    filename: truncated_filename,
                    is_truncated: true,
                };
            }
        }

        // 如果prefix也太长，只保留截断的filename
        return PathSplit {
            prefix: String::new(),
            filename: truncated_filename,
            is_truncated: true,
        };
    }

    // 最后的备选方案：截断整个路径作为文件名
    PathSplit {
        prefix: String::new(),
        filename: truncate_by_bytes(path, MAX_FILENAME),
        is_truncated: true,
    }
}

#[cfg(test)]
pub mod tests {
    use crate::file_types::archives::tar::utils::{split_path, to_field};

    #[test]
    pub fn test_mode() {
        assert_eq!(
            String::from_utf8(to_field(0o755, 8, " \0")).unwrap(),
            "000755 \0"
        );
    }

    #[test]
    pub fn test_split_path_simple_case() {
        // 测试简单路径，如果 filename 能放下，就不要做任何分割
        let result = split_path("short/path.txt");
        assert_eq!(result.prefix, "");
        assert_eq!(result.filename, "short/path.txt");
        assert!(!result.is_truncated);
    }

    #[test]
    pub fn test_split_path_within_limits() {
        // 测试在限制范围内的较长路径
        let long_path = "a".repeat(50) + "/" + &"b".repeat(50) + ".txt";
        let result = split_path(&long_path);
        assert_eq!(result.prefix, "a".repeat(50));
        assert_eq!(result.filename, "b".repeat(50) + ".txt");
        assert!(!result.is_truncated);
    }

    #[test]
    pub fn test_split_path_filename_only_truncation() {
        // 测试只有文件名的超长路径（无分隔符）
        let very_long_path = "a".repeat(200);
        let result = split_path(&very_long_path);
        assert_eq!(result.prefix, "");
        assert_eq!(result.filename, "a".repeat(100));
        assert!(result.is_truncated);
    }

    #[test]
    pub fn test_split_path_skip_middle_segments() {
        // 测试跳过中间过长段的复杂路径分割
        // 路径结构：合适的prefix/过长的中间段/合适的filename
        let special_long_path = format!(
            "{}/{}/{}.txt",
            "a".repeat(140), // 140字节，符合prefix限制
            "b".repeat(200), // 200字节，太长，应被跳过
            "c".repeat(90)   // 90+4=94字节，符合filename限制
        );
        let result = split_path(&special_long_path);
        assert_eq!(result.prefix, "a".repeat(140));
        assert_eq!(result.filename, format!("{}.txt", "c".repeat(90)));
        assert!(result.is_truncated); // 中间段被跳过
    }

    #[test]
    pub fn test_split_path_skip_miltiple_segments() {
        // 测试跳过中间过长段的复杂路径分割
        let special_long_path = format!(
            "{}/{}/{}/{}/{}.txt",
            "a".repeat(90),
            "b".repeat(40),
            "c".repeat(60),
            "d".repeat(30),
            "e".repeat(60)
        );

        let result = split_path(&special_long_path);

        // 新算法：prefix从头保留尽可能多的fragment，filename从尾保留尽可能多的fragment
        // prefix保留前两个fragment：90个a + 40个b
        assert_eq!(
            result.prefix,
            format!("{}/{}", "a".repeat(90), "b".repeat(40))
        );
        // filename保留后两个fragment：30个d + 60个e.txt
        assert_eq!(
            result.filename,
            format!("{}/{}.txt", "d".repeat(30), "e".repeat(60))
        );
        assert!(result.is_truncated); // 中间的60个c被跳过
    }

    #[test]
    pub fn test_split_path_multibyte_character_handling() {
        // 测试多字节字符（UTF-8）的字节级截断
        // 每个中文字符占3字节，确保按字节而非字符数截断
        let chinese_path = "中".repeat(33) + ".txt";
        let result = split_path(&chinese_path);
        assert_eq!(result.prefix, "");
        // 应该截断到100字节以内
        assert!(result.filename.len() <= 100);
        assert!(result.is_truncated);

        // 测试带路径分隔符的多字节字符
        let path_with_chinese = "很长的路径名".repeat(10) + "/文件名.txt";
        let result = split_path(&path_with_chinese);
        // 验证结果的字节长度都在限制范围内
        assert!(result.prefix.len() <= 155); // MAX_PREFIX
        assert!(result.filename.len() <= 100); // MAX_FILENAME
    }
}
