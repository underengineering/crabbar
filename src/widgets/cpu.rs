use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

#[derive(Debug)]
pub enum CpuMsg {
    UpdateUsage { usage: f32 },
}

pub struct CpuModel {
    usage: f32,
}

#[relm4::component(pub)]
impl SimpleComponent for CpuModel {
    type Init = ();

    type Input = CpuMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "cpu"],

            append: label = &gtk::Label {
                #[watch]
                set_text: &format!(" {:.0}%", model.usage)
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { usage: 0.0 };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            CpuMsg::UpdateUsage { usage } => self.usage = usage,
        }
    }
}
