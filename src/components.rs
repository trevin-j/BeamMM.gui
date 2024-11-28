use crate::{App, BeamPaths};
use beammm::Preset;
use eframe::egui;
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

pub fn title_panel(ctx: &egui::Context, app_data: &App) {
    egui::TopBottomPanel::top("title_panel").show(ctx, |ui| {
        ui.heading("BeamMM.gui");
        ui.horizontal(|ui| {
            ui.label("Version:");
            ui.label(&app_data.version);
        });
        ui.horizontal(|ui| {
            ui.label("BeamNG.drive Version:");
            ui.label(&app_data.beamng_version);
        });
    });
}

pub fn presets_panel(ctx: &egui::Context, app_data: &mut App) {
    egui::SidePanel::right("presets_panel").show(ctx, |ui| {
        ui.heading("Presets");
        ui.horizontal(|_| {});

        ui.label("All Presets:");
        TableBuilder::new(ui)
            .column(Column::exact(75.0))
            .column(Column::auto().resizable(false))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.add(egui::Label::new("Enabled").wrap_mode(egui::TextWrapMode::Extend));
                });
                header.col(|ui| {
                    ui.label("Preset Name");
                });
            })
            .body(|mut body| {
                for (preset_name, preset) in &mut app_data.presets {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            let text = if preset.is_enabled() {
                                RichText::new("Enabled").color(egui::Color32::GREEN)
                            } else {
                                RichText::new("Disabled").color(egui::Color32::RED)
                            };
                            if ui.button(text).clicked() {
                                if preset.is_enabled() {
                                    preset.disable(&mut app_data.beam_mod_config).unwrap();
                                } else {
                                    preset.enable();
                                }
                                preset
                                    .save_to_path(&app_data.beam_paths.presets_dir)
                                    .unwrap();
                                app_data
                                    .beam_mod_config
                                    .apply_presets(&app_data.beam_paths.presets_dir)
                                    .unwrap();
                                app_data
                                    .beam_mod_config
                                    .save_to_path(&app_data.beam_paths.mods_dir)
                                    .unwrap();
                            }
                        });
                        row.col(|ui| {
                            ui.label(&*preset_name);
                        });
                    });
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            let mut preset_name: String = if let Some(preset_name) = &app_data.current_preset {
                preset_name
            } else {
                "None"
            }
            .into();
            ui.label("Edit Preset:");
            ui.menu_button(preset_name.clone(), |ui| {
                for preset in beammm::Preset::list(&app_data.beam_paths.presets_dir).unwrap() {
                    if ui.button(&preset).clicked() {
                        preset_name = preset.to_owned();
                        ui.close_menu();
                    }
                }
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut app_data.new_preset_name);
                    if ui.button("Create").clicked() {
                        let new_preset_name = app_data.new_preset_name.clone();
                        app_data.new_preset_name = "".into();
                        let new_preset = Preset::new(new_preset_name.clone(), vec![]);
                        new_preset
                            .save_to_path(&app_data.beam_paths.presets_dir)
                            .unwrap();
                        app_data.presets.push((new_preset_name.clone(), new_preset));
                        preset_name = new_preset_name;
                        ui.close_menu();
                    }
                })
            });
            app_data.current_preset = if preset_name == "None" {
                None
            } else {
                Some(preset_name)
            };
        });
        let mut delete_preset = false;
        if let Some(preset_name) = &app_data.current_preset {
            if ui.button("Delete Preset").clicked() {
                delete_preset = true;
            }

            // ui.label("Preset Mods");

            let preset = &mut app_data
                .presets
                .iter_mut()
                .find(|(name, _)| name == preset_name)
                .unwrap()
                .1;

            let mut mods_to_remove = Vec::new();

            ui.push_id("preset_mods", |ui| {
                TableBuilder::new(ui)
                    .column(Column::exact(75.0).resizable(false))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("");
                        });
                        header.col(|ui| {
                            ui.label("Preset Mods");
                        });
                    })
                    .body(|mut body| {
                        for mod_name in preset.get_mods().clone().into_iter() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("Remove").clicked() {
                                        mods_to_remove.push(mod_name.clone());
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(&*mod_name);
                                });
                            });
                        }
                    });
                preset.remove_mods(&mods_to_remove);
                preset
                    .save_to_path(&app_data.beam_paths.presets_dir)
                    .unwrap();
            });
        }
        if delete_preset {
            if let Some(preset_name) = &app_data.current_preset {
                Preset::delete(&preset_name, &app_data.beam_paths.presets_dir).unwrap();
                app_data.presets.retain(|(name, _)| name != preset_name);
            }
            app_data.current_preset = None;
        }
    });
}
