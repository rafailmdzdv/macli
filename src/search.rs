#![allow(dead_code)]

use macli::manghwa;
use regex;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use tl;

#[derive(Deserialize, Debug)]
pub struct JsonRemangaStruct {
    msg: String,
    content: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaNext {
    props: JsonRemangaProps,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaProps {
    #[serde(rename(deserialize = "pageProps"))]
    page_props: JsonRemangaPageProps,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaPageProps {
    #[serde(rename(deserialize = "fallbackData"))]
    fallback_data: JsonRemangaFallbackData,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaFallbackData {
    content: JsonRemangaContent,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaContent {
    branches: Vec<JsonRemangaBranch>,
}

#[derive(Deserialize, Debug)]
struct JsonRemangaBranch {
    id: i32,
}

pub async fn search_manghwa(title: String) -> Result<Vec<manghwa::Manghwa>, Box<dyn Error>> {
    let json_response = reqwest::get(format!(
        "https://api.remanga.org/api/search/?query={title}&count=3&field=titles"
    ))
    .await?
    .json::<JsonRemangaStruct>()
    .await?;
    let json_manghwas = json_response.content;
    let mut manghwas: Vec<manghwa::Manghwa> = Vec::new();
    for manghwa in json_manghwas {
        let manghwa_short_name =
            serde_json::from_value::<String>(manghwa.get("dir").unwrap().clone())?;
        manghwas.push(manghwa::Manghwa {
            id: serde_json::from_value(manghwa.get("id").unwrap().clone())?,
            title: serde_json::from_value(manghwa.get("main_name").unwrap().clone())?,
            short_name: manghwa_short_name.clone(),
            chapters: acquire_chapters(manghwa_short_name.clone()).await,
        });
    }
    Ok(manghwas)
}

async fn acquire_chapters(manghwa_short_name: String) -> Vec<manghwa::Chapter> {
    let manghwa_page = reqwest::get(format!("https://remanga.org/manga/{manghwa_short_name}",))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let dom = tl::parse(manghwa_page.as_str(), tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let elements = dom.query_selector("script").unwrap();

    let mut chapters: Vec<manghwa::Chapter> = Vec::new();
    for element in elements {
        let tag = element.get(parser).unwrap().as_tag().unwrap();
        let tag_id = tag.attributes().id();
        if let Some(id) = tag_id {
            if id == "__NEXT_DATA__" {
                let raw_json = regex::Regex::new(r"<script.*>(.*)</script>")
                    .unwrap()
                    .replace(tag.raw().try_as_utf8_str().unwrap(), "$1");
                let json: JsonRemangaNext = serde_json::from_str(raw_json.as_ref()).unwrap();
                let branch_id = json.props.page_props.fallback_data.content.branches[0].id;
                let chapters_json = reqwest::get(format!(
                    "https://api.remanga.org/api/titles/chapters/?branch_id={branch_id}&ordering=-index&count=1000"
                ))
                    .await.unwrap()
                    .json::<JsonRemangaStruct>()
                    .await.unwrap();
                for chapter in chapters_json.content {
                    chapters.push(manghwa::Chapter {
                        id: serde_json::from_value(chapter.get("id").unwrap().clone()).unwrap(),
                        number: serde_json::from_value::<String>(
                            chapter.get("chapter").unwrap().clone(),
                        )
                        .unwrap(),
                    })
                }
            }
        }
    }

    chapters
}
