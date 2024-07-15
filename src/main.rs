use inquire::Select;
use serde::Deserialize;
use std::fs;
use std::io;

mod search;

#[derive(Deserialize, Debug)]
struct JsonRemangaChapter {
    content: JsonRemangaChapterContent,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaChapterContent {
    pages: Vec<Vec<JsonRemangaPage>>,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaPage {
    link: String,
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
    let manghwas = search::search_manghwa(title_input).await;
    if let Ok(result) = manghwas {
        let manghwa_titles = result.iter().map(|m| &m.title).collect();
        let answer = Select::new("Select manghwa:", manghwa_titles).prompt();
        if let Ok(user_input) = answer {
            println!("Reading manghwa {user_input}!");
            for manghwa in &result {
                if manghwa.title == *user_input {
                    println!("{manghwa:#?}");
                    let short_name = &manghwa.short_name;
                    let chapter_answer = Select::new(
                        "Select chapter",
                        manghwa.chapters.iter().map(|c| &c.number).collect(),
                    )
                    .prompt();
                    fs::create_dir(format!("{TMP_DIR}/{short_name}"))
                        .expect("This manghwa's directory already exists.");
                    if let Ok(selected_chapter) = chapter_answer {
                        for chapter in &manghwa.chapters {
                            if chapter.number == *selected_chapter {
                                let chapter_id = chapter.id;
                                let pages_json = reqwest::get(format!(
                                    "https://api.remanga.org/api/titles/chapters/{chapter_id}"
                                ))
                                .await
                                .unwrap()
                                .json::<JsonRemangaChapter>()
                                .await
                                .unwrap();
                                for pages in pages_json.content.pages {
                                    for page in pages {
                                        let page_content = reqwest::get(page.link)
                                            .await
                                            .unwrap()
                                            .text()
                                            .await
                                            .unwrap();
                                        println!("{page_content}");
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
