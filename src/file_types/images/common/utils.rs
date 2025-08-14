pub fn get_padding_num(num: u32, padding_base: u32) -> u32 {
    (padding_base - (num % padding_base)) % padding_base
}

pub fn padding_to_base(num: u32, padding_base: u32) -> u32 {
    num + get_padding_num(num, padding_base)
}
