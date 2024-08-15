//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs;
use std::fs::File;

use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use tracing::info;

use crate::injector_app::InjectorApp as InjectorAppWindow;

mod dll_info;
mod emoji_button_widget;
mod emoji_label_widget;
mod injector_app;
mod process_selection_method;
mod utils;

fn load_system_fonts(ctx: &Context) {
    info!("Started loading system fonts");

    // Helper function to load font data by PostScript name or family name
    fn load_font_data(postscript_name: &str, fallback_family: FamilyName) -> Vec<u8> {
        let font_handle = SystemSource::new()
            //.select_by_postscript_name(postscript_name)
            .select_best_match(
                &[FamilyName::Title(postscript_name.to_owned())],
                &Properties::new(),
            )
            .or_else(|_| {
                SystemSource::new().select_best_match(&[fallback_family], &Properties::new())
            })
            .unwrap_or_else(|_| panic!("Failed to find the system font: {}", postscript_name));

        match font_handle {
            Handle::Path { path, .. } => fs::read(path).expect("Failed to read the font file"),
            Handle::Memory { bytes, .. } => bytes.to_vec(),
        }
    }

    // Load Bahnschrift font data
    let bahnschrift_font_data = load_font_data("Bahnschrift", FamilyName::SansSerif);
    info!(
        "Loaded Times New Roman font data with {} bytes",
        bahnschrift_font_data.len()
    );

    // Load Segoe UI Emoji font data
    let segoe_ui_emoji_font_data = load_font_data("Segoe UI Emoji", FamilyName::SansSerif);
    info!(
        "Loaded SimHei font data with {} bytes",
        segoe_ui_emoji_font_data.len()
    );

    // Load SimHei font data
    let simhei_font_data = load_font_data("SimHei", FamilyName::SansSerif);
    info!(
        "Loaded SimHei font data with {} bytes",
        segoe_ui_emoji_font_data.len()
    );

    // Convert the font data into FontData for egui
    let bahnscrift_font_data_obj = FontData::from_owned(bahnschrift_font_data);
    let segoi_ui_font_data_obj = FontData::from_owned(segoe_ui_emoji_font_data);
    let simhei_font_data_obj = FontData::from_owned(simhei_font_data);

    // Create FontDefinitions and add the font data
    let mut font_def = FontDefinitions::empty();
    font_def
        .font_data
        .insert("Bahnschrift".to_string(), bahnscrift_font_data_obj);
    font_def
        .font_data
        .insert("Segoe UI Emoji".to_string(), segoi_ui_font_data_obj);
    font_def
        .font_data
        .insert("SimHei".to_string(), simhei_font_data_obj);

    if let Some(vec) = font_def.families.get_mut(&FontFamily::Proportional) {
        vec.push("Bahnschrift".to_owned());
        vec.push("Segoe UI Emoji".to_owned());
        vec.push("SimHei".to_owned());
    }

    if let Some(vec) = font_def.families.get_mut(&FontFamily::Monospace) {
        vec.push("Bahnschrift".to_owned());
        vec.push("Segoe UI Emoji".to_owned());
        vec.push("SimHei".to_owned());
    }

    // Apply the font settings to the context
    ctx.set_fonts(font_def);
}

fn main() -> Result<(), eframe::Error> {
    // Open a log file in write mode
    let file = File::create("applog.json").unwrap();

    // Initialize tracing subscriber to log to a file
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(file) // Use the file as the writer
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(), //.with_inner_size([320.0, 240.0]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "💉Nullptr Injector💉",
        options,
        Box::new(|cc| {
            // This gives us image support:
            load_system_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<InjectorAppWindow>::default()
        }),
    )
}
