use futures::StreamExt;
use inquire::Select;
use serde::Deserialize;
use std::fs;
use std::io;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

mod search;

#[derive(Deserialize, Debug)]
struct TrendyMangaPage {
    id: String,
    #[serde(rename(deserialize = "chapterId"))]
    chapter_id: String,
    extension: String,
}

#[tokio::main]
async fn main() {
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
                                    let client = reqwest::Client::new();
                                    println!("{}", page.extension);
                                    let mut page_stream = client
                                        .get(format!(
                                            "https://img-cdn.trendymanga.com/{}/{}.{}",
                                            chapter_id, page.id, page.extension,
                                        ))
                                        .timeout(Duration::from_secs(1200))
                                        .send()
                                        .await
                                        .unwrap()
                                        .bytes_stream();
                                    let mut page_file = File::create(format!(
                                        "{}/{}.{}",
                                        chapter_tmp_path, page.id, page.extension,
                                    ))
                                    .await
                                    .unwrap();
                                    while let Some(item) = page_stream.next().await {
                                        page_file.write_all(&item.unwrap()).await.unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

const fn get_tmp_dir_path() -> &'static str {
    "tmp"
}

fn get_title_chapter_tmp_dir_path(title: String, chapter_id: String) -> String {
    format!("{}/{}/{}", get_tmp_dir_path(), title, chapter_id)
}
