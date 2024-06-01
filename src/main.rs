use std::{cell::RefCell, collections::HashMap, rc::Rc};

use futures_util::task::LocalSpawnExt;
use gtk::{
    glib::{ExitCode, MainContext},
    prelude::*,
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use hyprland::socket2::listener::Listener;

mod battery;
mod hyprland;
mod widgets;

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    let system = Rc::new(RefCell::new(System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::new().with_cpu_usage()),
    )));

    let root = gtk::CenterBox::new();

    let listener = Listener::new();

    let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let workspaces = widgets::workspaces::Widget::new(listener.receiver());

    left_box.append(workspaces.widget());

    let active_window = widgets::active_window::Widget::new(listener.receiver());

    let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let net = widgets::network::Widget::new("wlp4s0".to_string());
    let battery = widgets::battery::Widget::new("BAT0".to_string());
    let cpu = widgets::cpu::Widget::new(system.clone());
    let mem = widgets::memory::Widget::new(system.clone());
    let layout = widgets::layout::Widget::new(listener.receiver(), HashMap::new());
    let time = widgets::time::Widget::new();

    right_box.append(net.widget());
    right_box.append(battery.widget());
    right_box.append(cpu.widget());
    right_box.append(mem.widget());
    right_box.append(layout.widget());
    right_box.append(time.widget());

    root.set_start_widget(Some(&left_box));
    root.set_center_widget(Some(active_window.widget()));
    root.set_end_widget(Some(&right_box));

    let ctx = MainContext::default();
    ctx.spawn_local(async move {
        if let Err(err) = listener.run().await {
            println!("socket2 listener error: {err}");
        }
    });

    window.set_child(Some(&root));
    window.present();
}

fn main() -> ExitCode {
    let app = gtk::Application::builder()
        .application_id("ru.libpcap.crabbar")
        .build();

    app.connect_activate(build_ui);

    app.run()
}
