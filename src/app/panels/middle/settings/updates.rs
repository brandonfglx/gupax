use egui::{ComboBox, Label, ProgressBar, ScrollArea, TextStyle, Ui};
use log::warn;

use crate::{
    app::App,
    components::update::BINARIES_NAME,
    miscs::height_txt_before_button,
    utils::{
        constants::SPACE,
        errors::{ErrorButtons, ErrorFerris},
    },
};

impl App {
    pub fn show_settings_updates(&mut self, ui: &mut Ui) {
        #[cfg(feature = "distro")]
        {
            ui.horizontal_wrapped(|ui|{
            ui.label(RichText::new("Gupax has been installed by your distribution package manager.\nYou should update it from it.")
                    .color(ORANGE));
                });
            return;
        }
        ScrollArea::vertical().show(ui, |ui| {
            self.update_all_widget(ui);
            self.update_progress(ui);
            for name in BINARIES_NAME {
                self.binary_update_settings_widget(ui, name);
            }
        });
        // Update button + Progress bar
        // debug!("Gupax Tab | Rendering [Update] button + progress bar");
        //
        //
        // automatic updates (frequency)
        // notification for new update
        // automatic restart after automatic update
        // let height_font = ui.text_style_height(&TextStyle::Body);
        // egui::ScrollArea::vertical().show(ui, |ui| {
        //     ui.style_mut().spacing.item_spacing = [height_font, height_font].into();
        //     ui.group(|ui| {
        //         let updating = self.update.lock().unwrap().updating;
        //         ui.vertical_centered(|ui| {
        //             ui.add_space(height_font);
        //             ui.style_mut().spacing.button_padding = ui.style().spacing.button_padding * 3.0;
        //             // If [Gupax] is being built for a Linux distro,
        //             // disable built-in updating completely.
        //             #[cfg(feature = "distro")]
        //             ui.disable();
        //             #[cfg(feature = "distro")]
        //             // ui.add_sized([width, button], Button::new("Updates are disabled"))
        //             // .on_disabled_hover_text(DISTRO_NO_UPDATE);
        //             ui.button("Updates are disabled")
        //                 .on_disabled_hover_text(DISTRO_NO_UPDATE);
        //             #[cfg(not(feature = "distro"))]
        //             ui.add_enabled_ui(
        //                 !updating && *self.restart.lock().unwrap() == Restart::No,
        //                 |ui| {
        //                     #[cfg(not(feature = "distro"))]
        //                     use crate::utils::constants::GUPAX_UPDATE;

        //                     #[cfg(not(feature = "distro"))]
        //                     // if ui
        //                     //     .add_sized([width, button], Button::new("Check for updates"))
        //                     if ui
        //                         .button("Check for updates")
        //                         .on_hover_text(GUPAX_UPDATE)
        //                         .clicked()
        //                     {
        //                         use crate::components::update::Update;

        //                         Update::spawn_thread(
        //                             &self.og,
        //                             &self.state.gupax,
        //                             &self.state_path,
        //                             &self.update,
        //                             &mut self.error_state,
        //                             &self.restart,
        //                         );
        //                     }
        //                 },
        //             );
        //             ui.add_enabled_ui(updating, |ui| {
        //                 let prog = *self.update.lock().unwrap().prog.lock().unwrap();
        //                 let msg = format!(
        //                     "{}\n{}{}",
        //                     *self.update.lock().unwrap().msg.lock().unwrap(),
        //                     prog,
        //                     "%"
        //                 );
        //                 ui.label(msg);
        //                 if updating {
        //                     ui.spinner();
        //                 } else {
        //                     ui.label("...");
        //                 }
        //                 ui.add(ProgressBar::new(
        //                     self.update.lock().unwrap().prog.lock().unwrap().round() / 100.0,
        //                 ));
        //             });
        //         });
        //     });
        // });
    }
    fn update_progress(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                let updating = self.update.lock().unwrap().updating;
                ui.add_enabled_ui(updating, |ui| {
                    let prog = self.update.lock().unwrap().prog;
                    let msg = format!("{}\n{}{}", self.update.lock().unwrap().msg, prog, "%");
                    ui.label(msg);
                    if updating {
                        ui.spinner();
                    } else {
                        ui.label("...");
                    }
                    ui.add(ProgressBar::new(prog.round() / 100.0));
                });
            });
        });
    }
    fn warn_downgrade(&mut self, name: &str) -> bool {
        // TODO: warn if selected version is a downgrade from current version
        let selected_numeric_version: String = self
            .state
            .gupax
            .updates
            .selected_version_by_name(name)
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let current_numeric_version: String = self
            .binaries_version
            .version_by_name(name)
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        if let Ok(selected) = selected_numeric_version.parse::<u32>()
            && let Ok(current) = current_numeric_version.parse::<u32>()
            && selected < current
        {
            warn!("Trying to downgrade");
            let msg = if name != "gupax" {
                "You are trying to downgrade a binary. This is potentially dangerous as it is unsupported."
            } else {
                "You are trying to downgrade Gupax. This is really dangerous as you could loose the ability to upgrade it and be stuck on the old version. You would need to re-install it manually. You also could loose your saved configuration"
            };

            self.error_state
                .set(msg, ErrorFerris::Panic, ErrorButtons::Confirm);
            if self.error_state.msg == "Canceled" {
                self.error_state.msg = "".to_string();
                return false;
            }
        }
        true
    }
    fn warn_switch_beta(&mut self, name: &str) -> bool {
        if !self.binaries_version.version_by_name(name).contains("BETA")
            && self
                .state
                .gupax
                .updates
                .selected_version_by_name(name)
                .contains("BETA")
        {
            warn!("Trying to downgrade");
            let msg = "You are trying to update to a BETA. Do this only if you want to help test unstable software.";
            self.error_state
                .set(msg, ErrorFerris::Panic, ErrorButtons::Confirm);
            if self.error_state.msg == "Canceled" {
                self.error_state.msg = "".to_string();
                return false;
            }
        }
        true
    }
    fn update_all_widget(&mut self, ui: &mut Ui) {
        ui.add_space(SPACE);
        ui.horizontal(|ui|{
            ui.vertical(|ui|{
                ui.add_space(SPACE / 2.0);
        if ui.button("Update Everything").on_hover_text("Update all binaries to the latest release").clicked() {
            for name in BINARIES_NAME {
                if !(self.warn_downgrade(name) && self.warn_switch_beta(name))
                {
                    return;
                }
            }
            self.update
                .update_all(self.state.gupax.clone(), self.binaries_version.clone());
        }
            });
        ui.checkbox(&mut self.state.gupax.updates.beta, "Beta").on_hover_text("Participate in pre-release. Check only if you want to test release to come before they are stabilized. You will experience bugs.");
        });
    }
    fn binary_update_settings_widget(&mut self, ui: &mut Ui, name: &str) {
        ui.group(|ui| {
            ui.heading(name);
            ui.horizontal(|ui| {
                let version = self.binaries_version.version_by_name(name);
                ui.label("Version:    ");
                ui.label(version);
            });
            ui.horizontal(|ui| {
                self.refresh_versions_button(ui, name);
                self.update_binary_button(ui, name);
            });
            self.list_versions(ui, name);
            ui.spacing_mut().text_edit_width = ui.available_width() - SPACE;
            self.source_field(ui, name);
        });
    }
    fn refresh_versions_button(&mut self, ui: &mut Ui, name: &str) {
        if ui
            .button("Refresh")
            .on_hover_text("Refresh available versions")
            .clicked()
        {
            self.update
                .refresh_versions(vec![name.to_string()], self.state.gupax.clone());
        }
    }
    fn update_binary_button(&mut self, ui: &mut Ui, name: &str) {
        let enable = !self
            .update
            .lock()
            .unwrap()
            .releases_by_name(name)
            .is_empty();
        ui.add_enabled_ui(enable, |ui| {
            if ui
                .button("Update")
                .on_hover_text("Update to the selected version")
                .clicked()
                && self.warn_downgrade(name)
                && self.warn_switch_beta(name)
            {
                self.update.update_version(
                    vec![name.to_string()],
                    self.state.gupax.clone(),
                    self.binaries_version.clone(),
                );
            }
        });
    }
    fn list_versions(&mut self, ui: &mut Ui, name: &str) {
        let selected_version = self.state.gupax.updates.selected_version_by_name_mut(name);
        if !self
            .update
            .lock()
            .unwrap()
            .releases_by_name(name)
            .is_empty()
        {
            ComboBox::new(format!("combo_version_{name}"), "")
                .selected_text(selected_version.to_string())
                .wrap_mode(egui::TextWrapMode::Extend)
                .height(
                    12.0 * (ui.text_style_height(&TextStyle::Button)
                        + (ui.spacing().button_padding.y * 2.0)
                        + ui.spacing().item_spacing.y),
                )
                .show_ui(ui, |ui| {
                    for release in self.update.lock().unwrap().releases_by_name(name) {
                        ui.selectable_value(
                            selected_version,
                            release.to_string(),
                            release.tag_name.clone(),
                        )
                        .on_hover_text(release.body.clone());
                    }
                });
        }
    }
    fn source_field(&mut self, ui: &mut Ui, name: &str) {
        let repo = self.state.gupax.updates.source_by_name_mut(name);
        ui.horizontal(|ui|{
        ui.add_sized(
            [0.0, height_txt_before_button(ui, &TextStyle::Body)],
            Label::new("repository"),
        );
        ui.text_edit_singleline(repo).on_hover_text("repository from where the binary will be downloaded. Only github compatible API with specific name convention in releases for binaries are supported.\nExample: github.com/gupax-io/gupax");
        });
    }
}

// for gupax,p2pool,monerod,xmrig,xmrig-proxy
// refresh/update
// button select version (specific, latest, beta)
// source url
