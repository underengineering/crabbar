use gtk::prelude::*;

pub struct Widget {
    root: gtk::Label,
}

impl Widget {
    pub fn new(name: &str, text: &str) -> Self {
        let root = gtk::Label::new(Some(text));
        root.set_widget_name(name);
        root.set_css_classes(&["widget", "time"]);

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }
}
