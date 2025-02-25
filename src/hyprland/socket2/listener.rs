use futures_util::io::AsyncBufReadExt;
use relm4::gtk::{
    gio::{
        IOStreamAsyncReadWrite, InputStreamAsyncBufRead, PollableInputStream, SocketClient,
        SocketConnection, UnixSocketAddress,
    },
    prelude::*,
};
use std::{env, io, path::Path};
use thiserror::Error;

use super::events::HyprlandEvent;

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("failed to read line")]
    IOError(#[from] io::Error),
    #[error("failed to parse event")]
    EventError(#[from] anyhow::Error),
}

pub struct HyprlandListener {
    _stream: IOStreamAsyncReadWrite<SocketConnection>,
    reader: InputStreamAsyncBufRead<PollableInputStream>,
    buffer: String,
}

impl HyprlandListener {
    pub async fn connect() -> anyhow::Result<Self> {
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

        let reader = stream.input_stream().clone().into_async_buf_read(256);
        Ok(Self {
            _stream: stream,
            reader,
            buffer: String::new(),
        })
    }

    pub async fn next(&mut self) -> Result<HyprlandEvent, ListenerError> {
        self.buffer.clear();
        self.reader.read_line(&mut self.buffer).await?;

        Ok(HyprlandEvent::new(
            self.buffer.strip_suffix('\n').unwrap_or(&self.buffer),
        )?)
    }
}
