use relm4::{
    gtk::{self, glib::WeakRef, prelude::*},
    prelude::*,
};
use std::collections::HashMap;

use crate::hyprland::Workspace;

#[derive(Debug)]
pub enum WorkspacesMsg {
    Activate { id: usize },
    Create { id: usize, name: String },
    Destroy { id: usize },
}

type WorkspaceMap = HashMap<usize, gtk::Label>;
pub struct WorkspacesModel {
    old_workspace: Option<WeakRef<gtk::Label>>,
    workspaces: WorkspaceMap,
}

impl WorkspacesModel {
    fn add_workspace(&mut self, root: &gtk::Box, id: usize, name: &str) {
        let workspace = gtk::Label::new(Some(name));
        workspace.set_css_classes(&["workspace"]);

        let mut sorted_workspaces = self.workspaces.iter().collect::<Vec<_>>();
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

        self.workspaces.insert(id, workspace);
    }

    fn remove_workspace(&mut self, root: &gtk::Box, id: usize) {
        let workspace = self.workspaces.remove(&id).expect("Workspace not found");
        root.remove(&workspace);
    }

    fn activate_workspace(&mut self, id: usize) {
        if let Some(ref old_workspace) = self.old_workspace {
            if let Some(old_workspace) = old_workspace.upgrade() {
                old_workspace.remove_css_class("active");
            }
        }

        let workspace = self.workspaces.get_mut(&id).expect("Workspace not found");
        workspace.add_css_class("active");

        self.old_workspace = Some(workspace.downgrade());
    }
}

#[relm4::component(pub)]
impl Component for WorkspacesModel {
    type Init = Vec<Workspace>;

    type Input = WorkspacesMsg;
    type Output = ();

    type CommandOutput = ();

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            set_css_classes: &["widget", "workspaces"],
        }
    }

    fn init(
        workspaces: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            old_workspace: None,
            workspaces: HashMap::new(),
        };

        for workspace in workspaces {
            model.add_workspace(&root, workspace.id, &workspace.name);
        }

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            WorkspacesMsg::Activate { id, .. } => self.activate_workspace(id),
            WorkspacesMsg::Create { id, name } => self.add_workspace(root, id, &name),
            WorkspacesMsg::Destroy { id, .. } => self.remove_workspace(root, id),
        }

        self.update_view(widgets, sender);
    }
}
