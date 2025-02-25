use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

#[derive(Debug)]
pub enum NetworkMsg {
    UpdateStats { transmitted: u64, received: u64 },
}

pub struct NetworkModel {
    transmitted: u64,
    received: u64,
}

impl NetworkModel {
    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{size:.1}B")
        } else if size < 1024 * 1024 {
            format!("{:.1}KB", size as f64 / 1024.0)
        } else {
            format!("{:.1}MB", size as f64 / 1024.0 / 1024.0)
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for NetworkModel {
    type Init = ();

    type Input = NetworkMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "network"],

            append: label = &gtk::Label {
                #[watch]
                set_text: {
                    let tx = model.transmitted;
                    let rx = model.received;

                    &format!("󰕒 {}󰇚 {}", Self::format_size(tx), Self::format_size(rx))
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            transmitted: 0,
            received: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            NetworkMsg::UpdateStats {
                transmitted,
                received,
            } => {
                self.transmitted = transmitted;
                self.received = received;
            }
        }
    }
}
