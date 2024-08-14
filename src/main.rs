//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashMap;
use std::fs::{File, read};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use eframe::{egui, Frame, Storage};
use egui::{AboveOrBelow, Button, ComboBox, Context, CursorIcon, Event, FontData, FontDefinitions, FontFamily, FontId, Id, PointerButton, popup_above_or_below_widget, popup_below_widget, Response, RichText, Sense, Style, TextEdit, TextStyle, Ui, Vec2, WidgetText};
use egui::Event::Key;
use egui::FontFamily::{Name, Proportional};
use egui::TextStyle::{Body as BodyStyle, Button as ButtonStyle, Heading as HeadingStyle, Monospace as MonospaceStyle, Name as NameStyle, Small as SmallStyle};
use egui_twemoji::EmojiLabel;
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use libmem::{Arch, Bits, Pid, Process};
use tracing::info;


use crate::ProcessSelectionMethod::ProcessSelectionMethod::{ByPID, ByPIDInput, ByProcessName};
use crate::utils::processlist::get_process_list;
use crate::InjectorApp::{InjectorApp as InjectorAppWindow};

mod utils;
mod InjectorApp;
mod ProcessSelectionMethod;
mod EmojiLabelWidget;

fn load_system_font(ctx: &Context) {
    info!("Started loaading sys font");
    //let mut fonts = FontDefinitions::empty();
    //let ctx = Context::default();
    let font_file = {
        let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        font_path.push("Fonts");
        font_path.push("SimHei.ttf");
        font_path.to_str().unwrap().to_string().replace("\\", "/")
    };
    let font_name = font_file.split('/').last().unwrap().split('.').next().unwrap().to_string();
    let font_file_bytes = std::fs::read(font_file).ok().unwrap();

    let font_data = FontData::from_owned(font_file_bytes);
    let mut font_def = FontDefinitions::empty();
    font_def.font_data.insert(font_name.to_string(), font_data);

    font_def
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, font_name.to_owned());

    font_def
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, font_name.to_owned());

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
            load_system_font(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<InjectorAppWindow>::default()
        }),
    )
}

