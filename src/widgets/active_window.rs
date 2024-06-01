use async_broadcast::Receiver;
use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};

use crate::hyprland::socket2::events::Event;

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(mut events_rx: Receiver<Event>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "active-window"]);

        let icon = gtk::Image::new();
        root.append(&icon);

        let label = gtk::Label::new(None);
        label.set_css_classes(&["name"]);
        root.append(&label);

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(@strong root => async move {
            while let Ok(event) = events_rx.recv().await {
                if let Event::ActiveWindow { class, title } = event {
                    Self::update(&icon, &label, &class, &title);
                }
            }
        }));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn update(icon: &gtk::Image, label: &gtk::Label, class: &str, title: &str) {
        icon.set_from_icon_name(Some(class));
        label.set_text(title);
    }
}
