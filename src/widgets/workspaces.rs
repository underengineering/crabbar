use std::collections::HashMap;

use async_broadcast::Receiver;
use gtk::{
    glib::{clone, MainContext},
    prelude::*,
};

use crate::hyprland::{ctl::get_workspaces, socket2::events::Event};

type WorkspaceMap = HashMap<usize, gtk::Label>;
pub struct Widget {
    root: gtk::Box,
}

impl Widget {
    pub fn new(mut events_rx: Receiver<Event>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.set_css_classes(&["widget", "workspaces"]);

        let ctx = MainContext::default();
        ctx.spawn_local(clone!(
            #[strong]
            root,
            async move {
                let mut workspace_map = WorkspaceMap::new();

                let mut workspaces = get_workspaces().await.expect("Failed to get workspaces");
                workspaces.sort_unstable_by_key(|workspace| workspace.id);

                for workspace in &workspaces {
                    Self::add_workspace(&root, &mut workspace_map, workspace.id, &workspace.name);
                }

                let mut old_workspace: Option<gtk::Label> = None;
                while let Ok(event) = events_rx.recv().await {
                    match event {
                        Event::CreateWorkspaceV2 { id, name } => {
                            Self::add_workspace(&root, &mut workspace_map, id, &name);
                        }
                        Event::DestroyWorkspaceV2 { id, .. } => {
                            Self::remove_workspace(&root, &mut workspace_map, id);
                        }
                        Event::WorkspaceV2 { id, .. } => {
                            Self::activate_workspace(&mut old_workspace, &mut workspace_map, id);
                        }
                        _ => {}
                    }
                }
            }
        ));

        Self { root }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    fn add_workspace(root: &gtk::Box, workspace_map: &mut WorkspaceMap, id: usize, name: &str) {
        let workspace = gtk::Label::new(Some(name));
        workspace.set_css_classes(&["workspace"]);

        let mut sorted_workspaces = workspace_map.iter().collect::<Vec<_>>();
        sorted_workspaces.sort_unstable_by_key(|(id, _)| **id);

        let insert_after = sorted_workspaces
            .iter()
            .take_while(|(other_id, _)| **other_id < id)
            .last();

        if let Some((_, insert_after)) = insert_after {
            root.insert_child_after(&workspace, Some(*insert_after));
        } else if sorted_workspaces.first().is_some_and(|other| id > *other.0) {
            root.append(&workspace);
        } else {
            root.prepend(&workspace);
        }

        workspace_map.insert(id, workspace);
    }

    fn remove_workspace(root: &gtk::Box, workspace_map: &mut WorkspaceMap, id: usize) {
        let workspace = workspace_map.remove(&id).expect("Workspace not found");
        root.remove(&workspace);
    }

    fn activate_workspace(
        old_workspace: &mut Option<gtk::Label>,
        workspace_map: &mut WorkspaceMap,
        id: usize,
    ) {
        if let Some(old_workspace) = old_workspace {
            old_workspace.remove_css_class("active");
        }

        let workspace = workspace_map.get_mut(&id).expect("Workspace not found");
        workspace.add_css_class("active");

        *old_workspace = Some(workspace.clone());
    }
}
