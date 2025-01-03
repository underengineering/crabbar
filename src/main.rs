use std::{cell::RefCell, env, fs, path::PathBuf, rc::Rc};

use gtk::{
    gdk::Display,
    gio::ApplicationFlags,
    glib::{ExitCode, MainContext},
    prelude::*,
};
use gtk4_layer_shell::{Edge, LayerShell};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::{config::Config, hyprland::socket2::listener::Listener};

mod battery;
mod config;
mod hyprland;
mod pulse_wrapper;
mod widgets;

fn build_ui(app: &gtk::Application, config: &Config) {
    let window = gtk::ApplicationWindow::new(app);
    window.add_css_class("bar");

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.auto_exclusive_zone_enable();
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_namespace("crabbar");

    if let Some(margins) = &config.margins {
        if let Some(margin) = margins.left {
            window.set_margin(Edge::Left, margin);
        }
        if let Some(margin) = margins.right {
            window.set_margin(Edge::Right, margin);
        }
        if let Some(margin) = margins.top {
            window.set_margin(Edge::Top, margin);
        }
        if let Some(margin) = margins.bottom {
            window.set_margin(Edge::Bottom, margin);
        }
    }

    let system = Rc::new(RefCell::new(System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
    )));

    let mut pulse_wrapper = pulse_wrapper::PulseaudioWrapper::new();

    let listener = Listener::new();

    let root = gtk::CenterBox::new();

    let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);

    if let Some(image_path) = &config.image_path {
        let image = widgets::image::Widget::new(image_path);
        left_box.append(image.widget());
    }

    let workspaces = widgets::workspaces::Widget::new(listener.receiver());

    let center_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let active_window = widgets::active_window::Widget::new(listener.receiver());

    let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);

    let network = widgets::network::Widget::new(config.network_name.clone());
    right_box.append(network.widget());

    if let Some(battery_name) = &config.battery_name {
        let battery = widgets::battery::Widget::new(battery_name.clone());
        right_box.append(battery.widget());
    }

    let cpu = widgets::cpu::Widget::new(system.clone());
    right_box.append(cpu.widget());

    let memory = widgets::memory::Widget::new(system.clone());
    right_box.append(memory.widget());

    let sound = widgets::sound::Widget::new(pulse_wrapper.receiver());
    right_box.append(sound.widget());

    let layout = widgets::layout::Widget::new(
        listener.receiver(),
        config.layout_map.clone().unwrap_or_default(),
    );
    right_box.append(layout.widget());

    let time = widgets::time::Widget::new();
    right_box.append(time.widget());

    left_box.append(workspaces.widget());

    center_box.append(active_window.widget());

    root.set_start_widget(Some(&left_box));
    root.set_center_widget(Some(&center_box));
    root.set_end_widget(Some(&right_box));

    let ctx = MainContext::default();
    ctx.spawn_local(async move {
        if let Err(err) = listener.run().await {
            eprintln!("socket2 listener error: {err}");
        }
    });

    ctx.spawn_local(async move {
        if let Err(err) = pulse_wrapper.run().await {
            eprintln!("pulseaudio wrapper error: {err}");
        }
    });

    window.set_child(Some(&root));
    window.present();
}

fn main() -> anyhow::Result<ExitCode> {
    gtk::init()?;

    let config_path = env::var("XDG_CONFIG_HOME").map_or_else(
        |_| {
            let mut home = PathBuf::from(env::var("HOME").expect("HOME is not set"));
            home.push(".config");
            home
        },
        |x| PathBuf::from(&x),
    );

    let config_path = config_path.join("crabbar");

    env::set_current_dir(&config_path).expect("Failed to set current directory");

    let config = {
        let config_path = config_path.join("config.json");
        let data = fs::read_to_string(config_path).expect("Failed to read config");
        serde_json::from_str::<Config>(&data).expect("Failed to parse config")
    };

    let css_provider = {
        let style_path = config_path.join("style.css");
        if let Ok(data) = fs::read_to_string(style_path) {
            let provider = gtk::CssProvider::new();
            provider.load_from_string(&data);
            Some(provider)
        } else {
            None
        }
    };

    if let Some(css_provider) = css_provider {
        gtk::style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &css_provider,
            3000,
        );
    }

    let app = gtk::Application::builder()
        .application_id("ru.libpcap.crabbar")
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| build_ui(app, &config));

    Ok(app.run())
}
