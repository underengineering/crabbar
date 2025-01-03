use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};
use sysinfo::{MemoryRefreshKind, System};

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(system: Rc<RefCell<System>>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "memory"]);

        let label = gtk::Label::new(Some(&Self::format(&system)));
        root.append(&label);

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
