use beammm::Preset;
use eframe::egui;
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

pub fn title_panel(ctx: &egui::Context, version: &str, beamng_version: &str) {
    egui::TopBottomPanel::top("title_panel").show(ctx, |ui| {
        ui.heading("BeamMM.gui");
        ui.horizontal(|ui| {
            ui.label("Version:");
            ui.label(version);
        });
        ui.horizontal(|ui| {
            ui.label("BeamNG.drive Version:");
            ui.label(beamng_version);
        });
    });
}
