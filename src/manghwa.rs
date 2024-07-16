#[derive(Debug)]
pub struct Manghwa {
    pub id: String,
    pub title: String,
    pub short_name: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug)]
pub struct Chapter {
    pub id: String,
    pub number: String,
}

#[cfg(test)]
mod tests {
    use crate::manghwa::{Chapter, Manghwa};

    #[test]
    fn test_manghwa_create() {
        let manghwa: Manghwa = Manghwa {
            id: "73cc05a3-a007-4b2b-8d18-7f2497548906".to_string(),
            title: "Борьба в прямом эфире".to_string(),
            short_name: "how-to-fight".to_string(),
            chapters: vec![Chapter {
                id: "67aa85b6-6641-410a-bc6d-a06bc58ebaf4".to_string(),
                number: "1".to_string(),
            }],
        };
        assert_eq!(manghwa.id, "73cc05a3-a007-4b2b-8d18-7f2497548906");
        assert_eq!(manghwa.title, "Борьба в прямом эфире");
    }
}
