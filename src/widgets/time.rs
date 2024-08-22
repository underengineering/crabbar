use chrono::Local;
use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};

pub struct Widget {
    root: gtk::Label,
}

impl Widget {
    pub fn new() -> Self {
        let time = Self::format();
        let root = gtk::Label::new(Some(&time));
        root.set_css_classes(&["widget", "time"]);

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            root,
            async move {
                loop {
                    gtk::glib::timeout_future_seconds(1).await;
                    root.set_text(&Self::format());
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn format() -> String {
        Local::now().format("󰅐 %H:%M").to_string()
    }
}
