lazy_static::lazy_static! {
    pub static ref MOBILE_REGEX: regex::Regex = regex::Regex::new(r"^05\d{8}$").unwrap();
}
