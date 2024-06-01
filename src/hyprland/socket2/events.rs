#[derive(Clone)]
pub enum Event {
    WorkspaceV2 { id: usize, name: String },
    ActiveWindow { class: String, title: String },
    Fullscreen { status: bool },
    CreateWorkspaceV2 { id: usize, name: String },
    DestroyWorkspaceV2 { id: usize, name: String },
    RenameWorkspace { id: usize, new_name: String },
    ActiveLayout { name: String, layout: String },
}

impl Event {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let (event_name, data) = value.split_once(">>").expect("Invalid event");
        match event_name {
            "workspacev2" => {
                let (id, name) = data.split_once(',').expect("Invalid event");
                Ok(Self::WorkspaceV2 {
                    id: usize::from_str_radix(id, 16)?,
                    name: name.to_string(),
                })
            }
            "activewindow" => {
                let (class, title) = data.split_once(',').expect("Invalid event");
                Ok(Self::ActiveWindow {
                    class: class.to_string(),
                    title: title.to_string(),
                })
            }
            "createworkspacev2" => {
                let (id, name) = data.split_once(',').expect("Invalid event");
                Ok(Self::CreateWorkspaceV2 {
                    id: usize::from_str_radix(id, 16)?,
                    name: name.to_string(),
                })
            }
            "destroyworkspacev2" => {
                let (id, name) = data.split_once(',').expect("Invalid event");
                Ok(Self::DestroyWorkspaceV2 {
                    id: usize::from_str_radix(id, 16)?,
                    name: name.to_string(),
                })
            }
            "renameworkspace" => {
                let (id, new_name) = data.split_once(',').expect("Invalid event");
                Ok(Self::RenameWorkspace {
                    id: usize::from_str_radix(id, 16)?,
                    new_name: new_name.to_string(),
                })
            }
            "activelayout" => {
                let (name, layout) = data.split_once(',').expect("Invalid event");
                Ok(Self::ActiveLayout {
                    name: name.to_string(),
                    layout: layout.to_string(),
                })
            }
            _ => anyhow::bail!("Unknown event: {}", event_name),
        }
    }
}
