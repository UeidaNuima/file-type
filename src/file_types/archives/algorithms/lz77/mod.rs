fn to_base256_be(n: usize, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    let mut x = n;
    for i in 0..len {
        // 从尾部开始写低位字节
        out[len - 1 - i] = (x & 0xFF) as u8;
        x >>= 8;
    }
    if x != 0 {
        panic!("len not enough");
    }
    out
}

fn from_base256_be(bytes: &[u8]) -> usize {
    let mut x: usize = 0;
    for &b in bytes {
        x = (x << 8) | (b as usize);
    }
    x
}

pub struct LZ77 {
    pub n: usize,
    pub l_s: usize,
}

impl LZ77 {
    pub fn new(n: usize, l_s: usize) -> Self {
        if l_s >= n {
            panic!("l_s must be smaller then n")
        }
        Self { n, l_s }
    }

    pub fn encode(&self, mut original_bytes: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        let size_of_search = self.n - self.l_s;

        original_bytes.splice(0..0, vec![0; size_of_search]);
        // 将 original_bytes 转回只读
        let original_bytes = original_bytes;

        let mut encoded: Vec<u8> = vec![];

        let mut offset = 0;

        while let Some(window) = original_bytes.get(offset..(self.n + offset)) {
            let mut max_l = 0;
            let mut p = 0;
            for i in 0..(self.n - self.l_s) {
                // 查找起始位置
                for l in 1..self.l_s {
                    // 获取最长匹配
                    if window[i + l - 1] == window[self.n - self.l_s + l - 1] && l > max_l {
                        max_l = l;
                        p = i;
                    } else {
                        break;
                    }
                }
            }

            let c1 = to_base256_be(p, ((self.n - self.l_s) as f32).log(256_f32).ceil() as usize);
            let c2 = to_base256_be(max_l, ((self.l_s) as f32).log(256_f32).ceil() as usize);
            let c3 = window[self.n - self.l_s + max_l];

            encoded.extend(c1);
            encoded.extend(c2);
            encoded.push(c3);

            offset += max_l + 1;
        }

        let rest = original_bytes
            .get((offset + self.n - self.l_s)..)
            .unwrap_or_default()
            .to_vec();

        (encoded, rest)
    }

    pub fn decode(&self, encoded: Vec<u8>, rest: Vec<u8>) -> Vec<u8> {
        let c1_len = ((self.n - self.l_s) as f32).log(256_f32).ceil() as usize;
        let c2_len = ((self.l_s) as f32).log(256_f32).ceil() as usize;
        let c3_len = 1;
        let c_len = c1_len + c2_len + c3_len;
        let mut original_bytes: Vec<u8> = vec![0; self.n - self.l_s];
        let mut offset = 0;
        for c in encoded.chunks(c_len) {
            let p = from_base256_be(&c[0..c1_len]);
            let l = from_base256_be(&c[c1_len..(c1_len + c2_len)]);
            let s = *c.last().unwrap();
            for i in (offset + p)..(offset + p + l) {
                original_bytes.push(original_bytes[i]);
            }
            original_bytes.push(s);
            offset += l + 1;
        }
        original_bytes.extend(rest);

        original_bytes.split_off(self.n - self.l_s)
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::{BufWriter, Write},
        path::Path,
    };

    use crate::file_types::archives::algorithms::lz77::LZ77;

    #[test]
    pub fn test_encode() {
        let path = Path::new("test/archives/source");
        let data = fs::read(path).expect("无法读取 test/archives/source");

        let lz77 = LZ77::new(1024, 64);
        let (encoded, rest) = lz77.encode(data);

        let file = File::create("test/archives/lz77compressed").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&encoded).unwrap();

        let file = File::create("test/archives/lz77compressed_rest").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&rest).unwrap();
    }

    #[test]
    pub fn test_decode() {
        let path = Path::new("test/archives/lz77compressed");
        let encoded = fs::read(path).expect("无法读取 test/archives/source");
        let path = Path::new("test/archives/lz77compressed_rest");
        let rest = fs::read(path).expect("无法读取 test/archives/source");

        let lz77 = LZ77::new(1024, 64);
        let data = lz77.decode(encoded, rest);

        let file = File::create("test/archives/lz77uncompressed").unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(&data).unwrap();
    }
}
