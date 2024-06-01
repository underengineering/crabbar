use async_broadcast::Receiver;
use gtk::{glib::MainContext, prelude::*};
use pulse::volume::Volume;

use crate::pulse_wrapper::{PulseaudioEvent, SinkInfo};

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(mut pulse_rx: Receiver<PulseaudioEvent>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "sound"]);

        let label = gtk::Label::new(None);
        root.append(&label);

        let ctx = MainContext::default();
        ctx.spawn_local(async move {
            let mut active_sink_index = u32::MAX;
            while let Ok(event) = pulse_rx.recv().await {
                match event {
                    PulseaudioEvent::DefaultSinkChanged(sink_info) => {
                        active_sink_index = sink_info.index;
                        Self::update_label(&label, &sink_info);
                    }
                    PulseaudioEvent::SinkUpdate { sink_info, .. }
                        if sink_info.index == active_sink_index =>
                    {
                        Self::update_label(&label, &sink_info);
                    }
                    _ => {}
                }
            }
        });

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn update_label(label: &gtk::Label, sink_info: &SinkInfo) {
        if let Some(icon) = Self::format_icon(sink_info) {
            let volume = Self::format_volume(sink_info);
            label.set_text(&format!("{icon} {volume}"));
        }
    }

    fn format_icon(sink_info: &SinkInfo) -> Option<&'static str> {
        const SPEAKER_ICONS: [&str; 3] = ["󰕿", "󰖀", "󰕾"];
        const SPEAKER_MUTED_ICON: &str = "󰖁";

        const BLUETOOTH_ICON: &str = "󰂰";
        const BLUETOOTH_MUTED_ICON: &str = "󰂲";

        if let Some(sink_name) = &sink_info.name {
            if sink_name.starts_with("bluez_") {
                if sink_info.mute {
                    Some(BLUETOOTH_MUTED_ICON)
                } else {
                    Some(BLUETOOTH_ICON)
                }
            } else if sink_info.mute {
                Some(SPEAKER_MUTED_ICON)
            } else {
                Some(SPEAKER_ICONS[sink_info.index as usize % SPEAKER_ICONS.len()])
            }
        } else {
            None
        }
    }

    fn format_volume(sink_info: &SinkInfo) -> String {
        let volume_norm = f64::from(sink_info.volume.avg().0) / f64::from(Volume::NORMAL.0);
        let volume = volume_norm * 100.0;
        format!("{volume:.0}%")
    }
}
