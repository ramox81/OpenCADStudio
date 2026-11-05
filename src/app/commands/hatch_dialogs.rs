//! HATCH / GRADIENT windows: Hatch Settings (seTtings) and the boundary
//! warning an empty-area pick shows.

use super::*;
use crate::ui::window::hatch_dialogs::{
    edit, BoundaryWarningState, HatchDialogMsg, HatchSettingsState,
};

impl OpenCADStudio {
    /// seTtings at a HATCH / GRADIENT prompt.
    pub(in crate::app) fn open_hatch_settings(&mut self) {
        let i = self.active_tab;
        let Some(settings) = self.tabs[i]
            .active_cmd
            .as_ref()
            .and_then(|command| command.hatch_settings())
        else {
            return;
        };
        let mut layers: Vec<String> = self.tabs[i]
            .scene
            .document
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect();
        layers.sort_by_key(|name| name.to_lowercase());
        self.hatch_settings_dialog = Some(HatchSettingsState::new(settings, layers));
        self.active_modal = Some(crate::app::ModalKind::HatchSettings);
    }

    /// A pick or selection with no closed boundary: the warning window, or
    /// once it is turned off the message; the prompt comes back either way.
    pub(in crate::app) fn hatch_boundary_missed(&mut self) {
        let i = self.active_tab;
        let gap = self.tabs[i]
            .active_cmd
            .as_ref()
            .and_then(|command| command.hatch_gap_tolerance())
            .unwrap_or(0.0);
        if self.hatch_boundary_warning_off {
            let _ = self.apply_cmd_result(crate::command::CmdResult::ReportError(
                crate::tr!("hatch-cmd", "no-boundary"),
            ));
            return;
        }
        self.hatch_boundary_warning = Some(BoundaryWarningState {
            gap,
            ..Default::default()
        });
        self.active_modal = Some(crate::app::ModalKind::HatchBoundaryWarning);
    }

    pub(in crate::app) fn update_hatch_dialog(&mut self, message: HatchDialogMsg) -> Task<Message> {
        match message {
            HatchDialogMsg::WarningDetails(on) => {
                if let Some(state) = self.hatch_boundary_warning.as_mut() {
                    state.details = on;
                }
            }
            HatchDialogMsg::WarningDontShow(on) => {
                if let Some(state) = self.hatch_boundary_warning.as_mut() {
                    state.dont_show = on;
                }
            }
            HatchDialogMsg::WarningClose => self.close_active_modal(),
            HatchDialogMsg::Ok | HatchDialogMsg::Boundary(_) => {
                let Some(state) = self.hatch_settings_dialog.as_mut() else {
                    return Task::none();
                };
                let mut settings = match state.read() {
                    Ok(settings) => settings,
                    Err(error) => {
                        state.error = Some(error);
                        return Task::none();
                    }
                };
                if let HatchDialogMsg::Boundary(method) = message {
                    settings.method = Some(method);
                }
                let i = self.active_tab;
                if let Some(command) = self.tabs[i].active_cmd.as_mut() {
                    command.apply_hatch_settings(settings);
                }
                self.hatch_settings_dialog = None;
                self.close_active_modal();
                return self.apply_cmd_result(crate::command::CmdResult::NeedPoint);
            }
            other => {
                if let Some(state) = self.hatch_settings_dialog.as_mut() {
                    edit(state, &other);
                }
            }
        }
        Task::none()
    }

    /// Closing the boundary warning: keep "Do not show this message again"
    /// and ask the prompt again.
    pub(in crate::app) fn close_hatch_boundary_warning(&mut self) {
        if let Some(state) = self.hatch_boundary_warning.take() {
            if state.dont_show && !self.hatch_boundary_warning_off {
                self.hatch_boundary_warning_off = true;
                self.save_config();
            }
        }
        self.hatch_reprompt();
    }

    /// The running command's prompt and options again.
    pub(in crate::app) fn hatch_reprompt(&mut self) {
        let i = self.active_tab;
        if let Some(prompt) = self.tabs[i].active_cmd.as_ref().map(|c| c.prompt()) {
            self.command_line.push_info(&prompt);
        }
        let options = self.tabs[i]
            .active_cmd
            .as_ref()
            .map(|c| c.options())
            .unwrap_or_default();
        self.command_line.set_step_options(options);
    }
}
