use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};
use relm4_macros::view;
use std::{cell::RefCell, rc::Rc};
use sysinfo::{MemoryRefreshKind, System};

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(system: Rc<RefCell<System>>) -> Self {
        view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                set_css_classes: &["widget", "memory"],

                append: label = &gtk::Label {
                    set_text: &Self::format(&system),
                }
            }
        }

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            label,
            async move {
                loop {
                    let usage = {
                        system
                            .borrow_mut()
                            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
                        Self::format(&system)
                    };

                    label.set_text(&usage);

                    gtk::glib::timeout_future_seconds(2).await;
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn format(system: &Rc<RefCell<System>>) -> String {
        let system = system.borrow();

        let used = system.used_memory() as f64;
        let total = system.total_memory() as f64;

        let usage = used / total * 100.0;
        format!("󰍛 {usage:.0}%")
    }
}
