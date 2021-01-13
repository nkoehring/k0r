use radix_fmt::radix_36;

pub struct ShortCode {
    pub code: String,
    pub n: usize,
}

impl ShortCode {
    pub fn new(n: usize) -> ShortCode {
        let code = format!("{}", radix_36(n));
        ShortCode { code, n }
    }

    pub fn from_code(code: &str) -> ShortCode {
        let n = usize::from_str_radix(code, 36).unwrap().to_owned();
        let code = code.to_owned();
        ShortCode { code, n }
    }
}
