#[derive(Debug)]
pub struct Manghwa {
    pub id: i32,
    pub title: String,
    pub short_name: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug)]
pub struct Chapter {
    pub id: i32,
    pub number: String,
}

#[cfg(test)]
mod tests {
    use crate::manghwa::{Chapter, Manghwa};

    #[test]
    fn test_manghwa_create() {
        let manghwa: Manghwa = Manghwa {
            id: 100,
            title: "Борьба в прямом эфире".to_string(),
            short_name: "how-to-fight".to_string(),
            chapters: vec![Chapter {
                id: 1,
                number: "1".to_string(),
            }],
        };
        assert_eq!(manghwa.id, 100);
        assert_eq!(manghwa.title, "Борьба в прямом эфире");
    }
}
