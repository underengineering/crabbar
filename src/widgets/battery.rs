use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

use crate::battery::{get_batteries, BatteryInfo};

const ICONS_CHARGING: [&str; 11] = [
    "󰢟 ", "󰢜 ", "󰂆 ", "󰂇 ", "󰂈 ", "󰢝 ", "󰂉 ", "󰢞 ", "󰂊 ", "󰂋 ", "󰂅 ",
];

const ICONS: [&str; 11] = [
    "󰂎 ", "󰁺 ", "󰁻 ", "󰁼 ", "󰁽 ", "󰁾 ", "󰁿 ", "󰂀 ", "󰂁 ", "󰂂 ", "󰁹 ",
];

#[derive(Debug)]
pub enum BatteryMsg {
    Update,
}

pub struct BatteryModel {
    battery_name: String,

    battery_info: BatteryInfo,
}

impl BatteryModel {
    fn format_icon(&self) -> &'static str {
        let status = &self.battery_info.status;
        let capacity = self.battery_info.capacity;

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

    fn format(&self) -> String {
        let icon = self.format_icon();
        format!("{}{}%", icon, self.battery_info.capacity)
    }
}

#[relm4::component(pub)]
impl SimpleComponent for BatteryModel {
    type Init = String;

    type Input = BatteryMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "battery"],

            append: label = &gtk::Label {
                #[watch]
                set_text: &model.format(),
            }
        }
    }

    fn init(
        battery_name: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let battery_info = get_batteries().remove(&battery_name).unwrap();
        let model = Self {
            battery_name,
            battery_info,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            BatteryMsg::Update => {
                if let Some(battery_info) = get_batteries().remove(&self.battery_name) {
                    self.battery_info = battery_info;
                }
            }
        }
    }
}
