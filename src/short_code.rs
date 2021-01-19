use radix_fmt::radix_36;
use std::num::ParseIntError;
use uuid::Uuid;

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

/// Creates a new random UUID and encodes it as lower case hyphenated string
// see https://docs.rs/uuid/0.8.2/uuid/adapter/struct.Hyphenated.html
// in case you wonder about that Uuid::encode_buffer()
pub fn random_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}
