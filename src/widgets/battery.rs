use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};

use crate::battery::get_batteries;

const ICONS_CHARGING: [&str; 11] = [
    "󰢟 ", "󰢜 ", "󰂆 ", "󰂇 ", "󰂈 ", "󰢝 ", "󰂉 ", "󰢞 ", "󰂊 ", "󰂋 ", "󰂅 ",
];

const ICONS: [&str; 11] = [
    "󰂎 ", "󰁺 ", "󰁻 ", "󰁼 ", "󰁽 ", "󰁾 ", "󰁿 ", "󰂀 ", "󰂁 ", "󰂂 ", "󰁹 ",
];

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(battery_name: String) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "battery"]);

        let label = gtk::Label::new(Some(&Self::format(&battery_name)));
        root.append(&label);

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            label,
            async move {
                loop {
                    gtk::glib::timeout_future_seconds(10).await;
                    let capacity = Self::format(&battery_name);
                    label.set_text(&capacity);
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn format_icon(status: &str, capacity: i32) -> &'static str {
        let capacity_norm = f64::from(capacity) / 100.0;
        if status == "Charging" {
            let idx = capacity_norm * (ICONS_CHARGING.len() as f64 - 1.0);
            let idx = idx.round() as usize;
            ICONS_CHARGING[idx]
        } else {
            let idx = capacity_norm * (ICONS.len() as f64 - 1.0);
            let idx = idx.round() as usize;
            ICONS[idx]
        }
    }

    fn format(battery_name: &str) -> String {
        let batteries = get_batteries();
        let battery = &batteries[battery_name];

        let icon = Self::format_icon(&battery.status, battery.capacity);
        format!("{}{}%", icon, battery.capacity)
    }
}
