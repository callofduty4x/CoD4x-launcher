mod cod4x;
mod component;
pub mod github;
mod gui;
mod launcher;
pub mod updater_app;
use crate::launcher::{msg_box, process};
use component::{Component, ComponentUpdates, Update};
use std::sync::Arc;
use winapi::um::processthreadsapi::ExitThread;

fn get_updates() -> Vec<(Vec<Update>, Box<dyn Component>)> {
    let mut components: Vec<Box<dyn Component>> = Vec::new();

    match cod4x::CoD4xComponent::new() {
        Ok(component) => components.push(Box::new(component)),
        Err(e) => msg_box::message_box(
            format!("Error updating CoD4x:\n{}", e).as_str(),
            "CoD4x Updater",
        ),
    }
    match launcher::LauncherComponent::new() {
        Ok(component) => components.push(Box::new(component)),
        Err(e) => msg_box::message_box(
            format!("Error updating launcher:\n{}", e).as_str(),
            "CoD4x Updater",
        ),
    }

    components
        .into_iter()
        .filter_map(|component| match component.get_updates() {
            Ok(updates) if !updates.is_empty() => Some((updates, component)),
            _ => None,
        })
        .collect::<Vec<ComponentUpdates>>()
}

fn build_updates_message(
    updates: &[(Vec<Update>, Box<dyn Component>)],
    needs_elevation: bool,
) -> String {
    let updates_string = updates
        .iter()
        .map(|(updates, component)| {
            let mut component_updates = format!("{}:", component.name());
            for update in updates {
                let current_version = update
                    .current
                    .as_ref()
                    .map_or("unknown".to_string(), |v| format!("{}", v));

                component_updates += format!(
                    "\n  - {}: {} => {}",
                    update.display_name, current_version, update.upstream
                )
                .as_str();
            }
            component_updates
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "Updates available:\n {}\n\n{}Do you want to update?",
        updates_string,
        if needs_elevation {
            "The update requires administrator rights\n"
        } else {
            ""
        }
    )
}

pub fn run_updater(is_elevated: bool) -> anyhow::Result<()> {
    let updates = get_updates();

    if updates.is_empty() {
        return Ok(());
    }

    let needs_elevation = !is_elevated
        && updates.iter().any(|(component, _)| {
            component
                .iter()
                .any(|update_artifact| update_artifact.requires_elevate)
        });

    let update_message = build_updates_message(&updates, needs_elevation);
    let params = nwg::MessageParams {
        title: "CoD4x Updater",
        content: update_message.as_str(),
        buttons: nwg::MessageButtons::YesNo,
        icons: nwg::MessageIcons::Question,
    };

    if is_elevated || nwg::message(&params) == nwg::MessageChoice::Yes {
        if needs_elevation {
            process::restart(process::Privileges::Admin, Some("+set elevated 1"))?;
            unsafe { ExitThread(0) };
            return Ok(());
        }

        gui::run_gui(Arc::new(updates))?;
        if is_elevated {
            msg_box::message_box("Update installed, restart the game now.", "CoD4x Updater");
            unsafe { ExitThread(0) };
            return Ok(());
        }
    }

    Ok(())
}
