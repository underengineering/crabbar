use async_broadcast::{InactiveReceiver, Receiver, Sender};
use futures_util::io::AsyncBufReadExt;
use gtk::gio::{prelude::*, SocketClient, UnixSocketAddress};
use std::{env, path::Path};

use super::events::Event;

pub struct Listener {
    tx: Sender<Event>,
    rx: InactiveReceiver<Event>,
}
impl Listener {
    pub fn new() -> Self {
        let (tx, rx) = async_broadcast::broadcast(4);
        Self {
            tx,
            rx: rx.deactivate(),
        }
    }

    pub fn receiver(&self) -> Receiver<Event> {
        self.rx.activate_cloned()
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");
        let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE is not set");

        let socket_path =
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock");
        let socket_path = Path::new(&socket_path);

        let socket = SocketClient::new();
        let conn = socket
            .connect_future(&UnixSocketAddress::new(socket_path))
            .await?;
        let stream = conn.into_async_read_write().unwrap();

        let mut reader = stream.input_stream().clone().into_async_buf_read(256);

        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).await?;
            if let Ok(event) = Event::new(line.strip_suffix('\n').unwrap_or(&line)) {
                self.tx.broadcast(event).await?;
            }
        }
    }
}
