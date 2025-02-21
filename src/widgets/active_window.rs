use async_broadcast::Receiver;
use gtk::{glib::MainContext, prelude::*};
use relm4_macros::view;

use crate::hyprland::socket2::events::Event;

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(mut events_rx: Receiver<Event>) -> Self {
        view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                set_css_classes: &["widget", "active-window"],

                append: icon = &gtk::Image::new(),
                append: label = &gtk::Label {
                    set_css_classes: &["name"]
                },
            }
        }

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
