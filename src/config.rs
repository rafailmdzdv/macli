pub struct MacliConf {
    pub tmp_path: String,
}

impl Default for MacliConf {
    fn default() -> Self {
        MacliConf {
            tmp_path: format!("{}/.macli", home::home_dir().unwrap().to_str().unwrap(),),
        }
    }
}
