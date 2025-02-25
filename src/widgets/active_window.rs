use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use std::borrow::Cow;

#[derive(Debug)]
pub enum ActiveWindowMsg {
    ActiveWindow { title: String, class: String },
}

pub struct ActiveWindowModel {
    icon_name: String,
    title: String,
}

impl ActiveWindowModel {
    fn window_name(&self) -> Cow<str> {
        let length = self.title.chars().count();

        // Truncate if over 60 chars
        if length > 60 {
            // Get 60th character's index
            let truncated_length = self
                .title
                .char_indices()
                .enumerate()
                .take_while(|(a, (_, _))| *a < 60)
                .last()
                .unwrap()
                .1
                 .0;
            Cow::Owned(format!("{}...", &self.title[..truncated_length]))
        } else {
            Cow::Borrowed(&self.title)
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ActiveWindowModel {
    type Init = ();

    type Input = ActiveWindowMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "active-window"],

            append: icon = &gtk::Image {
                #[watch]
                set_icon_name: Some(&model.icon_name),
            },
            append: label = &gtk::Label {
                #[watch]
                set_text: &model.window_name(),
                set_css_classes: &["name"]
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            icon_name: String::new(),
            title: String::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ActiveWindowMsg::ActiveWindow { class, title } => {
                self.icon_name = class;
                self.title = title;
            }
        }
    }
}
