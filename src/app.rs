#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::future::Future;
use std::io;

use inquire::Select;
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::search;

pub trait Application {
    fn run_cmd(&self) -> impl Future<Output = Result<(), Box<dyn Error>>>;
}

#[derive(Default)]
pub struct Macli {}

#[derive(Deserialize, Debug)]
struct TrendyMangaPage {
    id: String,
    #[serde(rename(deserialize = "chapterId"))]
    chapter_id: String,
    extension: String,
}

impl Macli {
    pub fn new() -> Self {
        Macli {}
    }
}
impl Application for Macli {
    async fn run_cmd(&self) -> Result<(), Box<dyn Error>> {
        const TMP_DIR: &str = get_tmp_dir_path();
        let created_tmp = fs::create_dir(TMP_DIR);
        if created_tmp.is_err() {
            println!("{TMP_DIR}/ already exists.");
        }
        println!("Type manghwa title:");
        let mut title_input: String = String::new();
        io::stdin().read_line(&mut title_input).unwrap();
        let manghwas = search::search_manghwa(title_input.trim().to_string()).await;
        if let Ok(result) = manghwas {
            let manghwa_titles = result.iter().map(|m| &m.title).collect();
            let answer = Select::new("Select manghwa:", manghwa_titles).prompt();
            if let Ok(user_input) = answer {
                println!("Reading manghwa {user_input}!");
                for manghwa in &result {
                    if manghwa.title == *user_input {
                        let manghwa_shortname = &manghwa.short_name;
                        let chapter_answer = Select::new(
                            "Select chapter",
                            manghwa.chapters.iter().map(|c| &c.number).collect(),
                        )
                        .prompt();

                        let created_manghwa_tmp =
                            fs::create_dir(format!("{TMP_DIR}/{manghwa_shortname}"));
                        if created_manghwa_tmp.is_err() {
                            println!("This manghwa's tmp dir already exists.");
                        }

                        if let Ok(selected_chapter) = chapter_answer {
                            for chapter in &manghwa.chapters {
                                if chapter.number == *selected_chapter {
                                    let chapter_id = &chapter.id;

                                    let chapter_tmp_path =
                                        format!("{TMP_DIR}/{manghwa_shortname}/{chapter_id}");
                                    let created_chapter_tmp = fs::create_dir(&chapter_tmp_path);
                                    if created_chapter_tmp.is_err() {
                                        println!("This chapter's directory already exists.");
                                    }

                                    let pages_json = reqwest::get(format!(
                                    "https://api.trendymanga.com/titles/{manghwa_shortname}/chapters/{chapter_id}/pages"
                                ))
                                .await
                                .unwrap()
                                .json::<Vec<TrendyMangaPage>>()
                                .await
                                .unwrap();
                                    for page in pages_json {
                                        let page_img = reqwest::get(format!(
                                            "http://img-cdn.trendymanga.com/{}/{}.{}",
                                            chapter_id, page.id, page.extension,
                                        ))
                                        .await
                                        .unwrap()
                                        .bytes()
                                        .await
                                        .unwrap();
                                        let mut page_file = File::create(format!(
                                            "{}/{}.{}",
                                            chapter_tmp_path, page.id, page.extension,
                                        ))
                                        .await
                                        .unwrap();
                                        page_file.write_all(&page_img).await.unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

const fn get_tmp_dir_path() -> &'static str {
    "tmp"
}
