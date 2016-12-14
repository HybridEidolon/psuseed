#[derive(Debug, RustcDecodable)]
pub struct Config {
    pub host: Option<String>, // psu-clementine.ddns.net
    pub patch_port: Option<u16>, // 11030
    pub login_port: Option<u16> // 12030
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: None,
            patch_port: None,
            login_port: None
        }
    }
}
