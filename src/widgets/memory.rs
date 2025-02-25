use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

#[derive(Debug)]
pub enum MemoryMsg {
    UpdateStats { used: u64, total: u64 },
}

pub struct MemoryModel {
    used: u64,
    total: u64,
}

#[relm4::component(pub)]
impl SimpleComponent for MemoryModel {
    type Init = ();

    type Input = MemoryMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "cpu"],

            append: label = &gtk::Label {
                #[watch]
                set_text: {
                    let usage = model.used as f64 / model.total as f64 * 100.0;
                    &format!("󰍛 {usage:.0}%")
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { used: 0, total: 0 };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            MemoryMsg::UpdateStats { used, total } => {
                self.used = used;
                self.total = total;
            }
        }
    }
}
