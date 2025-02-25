use pulse::{context::State, volume::Volume};
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

use crate::pulse_wrapper::{PulseaudioEvent, SinkInfo};

#[derive(Debug)]
pub enum SoundMsg {
    Update(PulseaudioEvent),
}

pub struct SoundModel {
    active_sink_info: Option<SinkInfo>,
}

impl SoundModel {
    fn format_icon(&self) -> &'static str {
        const SPEAKER_ICONS: [&str; 3] = ["󰕿", "󰖀", "󰕾"];
        const SPEAKER_MUTED_ICON: &str = "󰖁";
        const BLUETOOTH_ICON: &str = "󰂰";
        const BLUETOOTH_MUTED_ICON: &str = "󰂲";

        let Some(ref sink_info) = self.active_sink_info else {
            return SPEAKER_ICONS[0];
        };

        if let Some(name) = sink_info.name.as_deref() {
            if name.starts_with("bluez_") {
                return if sink_info.mute {
                    BLUETOOTH_MUTED_ICON
                } else {
                    BLUETOOTH_ICON
                };
            }
        }

        if sink_info.mute {
            return SPEAKER_MUTED_ICON;
        }

        let volume_norm = f64::from(sink_info.volume.avg().0) / f64::from(Volume::NORMAL.0);
        let icon_index = (volume_norm * (SPEAKER_ICONS.len() - 1) as f64).round() as usize;
        SPEAKER_ICONS[icon_index]
    }

    fn format_volume(&self) -> String {
        if let Some(ref sink_info) = self.active_sink_info {
            let volume_norm = f64::from(sink_info.volume.avg().0) / f64::from(Volume::NORMAL.0);
            let volume = volume_norm * 100.0;
            format!("{volume:.0}%")
        } else {
            "0%".to_string()
        }
    }

    fn format(&self) -> String {
        if self.active_sink_info.is_some() {
            let icon = self.format_icon();
            let volume = self.format_volume();
            format!("{icon} {volume}")
        } else {
            "󰝞".to_string()
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for SoundModel {
    type Init = ();

    type Input = SoundMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "sound"],

            append: label = &gtk::Label {
                #[watch]
                set_text: &model.format()
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            active_sink_info: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SoundMsg::Update(PulseaudioEvent::DefaultSinkChanged(default_sink)) => {
                self.active_sink_info = Some(default_sink);
            }
            SoundMsg::Update(PulseaudioEvent::SinkUpdate { sink_info, .. })
                if self
                    .active_sink_info
                    .as_ref()
                    .is_none_or(|active_sink_info| sink_info.index == active_sink_info.index) =>
            {
                self.active_sink_info = Some(sink_info);
            }
            SoundMsg::Update(PulseaudioEvent::StateChange(State::Failed | State::Terminated)) => {
                self.active_sink_info = None;
            }
            _ => {}
        }
    }
}
