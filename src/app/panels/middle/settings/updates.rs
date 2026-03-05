use egui::{
    Checkbox, ComboBox, Label, ProgressBar, RichText, ScrollArea, Separator, TextStyle, Ui,
};
use egui_commonmark::CommonMarkViewer;
use log::warn;

use crate::{
    app::App,
    components::update::BINARIES_NAME,
    miscs::height_txt_before_button,
    utils::{
        constants::SPACE,
        errors::{ErrorButtons, ErrorFerris, WarnUpdateData},
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
        ScrollArea::both().show(ui, |ui| {
            self.update_progress(ui);
            ui.horizontal(|ui| {
                self.update_all_widget(ui);
                ui.group(|ui| {
                    self.horizontal_flex_button_update(ui);
                });
            });
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for name in BINARIES_NAME {
                        self.binary_update_settings_widget(ui, name);
                    }
                });
                self.view_changelog(ui);
            });
        });
    }

    fn view_changelog(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    if let Some(release) = &self.changelog_selected {
                        ui.heading("Changelog");
                        ui.label(
                            RichText::new(format!(
                                "{}\n{}\n{}",
                                release.1,
                                release.0.tag_name,
                                release.0.published_at.date_naive(),
                            ))
                            .text_style(TextStyle::Button),
                        );
                        ScrollArea::vertical().show(ui, |ui| {
                            CommonMarkViewer::new().show(
                                ui,
                                &mut self.markdown_cache,
                                &release.0.body,
                            );
                        });
                    } else {
                        ui.heading("Changelog");
                        ui.label(
                            "Click on a refresh button and then on a version to see the changelog",
                        );
                    }
                });
            });
        });
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
        let selected_numeric_version: String = self
            .state
            .gupax
            .updates
            .selected_version_by_name(name)
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let current_numeric_version: String = self
            .binaries_version
            .version_by_name(name)
            .chars()
            .filter(|c| c.is_ascii_digit())
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

            self.error_state.set(
                msg,
                ErrorFerris::Oops,
                ErrorButtons::WarnUpdate(WarnUpdateData {
                    yes_button: "Yes downgrade".to_string(),
                    no_button: "No Abort the downgrade".to_string(),
                    name: name.to_string(),
                }),
            );
            return false;
        }
        true
    }
    pub fn warn_switch_beta(&mut self, name: &str) -> bool {
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
            self.error_state.set(
                msg,
                ErrorFerris::Panic,
                ErrorButtons::WarnUpdate(WarnUpdateData {
                    yes_button: "Yes, switch to BETA".to_string(),
                    no_button: "No, stay in STABLE releases".to_string(),
                    name: name.to_string(),
                }),
            );
            return false;
        }
        true
    }
    fn update_all_widget(&mut self, ui: &mut Ui) {
        ui.add_space(SPACE * 0.9);
        ui.group(|ui|{
            ui.vertical(|ui|{
        ui.add_space(SPACE);
        ui.horizontal(|ui|{
            ui.vertical(|ui|{
        ui.add_space(SPACE * 0.5);
                 let updating = self.update.lock().unwrap().updating;
        ui.add_enabled_ui(!updating, |ui|{
        if ui.button("Update Everything").on_hover_text("Update all binaries to the latest release").clicked() {
            self.update
                .update_all(self.state.gupax.clone(), self.binaries_version.clone(), self.restart.clone());
        }
        });
            });
        ui.add_space(SPACE );
        ui.checkbox(&mut self.state.gupax.updates.beta, "Beta").on_hover_text("Participate in pre-release. Check only if you want to test release to come before they are stabilized. You will experience bugs.");           
        });
        ui.add_space(SPACE * 0.5);
            });
        });
    }

    fn binary_update_settings_widget(&mut self, ui: &mut Ui, name: &str) {
        ui.group(|ui| {
            ui.set_min_width((ui.available_width() / 2.0) - SPACE);
            ui.heading(name);
            ui.horizontal(|ui| {
                let version = self.binaries_version.version_by_name(name);
                ui.label("Currently installed version:    ");
                ui.label(version);
            });
            ui.horizontal(|ui| {
                self.refresh_versions_button(ui, name);
                self.update_binary_button(ui, name);
            });
            self.list_versions(ui, name);
            ui.spacing_mut().text_edit_width =
                ui.available_width() / 2.0 - (ui.text_style_height(&TextStyle::Body) * 10.0);
            self.source_field(ui, name);
        });
    }
    fn refresh_versions_button(&mut self, ui: &mut Ui, name: &str) {
        let updating = self.update.lock().unwrap().updating;
        ui.add_enabled_ui(!updating, |ui| {
            if ui
                .button("Refresh")
                .on_hover_text("Refresh available versions")
                .clicked()
            {
                self.update.refresh_versions(
                    vec![name.to_string()],
                    self.state.gupax.clone(),
                    self.binaries_version.clone(),
                );
            }
        });
    }
    fn update_binary_button(&mut self, ui: &mut Ui, name: &str) {
        let enable = !self
            .update
            .lock()
            .unwrap()
            .releases_by_name(name)
            .is_empty()
            && !self.update.lock().unwrap().updating;
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
                    self.restart.clone(),
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
                        if ui
                            .selectable_value(
                                selected_version,
                                release.to_string(),
                                release.tag_name.clone(),
                            )
                            .clicked()
                        {
                            self.changelog_selected = Some((release.clone(), name.to_owned()));
                        }
                    }
                });
        }
    }
    fn source_field(&mut self, ui: &mut Ui, name: &str) {
        let repo = self.state.gupax.updates.source_by_name_mut(name);
        ui.horizontal(|ui|{
        ui.add_sized(
            [0.0, height_txt_before_button(ui, &TextStyle::Body)],
            Label::new("repository:     "),
        );
        ui.text_edit_singleline(repo).on_hover_text("repository from where the binary will be downloaded. Only github compatible API with specific name convention in releases for binaries are supported.\nExample: github.com/gupax-io/gupax");
        });
    }

    pub fn horizontal_flex_button_update(&mut self, ui: &mut Ui) {
        let notification_button = (
            Checkbox::new(
                &mut self.state.gupax.updates.notification_update,
                "Notification for new updates",
            ),
            "Checks for new updates every 24 hours and at each restart.\nDoes not apply them.\nRestart Gupax to apply changes.",
        );
        let auto_update_button = (
            Checkbox::new(
                &mut self.state.gupax.updates.automatic_update,
                "Automatic updates",
            ),
            "Apply for new updates automatically. It is done every 24h and at each restart\nRestart Gupax to apply changes.",
        );
        let auto_restart_button = (
            Checkbox::new(
                &mut self.state.gupax.updates.automatic_restart,
                "Automatic restart",
            ),
            "Gupax will be restarted at the end of an update.\nRestart Gupax to apply changes.",
        );
        let widgets = vec![notification_button, auto_update_button, auto_restart_button];
        let text_style = TextStyle::Button;
        ui.style_mut().override_text_style = Some(text_style);
        let spacing = 2.0;
        ScrollArea::horizontal().show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let width = (((ui.available_width()) / widgets.len() as f32)
                    - ((ui.style().spacing.item_spacing.x * 2.0) + spacing))
                    .max(0.0);
                let size = [width, 0.0];
                let len = widgets.iter().len();
                for (count, (widget, hover)) in widgets.into_iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add_sized(size, widget).on_hover_text(hover);
                            });
                            // add a space to prevent selectable button to be at the same line as the end of the top bar. Make it the same spacing as separators.
                            ui.add_space(spacing * 4.0);
                        });
                        if count + 1 != len {
                            ui.add(Separator::default().spacing(spacing).vertical());
                        }
                    });
                }
            });
        });
    }
}

// for gupax,p2pool,monerod,xmrig,xmrig-proxy
// refresh/update
// button select version (specific, latest, beta)
// source url
