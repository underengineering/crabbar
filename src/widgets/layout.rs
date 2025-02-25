use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum LayoutMsg {
    ActiveLayout { layout: String },
}

pub struct LayoutInit {
    pub active_layout: String,
    pub layout_map: HashMap<String, String>,
}

pub struct LayoutModel {
    layout: String,
    layout_map: HashMap<String, String>,
}

#[relm4::component(pub)]
impl SimpleComponent for LayoutModel {
    type Init = LayoutInit;

    type Input = LayoutMsg;
    type Output = ();

    view! {
        gtk::Label {
            set_css_classes: &["widget", "layout"],

            #[watch]
            set_text: model.layout_map.get(&model.layout).unwrap_or(&model.layout),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            layout: init.active_layout,
            layout_map: init.layout_map,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            LayoutMsg::ActiveLayout { layout } => self.layout = layout,
        }
    }
}
