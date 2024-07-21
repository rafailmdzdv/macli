#![allow(dead_code)]

use std::{
    error::Error,
    fs,
    future::Future,
    io::{self, Write},
};

use gtk4::prelude::*;
use image::io::Reader;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use serde::Deserialize;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{config, search};

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
        let app = gtk4::Application::builder().build();
        let config = config::MacliConf::default();

        let created_tmp = fs::create_dir(&config.tmp_path);
        if created_tmp.is_err() {
            println!("{}/ already exists.", config.tmp_path);
        }

        print!("Type manghwa title: ");
        io::stdout().flush().unwrap();
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
                                    }

                                    let pages_json = reqwest::get(format!(
                                        "https://api.trendymanga.com/titles/{}/chapters/{}/pages",
                                        &manghwa_shortname, &chapter_id,
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
                                    app.connect_activate(move |appl| {
                                        let config = config::MacliConf::default();
                                        build_ui(appl, &manghwa_shortname, &chapter_id, &config);
                                    });
                                    app.run();
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

fn build_ui(
    app: &gtk4::Application,
    title_name: &String,
    chapter_id: &String,
    config: &config::MacliConf,
) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(540)
        .default_height(960)
        .title("Macli")
        .build();
    let pages = fs::read_dir(format!("{}/{}/{}", config.tmp_path, title_name, chapter_id)).unwrap();
    let list_store = gio::ListStore::new::<gtk4::StringObject>();
    let mut paths: Vec<gtk4::StringObject> = pages
        .map(|page| gtk4::StringObject::new(page.unwrap().path().to_str().unwrap()))
        .collect();
    paths.sort_by(|a, b| {
        let a_str = a.string();
        let b_str = b.string();
        let splitted_a = a_str.split('/').collect::<Vec<&str>>();
        let splitted_b = b_str.split('/').collect::<Vec<&str>>();
        splitted_a[splitted_a.len() - 1]
            .split('.')
            .collect::<Vec<&str>>()[0]
            .parse::<i16>()
            .unwrap()
            .cmp(
                &splitted_b[splitted_b.len() - 1]
                    .split('.')
                    .collect::<Vec<&str>>()[0]
                    .parse::<i16>()
                    .unwrap(),
            )
    });
    let factory = gtk4::SignalListItemFactory::new();
    list_store.extend_from_slice(&paths);
    factory.connect_setup(|_, list_item| {
        let picture = gtk4::Picture::new();
        list_item
            .downcast_ref::<gtk4::ListItem>()
            .unwrap()
            .set_child(Some(&picture));
    });
    factory.connect_bind(|_, list_item| {
        let image = list_item
            .downcast_ref::<gtk4::ListItem>()
            .unwrap()
            .child()
            .and_downcast::<gtk4::Picture>()
            .unwrap();
        let path = list_item
            .downcast_ref::<gtk4::ListItem>()
            .unwrap()
            .item()
            .and_downcast::<gtk4::StringObject>()
            .unwrap();
        let dimensions = Reader::open(path.string())
            .unwrap()
            .into_dimensions()
            .unwrap();
        image.set_file(Some(&gio::File::for_path(path.string())));
        image.set_width_request(dimensions.0 as i32);
        image.set_height_request(dimensions.1 as i32);
    });
    let selection_model = gtk4::SingleSelection::new(Some(list_store));
    let view = gtk4::ListView::new(Some(selection_model), Some(factory));
    let scrolled_window = gtk4::ScrolledWindow::builder().child(&view).build();
    window.set_child(Some(&scrolled_window));
    window.present();
}
