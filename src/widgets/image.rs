use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use std::path::PathBuf;

pub struct ImageModel {}

#[relm4::component(pub)]
impl SimpleComponent for ImageModel {
    type Init = PathBuf;

    type Input = ();
    type Output = ();

    view! {
        root = gtk::Image {
            set_from_file: Some(path),
            set_css_classes: &["widget", "image"],
        }
    }

    fn init(
        path: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
