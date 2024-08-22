use async_broadcast::Receiver;
use gtk::{glib::MainContext, prelude::*};

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
        ctx.spawn_local(async move {
            while let Ok(event) = events_rx.recv().await {
                if let Event::ActiveWindow { class, title } = event {
                    Self::update(&icon, &label, &class, &title);
                }
            }
        });

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn update(icon: &gtk::Image, label: &gtk::Label, class: &str, title: &str) {
        icon.set_icon_name(Some(class));

        let length = title.chars().count();

        // Truncate if over 60 chars
        if length > 60 {
            // Get 60th character's index
            let truncated_length = title
                .char_indices()
                .enumerate()
                .take_while(|(a, (_, _))| *a < 60)
                .last()
                .unwrap()
                .1
                 .0;
            label.set_text(&format!("{}...", &title[..truncated_length]));
        } else {
            label.set_text(title);
        };
    }
}
