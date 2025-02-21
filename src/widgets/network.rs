use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};
use relm4_macros::view;
use sysinfo::{NetworkData, Networks};

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(network_name: String) -> Self {
        let mut networks = Networks::new_with_refreshed_list();

        view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                set_css_classes: &["widget", "network"],

                append: label = &gtk::Label {
                    set_text: &Self::format(&networks[&network_name])
                }
            }
        }

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            label,
            async move {
                loop {
                    networks.refresh(false);

                    let network = &networks[&network_name];
                    let usage = Self::format(network);

                    label.set_text(&usage);

                    gtk::glib::timeout_future_seconds(1).await;
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{size:.1}B")
        } else if size < 1024 * 1024 {
            format!("{:.1}KB", size as f64 / 1024.0)
        } else {
            format!("{:.1}MB", size as f64 / 1024.0 / 1024.0)
        }
    }

    fn format(network: &NetworkData) -> String {
        let tx = network.transmitted();
        let rx = network.received();

        format!("󰕒 {}󰇚 {}", Self::format_size(tx), Self::format_size(rx))
    }
}
