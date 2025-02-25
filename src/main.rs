use gtk4_layer_shell::{Edge, LayerShell};
use relm4::{
    gtk::{
        self,
        gdk::Display,
        glib::{timeout_future, MainContext},
        prelude::*,
    },
    prelude::*,
    view,
};
use std::{cell::RefCell, env, fs, path::PathBuf, rc::Rc, time::Duration};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::{
    config::Config,
    hyprland::{
        events::HyprlandEvent,
        listener::{HyprlandListener, ListenerError},
    },
    pulse_wrapper::PulseaudioEvent,
    widgets::{
        active_window::{ActiveWindowModel, ActiveWindowMsg},
        battery::{BatteryModel, BatteryMsg},
        cpu::{CpuModel, CpuMsg},
        image::ImageModel,
        layout::{LayoutInit, LayoutModel, LayoutMsg},
        memory::{MemoryModel, MemoryMsg},
        network::{NetworkModel, NetworkMsg},
        sound::{SoundModel, SoundMsg},
        time::{TimeModel, TimeMsg},
        workspaces::{WorkspacesModel, WorkspacesMsg},
    },
};

mod battery;
mod config;
mod hyprland;
mod pulse_wrapper;
mod widgets;

#[derive(Debug)]
enum AppMsg {
    HyprlandEvent(HyprlandEvent),
    NetworkRefresh { transmitted: u64, received: u64 },
    SystemRefresh,
    PulseaudioEvent(PulseaudioEvent),
}

struct AppModel {
    system: Rc<RefCell<System>>,

    workspaces: Controller<WorkspacesModel>,
    active_window: Controller<ActiveWindowModel>,

    network: Controller<NetworkModel>,
    battery: Option<Controller<BatteryModel>>,
    cpu: Controller<CpuModel>,
    memory: Controller<MemoryModel>,
    sound: Controller<SoundModel>,
    layout: Controller<LayoutModel>,
    time: Controller<TimeModel>,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = Config;

    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            add_css_class: "bar",

            init_layer_shell: (),
            set_layer: gtk4_layer_shell::Layer::Top,
            set_namespace: "crabbar",
            auto_exclusive_zone_enable: (),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_default_width: 999999,
            set_resizable: false,

            gtk::CenterBox {
                set_start_widget: Some(&start_widget),
                set_center_widget: Some(&active_window_widget),
                set_end_widget: Some(&end_widget),
            },
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let image_widget = if let Some(image_path) = config.image_path {
            Some(
                ImageModel::builder()
                    .launch(image_path)
                    .detach()
                    .widget()
                    .clone(),
            )
        } else {
            None
        };

        let ctx = MainContext::default();
        let workspaces = {
            let mut workspaces = ctx.block_on(hyprland::get_workspaces()).unwrap();
            workspaces.sort_unstable_by_key(|workspace| workspace.id);
            workspaces
        };

        let main_keyboard = ctx.block_on(hyprland::get_main_keyboard()).unwrap();

        let workspaces = WorkspacesModel::builder().launch(workspaces).detach();
        let workspaces_widget = workspaces.widget().clone();

        let active_window = ActiveWindowModel::builder().launch(()).detach();
        let active_window_widget = active_window.widget().clone();

        let refresh_specifics = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());
        let system = Rc::new(RefCell::new(System::new_with_specifics(refresh_specifics)));

        let network = NetworkModel::builder().launch(()).detach();
        let network_widget = network.widget().clone();

        let battery = if let Some(battery_name) = config.battery_name {
            let battery = BatteryModel::builder()
                .launch(battery_name.clone())
                .detach();
            Some(battery)
        } else {
            None
        };

        let cpu = CpuModel::builder().launch(()).detach();
        let cpu_widget = cpu.widget().clone();

        let memory = MemoryModel::builder().launch(()).detach();
        let memory_widget = memory.widget().clone();

        let sound = SoundModel::builder().launch(()).detach();
        let sound_widget = sound.widget().clone();

        let layout = LayoutModel::builder()
            .launch(LayoutInit {
                active_layout: main_keyboard.active_keymap,
                layout_map: config.layout_map.unwrap_or_default().clone(),
            })
            .detach();
        let layout_widget = layout.widget().clone();

        let time = TimeModel::builder().launch(()).detach();
        let time_widget = time.widget().clone();

        view! {
            start_widget = gtk::Box {
                set_spacing: 4,

                // append: &image_widget,
                append: &workspaces_widget,
            },
            end_widget = gtk::Box {
                set_spacing: 4,

                append: &network_widget,
                // append: &battery_widget,
                append: &cpu_widget,
                append: &memory_widget,
                append: &sound_widget,
                append: &layout_widget,
                append: &time_widget,
            }
        };

        if let Some(ref image_widget) = image_widget {
            start_widget.prepend(image_widget);
        }

        if let Some(ref battery) = battery {
            end_widget.insert_child_after(&battery.widget().clone(), Some(&network_widget));
        }

        let model = AppModel {
            system: system.clone(),

            workspaces,
            active_window,

            network,
            battery,
            cpu,
            memory,
            sound,
            layout,
            time,
        };

        let widgets = view_output!();
        ctx.spawn_local({
            let sender = sender.clone();
            async move {
                let mut networks = Networks::new();
                loop {
                    system.borrow_mut().refresh_specifics(refresh_specifics);
                    sender.input(AppMsg::SystemRefresh);

                    networks.refresh(true);

                    let network = &networks[&config.network_name];
                    sender.input(AppMsg::NetworkRefresh {
                        transmitted: network.transmitted(),
                        received: network.received(),
                    });

                    timeout_future(Duration::from_secs(1)).await;
                }
            }
        });

        let pulseaudio = pulse_wrapper::PulseaudioWrapper::new();

        ctx.spawn_local({
            let mut rx = pulseaudio.receiver();
            let sender = sender.clone();
            async move {
                while let Ok(event) = rx.recv().await {
                    sender.input(AppMsg::PulseaudioEvent(event));
                }
            }
        });

        ctx.spawn_local(async move {
            let mut rx = pulseaudio.receiver();
            loop {
                pulseaudio.connect().unwrap();

                while let Ok(event) = rx.recv().await {
                    if matches!(
                        event,
                        PulseaudioEvent::StateChange(
                            pulse::context::State::Terminated | pulse::context::State::Failed
                        )
                    ) {
                        eprintln!("PulseAudio connection terminated, reconnecting.");
                        timeout_future(Duration::from_secs(1)).await;
                        break;
                    }
                }
            }
        });

        ctx.spawn_local(async move {
            let mut listener = HyprlandListener::connect()
                .await
                .expect("failed to connect to the hyprland socket2");

            loop {
                match listener.next().await {
                    Ok(event) => {
                        sender.input(AppMsg::HyprlandEvent(event));
                    }
                    Err(ListenerError::IOError(err)) => {
                        eprintln!("Hyprland listener error: {err}");
                        break;
                    }
                    Err(ListenerError::EventError(_)) => continue,
                };
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::HyprlandEvent(HyprlandEvent::ActiveLayout { layout, .. }) => {
                self.layout.emit(LayoutMsg::ActiveLayout { layout });
            }
            AppMsg::HyprlandEvent(HyprlandEvent::ActiveWindow { class, title }) => {
                self.active_window
                    .emit(ActiveWindowMsg::ActiveWindow { class, title });
            }
            AppMsg::HyprlandEvent(HyprlandEvent::WorkspaceV2 { id, .. }) => {
                self.workspaces.emit(WorkspacesMsg::Activate { id });
            }
            AppMsg::HyprlandEvent(HyprlandEvent::CreateWorkspaceV2 { id, name }) => {
                self.workspaces.emit(WorkspacesMsg::Create { id, name });
            }
            AppMsg::HyprlandEvent(HyprlandEvent::DestroyWorkspaceV2 { id, .. }) => {
                self.workspaces.emit(WorkspacesMsg::Destroy { id });
            }
            AppMsg::NetworkRefresh {
                transmitted,
                received,
            } => {
                self.network.emit(NetworkMsg::UpdateStats {
                    transmitted,
                    received,
                });
            }
            AppMsg::SystemRefresh => {
                let system = self.system.borrow();

                if let Some(ref battery) = self.battery {
                    battery.emit(BatteryMsg::Update);
                }

                self.cpu.emit(CpuMsg::UpdateUsage {
                    usage: system.global_cpu_usage(),
                });

                self.memory.emit(MemoryMsg::UpdateStats {
                    used: system.used_memory(),
                    total: system.total_memory(),
                });

                self.time.emit(TimeMsg::Update);
            }
            AppMsg::PulseaudioEvent(event) => self.sound.emit(SoundMsg::Update(event)),
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
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

    let provider = {
        let style_path = config_path.join("style.css");
        let provider = gtk::CssProvider::new();
        provider.load_from_path(&style_path);
        provider
    };
    gtk::style_context_add_provider_for_display(&Display::default().unwrap(), &provider, 3000);

    let relm = RelmApp::new("com.github.underengineering.Crabbar");
    relm.allow_multiple_instances(false);
    relm.run::<AppModel>(config);

    Ok(())
}
