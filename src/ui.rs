use std::fs;

use gtk4::prelude::*;
use image::io::Reader;

use crate::config::MacliConf;

#[derive(Clone, Default)]
pub struct MacliUI {
    pub app: gtk4::Application,
    config: MacliConf,
}

impl MacliUI {
    pub fn new() -> Self {
        MacliUI {
            app: gtk4::Application::builder().build(),
            config: MacliConf::default(),
        }
    }

    pub fn build(&self, title_name: &String, chapter_id: &String) {
        let window = gtk4::ApplicationWindow::builder()
            .application(&self.app)
            .default_width(540)
            .default_height(960)
            .title("Macli")
            .build();
        let pages = fs::read_dir(format!(
            "{}/{}/{}",
            self.config.tmp_path, title_name, chapter_id
        ))
        .unwrap();
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
}
