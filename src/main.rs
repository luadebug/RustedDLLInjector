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
use libmem::{Arch, Bits, Pid, Process};
use tracing::info;
use crate::utils::processlist::get_process_list;

use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};


mod utils;

fn load_system_font(ctx: &Context) {
    info!("Started loaading sys font");
    //let mut fonts = FontDefinitions::empty();
    //let ctx = Context::default();
    let font_file = {
        let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        font_path.push("Fonts");
        font_path.push("arial.ttf");
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
        //.push(font_name.to_owned());

    // Set up the font families with fallback
/*    if let Some(vec) = font_def.families.get_mut(&FontFamily::Proportional) {
        vec[0]=font_name.to_owned(); // Primary font for Latin characters
    }
    if let Some(vec) = font_def.families.get_mut(&FontFamily::Monospace) {
        vec[0]=font_name.to_owned(); // Primary font for Latin characters
    }*/
    ctx.set_fonts(font_def);
}

/*    // Load Microsoft YaHei (for Chinese characters)
    let handle_msyh = SystemSource::new()
        .select_best_match(&[FamilyName::Title("azuki_font".to_string())], &Properties::new())
        .unwrap();

    let buf_msyh: Vec<u8> = match handle_msyh {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path).unwrap(),
    };


    // Insert fonts into the FontData
    fonts.font_data.insert("azuki_font".to_owned(), FontData::from_owned(buf_msyh));


    // Set up the font families with fallback
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push("azuki_font".to_owned()); // Primary font for Latin characters

    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push("azuki_font".to_owned()); // Primary font for Latin characters

    }

    // Apply the font definitions
    ctx.set_fonts(fonts);
}*/


/*    let mut fonts = FontDefinitions::empty();

    let handle = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap();

    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path).unwrap(),
    };

    let handle2 = SystemSource::new()
        .select_best_match(&[FamilyName::Title("SimHei".to_string())], &Properties::new())
        .unwrap();

    let buf2: Vec<u8> = match handle2 {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path).unwrap(),
    };

    const FONT_SYSTEM_SANS_SERIF: &'static str = "System Sans Serif";
    const FONT_SYSTEM_SEGOI_UI_EMOJI: &'static str = "SimHei";
    fonts
        .font_data
        .insert(FONT_SYSTEM_SANS_SERIF.to_owned(), FontData::from_owned(buf));
    fonts
        .font_data
        .insert(FONT_SYSTEM_SEGOI_UI_EMOJI.to_owned(), FontData::from_owned(buf2));

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(FONT_SYSTEM_SANS_SERIF.to_owned());
        vec.push(FONT_SYSTEM_SEGOI_UI_EMOJI.to_owned());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(FONT_SYSTEM_SANS_SERIF.to_owned());
        vec.push(FONT_SYSTEM_SEGOI_UI_EMOJI.to_owned());
    }

    ctx.set_fonts(fonts);
}*/


//fn main() -> eframe::Result {
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
            egui_extras::install_image_loaders(&cc.egui_ctx);
            /*Ok(Box::new(MyApp::new(cc)))*/
            Box::<MyApp>::default()
        }),
    )
}
static mut SHOW_POPUP: bool = false;
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcessSelectionMethod {
    ByProcessName,
    ByPID,
    ByPIDInput,
}

struct MyApp {
    combo_box_process_name: String,
    combo_box_pid: String,
    radio_button_proc_sel_meth: ProcessSelectionMethod,
    checkbox_value: bool,
    text_edit_value: String,
    process_architecture: String,
    process_list: Vec<Process>,
    current_process_selected_index: usize,
    focused_item_index: Option<usize>
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            combo_box_process_name: "".to_owned(),
            combo_box_pid: "".to_owned(),
            radio_button_proc_sel_meth: ProcessSelectionMethod::ByProcessName,
            checkbox_value: false,
            text_edit_value: "".to_owned(),
            process_architecture: "x64".to_owned(),
            process_list: get_process_list(),
            current_process_selected_index: 0,
            focused_item_index: Some(0)
        }
    }
}
impl MyApp {

    fn filter_system_services_and_daemon_processes(&self) -> Vec<& Process> {
        let mut sys32dir = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        sys32dir.push("System32");
        let mut syswow64dir = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        syswow64dir.push("SysWOW64");

        let mut unique_processes: HashMap<&str, &Process> = HashMap::new();

        for process in self.process_list.iter()
            .filter(|process| !process.path.starts_with(sys32dir.as_os_str().to_str().unwrap()) || process.path.starts_with(syswow64dir.as_os_str().to_str().unwrap()))
        {
            // If the process name already exists in the HashMap,
            // compare PPIDs and keep the one with the lower PPID.
            if let Some(existing_process) = unique_processes.get_mut(&process.name as &str) {
                if process.ppid < existing_process.ppid {
                    *existing_process = process; // Update with the process having lower PPID
                }
            } else {
                // If the process name is not found, insert it into the HashMap.
                unique_processes.insert(&process.name as &str, process);
            }
        }

        // Collect the values (processes) from the HashMap into a Vec.
        unique_processes.values().copied().collect()
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: Frame,
        _storage: Option<&dyn Storage>,
    ) {
        // Load the Chinese font (SimHei)
        let font_file = {
            let mut font_path = PathBuf::from(std::env::var("SystemRoot").unwrap());
            font_path.push("Fonts");
            font_path.push("SimHei.ttf");
            font_path.to_str().unwrap().to_string().replace("\\", "/")
        };
        info!("Font path: {}", font_file); // Debugging: Confirm the font path
        let font_name = font_file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        let font_file_bytes = std::fs::read(font_file).unwrap();
        let font_data = FontData::from_owned(font_file_bytes);

        // Define the font definitions
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(font_name.clone(), font_data);

        // Set a custom font family
        let custom_font_family = FontFamily::Name(font_name.clone().into());

        // Set font families for Proportional and Monospace
        font_def.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, font_name.clone());
        font_def.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, font_name.clone());

        ctx.set_fonts(font_def.to_owned());
    }


    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load the Chinese font (SimHei)
        let font_file = {
            let mut font_path = PathBuf::from(std::env::var("SystemRoot").unwrap());
            font_path.push("Fonts");
            font_path.push("SimHei.ttf");
            font_path.to_str().unwrap().to_string().replace("\\", "/")
        };
        info!("Font path: {}", font_file); // Debugging: Confirm the font path
        let font_name = font_file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        let font_file_bytes = std::fs::read(font_file).unwrap();
        let font_data = FontData::from_owned(font_file_bytes);

        // Define the font definitions
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(font_name.clone(), font_data);

        // Set a custom font family
        let custom_font_family = FontFamily::Name(font_name.clone().into());

        // Set font families for Proportional and Monospace
        font_def.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, font_name.clone());
        font_def.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, font_name.clone());

        _cc.egui_ctx.set_fonts(font_def.to_owned());

        //load_system_font(&_cc.egui_ctx);
        //egui_extras::install_image_loaders(&_cc.egui_ctx);
        let mut style = Style::default();
        style.text_styles = [
            (HeadingStyle, FontId::new(30.0, Proportional)),
            (NameStyle("Heading2".into()), FontId::new(25.0, Proportional)),
            (NameStyle("Context".into()), FontId::new(23.0, Proportional)),
            (BodyStyle, FontId::new(18.0, Proportional)),
            (MonospaceStyle, FontId::new(14.0, Proportional)),
            (ButtonStyle, FontId::new(14.0, Proportional)),
            (SmallStyle, FontId::new(10.0, Proportional)),
        ].into();

        _cc.egui_ctx.set_style(style.to_owned());
        Self::default()
    }

/*    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let font_file = {
            let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
            font_path.push("Fonts");
            font_path.push("msyh.ttc");
            font_path.to_str().unwrap().to_string().replace("\\", "/")
        };
        let font_name = font_file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        let font_file_bytes = std::fs::read(font_file).ok().unwrap();

        let font_data = FontData::from_owned(font_file_bytes);
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(font_name.to_owned(), font_data);

        font_def.families.get_mut(&Proportional).unwrap().insert(0, font_name);


        _cc.egui_ctx.set_fonts(font_def.to_owned());

        let mut style = Style::default();
        style.text_styles = [
            (HeadingStyle, FontId::new(30.0, Proportional)),
            (NameStyle("Heading2".into()), FontId::new(25.0, Proportional)),
            (NameStyle("Context".into()), FontId::new(23.0, Proportional)),
            (BodyStyle, FontId::new(18.0, Proportional)),
            (MonospaceStyle, FontId::new(14.0, Proportional)),
            (ButtonStyle, FontId::new(14.0, Proportional)),
            (SmallStyle, FontId::new(10.0, Proportional)),
        ].into();

        _cc.egui_ctx.set_style(style.to_owned());
        Self::default()
    }*/
}

struct EmojiLabelWidget {
    label: EmojiLabel,
    text: String
}

impl EmojiLabelWidget {
    fn new(label: &str) -> Self {
        Self {
            text: label.to_owned(),
            label: EmojiLabel::new(label),
        }
    }

}
impl egui::Widget for EmojiLabelWidget {
    fn ui(self, ui_: &mut egui::Ui) -> egui::Response {
        let resp = self.label.show(ui_);
        // Ensure the cursor remains as an arrow when hovering over the label
        if resp.hovered() {
            ui_.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default);
        }
        resp
    }
}




/*pub fn unix_time_to_normal(timestamp: u64) -> String {
    let naive_datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap();
    naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}*/
pub fn unix_time_to_normal(timestamp: u64) -> String {
    let duration = Duration::from_secs(timestamp);
    let system_time = SystemTime::UNIX_EPOCH + duration;
    let naive_datetime: DateTime<Utc> = system_time.into();

    naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}


impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let resp1 = ui.add(EmojiLabelWidget::new("⚙ Process:\t\t\t\t\t\t\t\t\t\t\t\t"));
                    if resp1.hovered() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByProcessName {
                        let popup_id = Id::new("SelectedProcessNamePopUP");
                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                        let _ = popup_below_widget(
                            ui,
                            popup_id,
                            &resp1,
                            |popup_ui| {
                                let process = &self.process_list[self.current_process_selected_index];
                                let process_info = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                                                           format!("PID:{:#?}", process.pid),
                                                           format!("PPID:{:#?}", process.ppid),
                                                           format!("Architecture:{:#?}", process.arch),
                                                           format!("Bits:{:#?}", process.bits),
                                                           format!("Start Time:{:#?}", process.start_time),
                                                           format!("Path:\n{:#?}", process.path),
                                                           format!("Name:{:#?}", process.name));
                                popup_ui.label(process_info);
                            },
                        );
                    }

                    if ui.radio(self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByProcessName,
                                "".to_owned()).clicked()
                    {
                        self.radio_button_proc_sel_meth = ProcessSelectionMethod::ByProcessName;
                    }

                    let cb1_resp = ComboBox::from_id_source("ProcessListComboBox")
                        .width(400.0f32)
                        .selected_text(&self.combo_box_process_name)
                        .show_ui(ui, |ui| {
                            // Collect filtered processes
                            let mut filtered_processes: Vec<&Process> = self.filter_system_services_and_daemon_processes();
                            filtered_processes.sort_by_key(|process| &process.name);
                            // Store potential updates here
                            let mut new_selected_process_name = None;
                            let mut new_selected_process_index = None;

                            // Iterate over the filtered process list
                            for process in &filtered_processes {
                                // Clone the string for selectable_value
                                let process_name = process.name.to_owned();
                                let selectable_text = format!("{}\t{}\t{}", process.name, process.pid, process.ppid);

                                if ui.selectable_value(
                                    &mut self.combo_box_process_name.as_str(), // No longer modifying directly
                                    process_name.as_str(),
                                    selectable_text.as_str()
                                ).clicked() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByProcessName {

                                    // Store potential update
                                    new_selected_process_name = Some(process.name.to_owned());
                                    new_selected_process_index = Some(self.process_list.iter()
                                        .position(|x| x.pid == process.pid)
                                        .unwrap());
                                }
                            }

                            // Update after the loop
                            if let Some(name) = new_selected_process_name {
                                self.combo_box_process_name = name;
                            }

                            if let Some(index) = new_selected_process_index {
                                self.current_process_selected_index = index;
                            }
                        }).response;
/*                    if cb1_resp.hovered() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByProcessName
                    {


                    }*/
                    if cb1_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByProcessName
                    {
                        self.process_list = get_process_list();
                    }
                });


                ui.horizontal(|ui| {
                    //ui.label("\u{2699} PID:\t\t\t");

                    let resp2 = ui.add(EmojiLabelWidget::new("⚙ PID:\t\t\t\t\t\t\t\t\t\t\t\t"));
                    if resp2.hovered() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPID {
                        let popup_id = Id::new("SelectedProcessByPIDPopUP");
                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                        let _ = popup_below_widget(
                            ui,
                            popup_id,
                            &resp2,
                            |popup_ui| {
                                let process = &self.process_list[self.current_process_selected_index];
                                let process_info = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                                                           format!("PID:{:#?}", process.pid),
                                                           format!("PPID:{:#?}", process.ppid),
                                                           format!("Architecture:{:#?}", process.arch),
                                                           format!("Bits:{:#?}", process.bits),
                                                           format!("Start Time:{:#?}", process.start_time),
                                                           format!("Path:\n{:#?}", process.path),
                                                           format!("Name:{:#?}", process.name));
                                popup_ui.label(process_info);
                            },
                        );
                    }


                    if ui.radio(self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPID,
                                "".to_owned()).clicked()
                    {
                        self.radio_button_proc_sel_meth = ProcessSelectionMethod::ByPID;
                    }
                    let cb2_resp = ComboBox::from_id_source("PIDListComboBox").width(400.0f32)
                        .selected_text(&self.combo_box_pid)
                        .show_ui(ui, |ui| {

                            // Collect filtered processes
                            let mut filtered_processes: Vec<&Process> = self.filter_system_services_and_daemon_processes();
                            filtered_processes.sort_by_key(|process| &process.pid);
                            // Store potential updates here
                            let mut new_selected_process_pid = None;
                            let mut new_selected_process_index = None;

                            for process in &filtered_processes {
                                let process_pid = process.pid.to_owned();
                                let selectable_text = format!("{}\t{}\t{}", process.pid, process.name, process.ppid);

                                if ui.selectable_value(&mut self.combo_box_pid.as_str(),
                                                       process_pid.to_string().as_str(),
                                                       selectable_text.as_str(),
                                ).clicked() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPID
                                {
                                    // Store potential update
                                    new_selected_process_pid = Some(process.pid.to_owned());
                                    new_selected_process_index = Some(self.process_list.iter()
                                        .position(|x| x.pid == process.pid)
                                        .unwrap());
                                }
                            }

                            // Update after the loop
                            if let Some(pid) = new_selected_process_pid {
                                self.combo_box_pid = pid.to_string();
                            }

                            if let Some(index) = new_selected_process_index {
                                self.current_process_selected_index = index;
                            }
                        }).response;

                    if cb2_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPID
                    {
                        self.process_list = get_process_list();
                    }
                });
                ui.horizontal(|ui| {
                    //ui.label("📝\u{2699} PID input:\t");
                    let resp3 = ui.add(EmojiLabelWidget::new("⚙📝 PID input::\t\t\t\t\t\t\t\t\t\t\t\t"));
                    if resp3.hovered() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPIDInput {
                        let popup_id = Id::new("SelectedProcessByPIDInputPopUP");
                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                        let _ = popup_below_widget(
                            ui,
                            popup_id,
                            &resp3,
                            |popup_ui| {
                                let process = &self.process_list[self.current_process_selected_index];
                                let process_info = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                                                           format!("PID:{:#?}", process.pid),
                                                           format!("PPID:{:#?}", process.ppid),
                                                           format!("Architecture:{:#?}", process.arch),
                                                           format!("Bits:{:#?}", process.bits),
                                                           format!("Start Time:{:#?}", process.start_time),
                                                           format!("Path:\n{:#?}", process.path),
                                                           format!("Name:{:#?}", process.name));
                                popup_ui.label(process_info);
                            },
                        );
                    }



                    if ui.radio(self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPIDInput,
                                "".to_owned()).clicked()
                    {
                        self.radio_button_proc_sel_meth = ProcessSelectionMethod::ByPIDInput;
                    }
                    let resp = ui.add(TextEdit::singleline(&mut self.text_edit_value)
                        .char_limit(6)
                        .desired_width(70.0f32)
                    );

                    if resp.has_focus()
                    {
                        self.text_edit_value = self.text_edit_value.chars()
                            .filter(|c| c.is_ascii_digit())
                            .take(6)
                            .collect();
                    }

                    if self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPIDInput
                    {

                        if self.text_edit_value.parse::<u32>().is_ok()
                        {
                            let result = self.process_list.iter().position(|x|
                            <u32 as Into<Pid>>::into(x.pid) ==
                                self.text_edit_value.parse::<u32>().unwrap());
                            if result.is_some()
                            {
                                self.current_process_selected_index = result.unwrap();
                                if resp.hovered()
                                {
/*                                    resp.show_tooltip_text(format!("Selected {}",
                                                                   self.process_list[self.current_process_selected_index]));*/
                                }
                            } else {
/*                                resp.show_tooltip_text("Wrong PID input. Please try again.");*/
                                self.process_list = get_process_list();
                            }
                        }
                    }
                    //ui.add(Button::new("⚙📝Select process".to_owned()).min_size(Vec2::from([140.0f32, 0.0f32])));
                    let button = egui::Button::new("\t\t\t\t\t\t"); // Create a button without a label
                    let response = ui.add(button); // Add the button to the UI

                    // Manually draw the label on top of the button
                    let label = EmojiLabelWidget::new("⚙📝 PID input::\t\t\t\t\t\t\t\t\t\t\t\t");//egui::Label::new("Click me!").sense(egui::Sense::click());
                    let label_response = ui.put(response.rect, label);

                    if response.clicked() || label_response.clicked()  {
                        println!("Button with custom label was clicked!");
                    }


                });
                ui.horizontal(|ui| {
                    ui.label("Selected process :\t");
                    ui.label(format!("{:#?}", self.process_list[self.current_process_selected_index]));
                });
                ui.horizontal(|ui| {
                    if ui.button("Update process list").clicked() {
                        self.process_list = get_process_list();
                    }
                    if ui.button("💉\u{2699} Inject DLL into selected process").clicked() {
                        info!("Injecting DLL into selected process");
                        info!("Process name: {}", self.process_list[self.current_process_selected_index].name);
                        info!("PID: {}", self.process_list[self.current_process_selected_index].pid);
                        //info!("Process PID: {}", self.current_process_selected.pid);
                        //info!("DLL path: {}", self.text_edit_value);
                    }
                });
            });
        });
    }
}