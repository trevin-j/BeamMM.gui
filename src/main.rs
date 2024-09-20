#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use beammm::Preset;
use eframe::egui;
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "BeamMM.gui",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

#[derive(Debug)]
struct BeamPaths {
    beamng_dir: PathBuf,
    mods_dir: PathBuf,
    beammm_dir: PathBuf,
    presets_dir: PathBuf,
}

struct StagedMod {
    mod_name: String,
    // active: bool,
    selected: bool,
}

struct App {
    beam_mod_config: beammm::game::ModCfg,
    beam_paths: BeamPaths,
    game_version: String,
    staged_mods: Vec<StagedMod>,
    presets: Vec<(String, Preset)>,
    current_preset: Option<String>,
    new_preset_name: String,
}

impl Default for App {
    // We will have to learn how to better handle these possible errors.
    fn default() -> Self {
        let beamng_dir = beammm::path::beamng_dir_default().unwrap();
        let game_version = beammm::game_version(&beamng_dir).unwrap();
        let mods_dir = beammm::path::mods_dir(&beamng_dir, &game_version).unwrap();
        let beammm_dir = beammm::path::beammm_dir().unwrap();
        let presets_dir = beammm::path::presets_dir(&beammm_dir).unwrap();
        let beam_paths = BeamPaths {
            beamng_dir,
            mods_dir,
            beammm_dir,
            presets_dir,
        };
        let mod_cfg = beammm::game::ModCfg::load_from_path(&beam_paths.mods_dir).unwrap();
        let mut staged_mods = mod_cfg.get_mods().collect::<Vec<&String>>();

        staged_mods.sort();

        let staged_mods = staged_mods
            .into_iter()
            .map(|mod_name| StagedMod {
                mod_name: mod_name.to_owned(),
                // active: mod_cfg.is_mod_active(&mod_name).unwrap(),
                selected: false,
            })
            .collect();

        let presets = Preset::list(&beam_paths.presets_dir)
            .unwrap()
            .map(|preset_name| {
                (
                    preset_name.clone(),
                    Preset::load_from_path(&preset_name, &beam_paths.presets_dir).unwrap(),
                )
            })
            .collect();
        Self {
            beam_mod_config: mod_cfg,
            beam_paths,
            game_version,
            staged_mods,
            presets,
            current_preset: None,
            new_preset_name: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title_panel").show(ctx, |ui| {
            ui.heading("BeamMM.gui");
            ui.horizontal(|ui| {
                ui.label("Version:");
                ui.label(env!("CARGO_PKG_VERSION"));
            });
            ui.horizontal(|ui| {
                ui.label("BeamNG.drive Version:");
                ui.label(&self.game_version);
            });
        });

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
                    for (preset_name, preset) in &mut self.presets {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                let text = if preset.is_enabled() {
                                    RichText::new("Enabled").color(egui::Color32::GREEN)
                                } else {
                                    RichText::new("Disabled").color(egui::Color32::RED)
                                };
                                if ui.button(text).clicked() {
                                    if preset.is_enabled() {
                                        preset.disable(&mut self.beam_mod_config).unwrap();
                                    } else {
                                        preset.enable();
                                    }
                                    preset.save_to_path(&self.beam_paths.presets_dir).unwrap();
                                    self.beam_mod_config
                                        .apply_presets(&self.beam_paths.presets_dir)
                                        .unwrap();
                                    self.beam_mod_config
                                        .save_to_path(&self.beam_paths.mods_dir)
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
                let mut preset_name: String = if let Some(preset_name) = &self.current_preset {
                    preset_name
                } else {
                    "None"
                }
                .into();
                ui.label("Edit Preset:");
                ui.menu_button(preset_name.clone(), |ui| {
                    for preset in beammm::Preset::list(&self.beam_paths.presets_dir).unwrap() {
                        if ui.button(&preset).clicked() {
                            preset_name = preset.to_owned();
                            ui.close_menu();
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_preset_name);
                        if ui.button("Create").clicked() {
                            let new_preset_name = self.new_preset_name.clone();
                            self.new_preset_name = "".into();
                            let new_preset = Preset::new(new_preset_name.clone(), vec![]);
                            new_preset
                                .save_to_path(&self.beam_paths.presets_dir)
                                .unwrap();
                            self.presets.push((new_preset_name.clone(), new_preset));
                            preset_name = new_preset_name;
                            ui.close_menu();
                        }
                    })
                });
                self.current_preset = if preset_name == "None" {
                    None
                } else {
                    Some(preset_name)
                };
            });
            let mut delete_preset = false;
            if let Some(preset_name) = &self.current_preset {
                if ui.button("Delete Preset").clicked() {
                    delete_preset = true;
                }

                // ui.label("Preset Mods");

                let preset = &mut self
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
                    preset.save_to_path(&self.beam_paths.presets_dir).unwrap();
                });
            }
            if delete_preset {
                if let Some(preset_name) = &self.current_preset {
                    Preset::delete(&preset_name, &self.beam_paths.presets_dir).unwrap();
                    self.presets.retain(|(name, _)| name != preset_name);
                }
                self.current_preset = None;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mods");
            ui.horizontal(|_| {});
            ui.horizontal(|ui| {
                if ui.button("Select All").clicked() {
                    for staged_mod in &mut self.staged_mods {
                        staged_mod.selected = true;
                    }
                }
                if ui.button("Deselect All").clicked() {
                    for staged_mod in &mut self.staged_mods {
                        staged_mod.selected = false;
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Enable Selected").clicked() {
                    for staged_mod in &self.staged_mods {
                        if staged_mod.selected {
                            self.beam_mod_config
                                .set_mod_active(&staged_mod.mod_name, true)
                                .unwrap();
                        }
                    }
                    self.beam_mod_config
                        .save_to_path(&self.beam_paths.mods_dir)
                        .unwrap();
                }
                if ui.button("Disable Selected").clicked() {
                    for staged_mod in &self.staged_mods {
                        if staged_mod.selected {
                            self.beam_mod_config
                                .set_mod_active(&staged_mod.mod_name, false)
                                .unwrap();
                        }
                    }
                    self.beam_mod_config
                        .apply_presets(&self.beam_paths.presets_dir)
                        .unwrap();
                    self.beam_mod_config
                        .save_to_path(&self.beam_paths.mods_dir)
                        .unwrap();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Add to Selected Preset").clicked() {
                    if let Some(preset_name) = &self.current_preset {
                        let preset = &mut self
                            .presets
                            .iter_mut()
                            .find(|(name, _)| name == preset_name)
                            .unwrap()
                            .1;
                        for staged_mod in &self.staged_mods {
                            if staged_mod.selected {
                                preset.add_mod(&staged_mod.mod_name);
                            }
                        }
                        preset.save_to_path(&self.beam_paths.presets_dir).unwrap();
                        self.beam_mod_config
                            .apply_presets(&self.beam_paths.presets_dir)
                            .unwrap();
                    }
                }
            });

            TableBuilder::new(ui)
                .column(Column::auto().resizable(false))
                .column(Column::exact(75.0).resizable(false))
                .column(Column::remainder())
                .header(15.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Select");
                    });
                    header.col(|ui| {
                        ui.label("Active");
                    });
                    header.col(|ui| {
                        ui.label("Mod Name");
                    });
                })
                .body(|mut body| {
                    for staged_mod in &mut self.staged_mods {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut staged_mod.selected, "");
                            });
                            row.col(|ui| {
                                let active = self
                                    .beam_mod_config
                                    .is_mod_active(&staged_mod.mod_name)
                                    .unwrap();
                                let text = if active {
                                    RichText::new("Active").color(egui::Color32::GREEN)
                                } else {
                                    RichText::new("Inactive").color(egui::Color32::RED)
                                };
                                if ui.button(text).clicked() {
                                    self.beam_mod_config
                                        .set_mod_active(&staged_mod.mod_name, !active)
                                        .unwrap();
                                    self.beam_mod_config
                                        .save_to_path(&self.beam_paths.mods_dir)
                                        .unwrap();
                                }
                            });
                            row.col(|ui| {
                                ui.label(&staged_mod.mod_name);
                            });
                        });
                    }
                });
        });
    }
}

