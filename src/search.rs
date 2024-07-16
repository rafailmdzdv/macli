#![allow(dead_code)]

use macli::manghwa;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct TrendyMangaSearchResponse {
    titles: Vec<TrendyMangaTitle>,
}

#[derive(Deserialize, Debug)]
struct TrendyMangaTitle {
    id: String,
    #[serde(rename(deserialize = "russianName"))]
    russian_name: String,
    #[serde(rename(deserialize = "urlName"))]
    url_name: String,
}

#[derive(Debug)]
struct TrendyMangaChapter {
    id: String,
    number: String,
}

impl<'de> Deserialize<'de> for TrendyMangaChapter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fields = serde_json::Value::deserialize(deserializer)?;
        Ok(TrendyMangaChapter {
            id: fields.get("id").unwrap().to_string(),
            number: fields.get("number").unwrap().to_string(),
        })
    }
}

pub async fn search_manghwa(title: String) -> Result<Vec<manghwa::Manghwa>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let json_body = format!(
        r#"{{
        "author": "",
        "name": "{title}",
        "limit": 3,
        "page": 1,
        "direction": "desc",
        "artist": "",
        "publisher": "",
        "sortBy": "CREATED_AT"
    }}"#
    );
    let json_response = client
        .post("https://api.trendymanga.com/titles/search")
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()
        .await?
        .json::<TrendyMangaSearchResponse>()
        .await?;
    let json_manghwas = json_response.titles;
    let mut manghwas: Vec<manghwa::Manghwa> = Vec::new();
    for manghwa in json_manghwas {
        manghwas.push(manghwa::Manghwa {
            id: manghwa.id,
            title: manghwa.russian_name,
            short_name: manghwa.url_name.clone(),
            chapters: acquire_chapters(manghwa.url_name.clone()).await,
        });
    }
    Ok(manghwas)
}

async fn acquire_chapters(manghwa_short_name: String) -> Vec<manghwa::Chapter> {
    let json_response = reqwest::get(format!(
        "https://api.trendymanga.com/titles/{manghwa_short_name}/chapters",
    ))
    .await
    .unwrap()
    .json::<Vec<TrendyMangaChapter>>()
    .await
    .unwrap();
    let mut chapters = Vec::new();
    for chapter in json_response {
        chapters.push(manghwa::Chapter {
            id: chapter.id.replace('"', ""),
            number: chapter.number,
        });
    }

    chapters
}
