#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

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
}

impl Default for App {
    // We will have to learn how to better handle these possible errors.
    fn default() -> Self {
        let beamng_dir = beammm::path::beamng_dir_default().unwrap();
        let game_version = beammm::game_version(&beamng_dir).unwrap();
        let mods_dir = beammm::path::mods_dir(&beamng_dir, &game_version).unwrap();
        let presets_dir = beammm::path::presets_dir(&beamng_dir).unwrap();
        let beam_paths = BeamPaths {
            beamng_dir,
            mods_dir,
            beammm_dir: beammm::path::beammm_dir().unwrap(),
            presets_dir,
        };
        let mod_cfg = beammm::game::ModCfg::load_from_path(&beam_paths.mods_dir).unwrap();
        let staged_mods: Vec<StagedMod> = mod_cfg
            .get_mods()
            .map(|mod_name| StagedMod {
                mod_name: mod_name.to_owned(),
                // active: mod_cfg.is_mod_active(&mod_name).unwrap(),
                selected: false,
            })
            .collect();
        Self {
            beam_mod_config: mod_cfg,
            beam_paths,
            game_version,
            staged_mods,
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
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        .save_to_path(&self.beam_paths.mods_dir)
                        .unwrap();
                }
            });

            TableBuilder::new(ui)
                .column(Column::auto().resizable(false))
                .column(Column::auto().resizable(false))
                .column(Column::remainder())
                .header(20.0, |mut header| {
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
                                let text = if self
                                    .beam_mod_config
                                    .is_mod_active(&staged_mod.mod_name)
                                    .unwrap()
                                {
                                    RichText::new("Yes").color(egui::Color32::GREEN)
                                } else {
                                    RichText::new("No").color(egui::Color32::RED)
                                };
                                ui.label(text);
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

