use gtk::prelude::*;
use relm4_macros::view;
use std::path::Path;

pub struct Widget {
    root: gtk::Image,
}

impl Widget {
    pub fn new(path: &Path) -> Self {
        view! {
            root = gtk::Image {
                set_from_file: Some(path),
                set_css_classes: &["widget", "image"],
            }
        }

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }
}
