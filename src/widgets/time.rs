use chrono::{DateTime, Local};
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

#[derive(Debug)]
pub enum TimeMsg {
    Update,
}

pub struct TimeModel {
    time: DateTime<Local>,
}

#[relm4::component(pub)]
impl SimpleComponent for TimeModel {
    type Init = ();

    type Input = TimeMsg;
    type Output = ();

    view! {
        root = gtk::Label {
            set_css_classes: &["widget", "time"],
            #[watch]
            set_text: &model.time.format("󰅐 %H:%M").to_string()
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { time: Local::now() };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            TimeMsg::Update => {
                self.time = Local::now();
            }
        }
    }
}
