use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};
use sysinfo::{CpuRefreshKind, System};

pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(system: Rc<RefCell<System>>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "cpu"]);

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
                            .refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
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
        let usage = system.global_cpu_usage();
        format!(" {usage:.0}%")
    }
}
