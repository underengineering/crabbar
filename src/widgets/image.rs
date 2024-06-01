use gtk::prelude::*;
use std::path::Path;

pub struct Widget {
    root: gtk::Image,
}

impl Widget {
    pub fn new(path: &Path) -> Self {
        let root = gtk::Image::from_file(path);
        root.set_css_classes(&["widget", "image"]);

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }
}
