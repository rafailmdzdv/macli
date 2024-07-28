#![allow(dead_code)]

use std::{
    error::Error,
    fs,
    future::Future,
    io::{self, Write},
};

use gtk4::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use serde::Deserialize;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{config, search, ui::MacliUI};

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
        let config = config::MacliConf::default();
        let macli_ui = MacliUI::new();

        let created_tmp = fs::create_dir(&config.tmp_path);
        if created_tmp.is_err() {
            println!("{}/ already exists.", config.tmp_path);
        }

        print!("Type manghwa title: ");
        io::stdout().flush().unwrap();
        let mut title_input: String = String::new();
        io::stdin().read_line(&mut title_input).unwrap();
        let manghwas = search::search_manghwa(title_input.trim().to_string(), &config).await;
        if let Ok(result) = manghwas {
            let manghwa_titles = result.iter().map(|m| &m.title).collect();
            let answer = Select::new("Select manghwa:", manghwa_titles).prompt();
            if let Ok(user_input) = answer {
                println!("Reading manghwa {user_input}!");
                for manghwa in &result {
                    if manghwa.title == *user_input {
                        let manghwa_shortname = manghwa.short_name.clone();
                        let chapter_answer = Select::new(
                            "Select chapter",
                            manghwa.chapters.iter().map(|c| &c.number).collect(),
                        )
                        .prompt();

                        let created_manghwa_tmp =
                            fs::create_dir(format!("{}/{}", config.tmp_path, manghwa_shortname));
                        if created_manghwa_tmp.is_err() {
                            println!("This manghwa's tmp dir already exists.");
                        }

                        if let Ok(selected_chapter) = chapter_answer {
                            for chapter in &manghwa.chapters {
                                if chapter.number == *selected_chapter {
                                    let chapter_id = chapter.id.clone();

                                    let chapter_tmp_path = format!(
                                        "{}/{}/{}",
                                        config.tmp_path, manghwa_shortname, chapter_id,
                                    );
                                    let created_chapter_tmp = fs::create_dir(&chapter_tmp_path);
                                    if created_chapter_tmp.is_err() {
                                        println!("This chapter's directory already exists.");
                                        let macli_clone = macli_ui.clone();
                                        macli_ui.app.connect_activate(move |_| {
                                            macli_clone.build(&manghwa_shortname, &chapter_id);
                                        });
                                        macli_ui.app.run();
                                        break;
                                    }

                                    let pages_json = reqwest::get(format!(
                                        "{}/titles/{}/chapters/{}/pages",
                                        config.manghwa_api, manghwa_shortname, chapter_id,
                                    ))
                                    .await
                                    .unwrap()
                                    .json::<Vec<TrendyMangaPage>>()
                                    .await
                                    .unwrap();
                                    let style =
                                        ProgressStyle::with_template("{bar:50.green} {pos}/{len}")
                                            .unwrap()
                                            .progress_chars("##-");
                                    let bar = ProgressBar::new(pages_json.len() as u64);
                                    bar.set_style(style);
                                    for (idx, page) in pages_json.iter().enumerate() {
                                        let page_img = reqwest::get(format!(
                                            "{}/{}/{}.{}",
                                            config.manghwa_cdn, chapter_id, page.id, page.extension,
                                        ))
                                        .await
                                        .unwrap()
                                        .bytes()
                                        .await
                                        .unwrap();
                                        let mut page_file = File::create(format!(
                                            "{}/{}.{}",
                                            chapter_tmp_path,
                                            idx + 1,
                                            page.extension,
                                        ))
                                        .await
                                        .unwrap();
                                        page_file.write_all(&page_img).await.unwrap();
                                        bar.inc(1);
                                    }
                                    bar.finish_with_message("Done.");
                                    let macli_clone = macli_ui.clone();
                                    macli_ui.app.connect_activate(move |_| {
                                        macli_clone.build(&manghwa_shortname, &chapter_id);
                                    });
                                    macli_ui.app.run();
                                    break;
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
