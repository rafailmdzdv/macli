#[derive(Clone)]
pub struct MacliConf {
    pub tmp_path: String,
    pub manghwa_api: String,
    pub manghwa_cdn: String,
}

impl Default for MacliConf {
    fn default() -> Self {
        MacliConf {
            tmp_path: format!("{}/.macli", home::home_dir().unwrap().to_str().unwrap(),),
            manghwa_api: "https://api.trendymanga.com".to_string(),
            manghwa_cdn: "http://img-cdn.trendymanga.com".to_string(),
        }
    }
}
