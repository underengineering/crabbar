use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use gtk::gio::{prelude::*, SocketClient, UnixSocketAddress};
use serde::Deserialize;
use std::{env, path::Path};

async fn request(command: &str) -> anyhow::Result<String> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");
    let hyprland_instance_signature =
        env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE is not set");

    let socket_path = format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock");
    let socket_path = Path::new(&socket_path);

    let socket = SocketClient::new();
    let conn = socket
        .connect_future(&UnixSocketAddress::new(socket_path))
        .await?;

    let mut stream = conn.into_async_read_write().unwrap();
    stream.write_all(command.as_bytes()).await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}

#[derive(Deserialize)]
pub struct Keyboard {
    pub address: String,
    pub name: String,
    pub rules: String,
    pub model: String,
    pub layout: String,
    pub variant: String,
    pub options: String,
    pub active_keymap: String,
    pub main: bool,
}

#[derive(Deserialize)]
struct Devices {
    keyboards: Vec<Keyboard>,
}

pub async fn get_main_keyboard() -> anyhow::Result<Keyboard> {
    let response = request("j/devices").await?;

    let devices: Devices = serde_json::from_str(&response)?;
    let main_keyboard = devices
        .keyboards
        .into_iter()
        .find(|kb| kb.main)
        .expect("No main keyboard found");

    Ok(main_keyboard)
}

#[derive(Deserialize)]
pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub monitor: String,
    pub windows: u64,
    pub hasfullscreen: bool,
    pub lastwindow: String,
    pub lastwindowtitle: String,
}

pub async fn get_workspaces() -> anyhow::Result<Vec<Workspace>> {
    let response = request("j/workspaces").await?;

    let workspaces: Vec<Workspace> = serde_json::from_str(&response)?;
    Ok(workspaces)
}
