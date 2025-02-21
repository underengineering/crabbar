use relm4_macros::view;
use std::collections::HashMap;

use async_broadcast::Receiver;
use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};

use crate::hyprland::{ctl::get_main_keyboard, socket2::events::Event};

pub struct Widget {
    root: gtk::Label,
}

type LayoutNameMap = HashMap<String, String>;
impl Widget {
    pub fn new(mut events_rx: Receiver<Event>, layout_map: LayoutNameMap) -> Self {
        view! {
            root = gtk::Label {
                set_css_classes: &["widget", "layout"],
            }
        }

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            root,
            async move {
                let main_keyboard = get_main_keyboard().await.unwrap();
                let layout = main_keyboard.active_keymap;
                let layout = layout_map.get(&layout).unwrap_or(&layout);
                root.set_label(layout);

                while let Ok(event) = events_rx.recv().await {
                    if let Event::ActiveLayout { layout, .. } = event {
                        let layout = layout_map.get(&layout).unwrap_or(&layout);
                        root.set_label(layout);
                    }
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }
}
