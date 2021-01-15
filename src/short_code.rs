use radix_fmt::radix_36;
use std::num::ParseIntError;

pub struct ShortCode {
    pub code: String,
    pub n: usize,
}

impl ShortCode {
    pub fn new(n: usize) -> ShortCode {
        let code = format!("{}", radix_36(n));
        ShortCode { code, n }
    }

    pub fn from_code(code: &str) -> Result<ShortCode, ParseIntError> {
        let n = usize::from_str_radix(code, 36)?;
        let code = code.to_owned();
        Ok(ShortCode { code, n })
    }
}
