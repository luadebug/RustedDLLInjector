use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::Error;
use std::path::{Path, PathBuf};
use dll_syringe::process::{BorrowedProcessModule, OwnedProcess};
use eframe::{Frame, Storage};
use eframe::epaint::{FontFamily, FontId};
use eframe::epaint::FontFamily::Proportional;
use eframe::epaint::text::{FontData, FontDefinitions};
use egui::{Button, ComboBox, Grid, Id, PointerButton, popup_below_widget, Style, TextEdit, Ui, Vec2};
use egui::TextStyle::{Body as BodyStyle, Button as ButtonStyle, Heading as HeadingStyle, Monospace as MonospaceStyle, Name as NameStyle, Small as SmallStyle};
use egui_extras::{Column, TableBuilder};
use libmem::{Pid, Process, process};
use tracing::{error, info};
use crate::DllInfo::{clear_all_dlls, DllInfo, enable_disable_dll, open_file_dialog_and_add_dll, remove_selected_dll};
use crate::EmojiLabelWidget::EmojiLabelWidget;
use crate::ProcessSelectionMethod::ProcessSelectionMethod;
use crate::ProcessSelectionMethod::ProcessSelectionMethod::{ByPID, ByPIDInput, ByProcessName};
use crate::utils::processlist::get_process_list;


impl Default for InjectorApp {
    fn default() -> Self {
        Self {
            combo_box_process_name: "".to_owned(),
            combo_box_pid: "".to_owned(),
            radio_button_proc_sel_meth: ByProcessName,
            checkbox_value: false,
            text_edit_value: "".to_owned(),
            process_architecture: "x64".to_owned(),
            process_list: get_process_list(),
            current_process_selected_index: 0,
            focused_item_index: Some(0),
            selected_row: None,
            dll_list_vector: Vec::new(),
            show_popup_error_dll_already_added: false
        }
    }
}

pub struct InjectorApp {
    combo_box_process_name: String,
    combo_box_pid: String,
    radio_button_proc_sel_meth: ProcessSelectionMethod,
    checkbox_value: bool,
    text_edit_value: String,
    process_architecture: String,
    process_list: Vec<Process>,
    current_process_selected_index: usize,
    focused_item_index: Option<usize>,
    selected_row: Option<usize>,
    dll_list_vector: Vec<DllInfo>,
    show_popup_error_dll_already_added: bool
}

impl InjectorApp {

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
        egui_extras::install_image_loaders(&_cc.egui_ctx);
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
}

fn dll_list_table(ui: &mut Ui, selected_row: &mut Option<usize>,
                  dll_list: &mut Vec<DllInfo>) {

    let c = dll_list.to_owned();

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(100.0).at_least(40.0)) // First column
        .column(Column::remainder().resizable(true))    // Second column
        .column(Column::remainder().resizable(true))    // Third column
        .column(Column::remainder().resizable(true))    // Fourth column
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("Switch");
            });
            header.col(|ui| {
                ui.label("DLL Name");
            });
            header.col(|ui| {
                ui.label("DLL Arch");
            });
            header.col(|ui| {
                ui.label("DLL Path");
            });
        })
        .body(|mut body| {

            if dll_list.is_empty() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.label("No DLLs found");
                    });
                });
            }
            else {
                for dll in dll_list {
                    let is_selected = *selected_row == Some(dll.index);
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            let response = ui.checkbox(&mut dll.switch, "ON/OFF");
                            if response.clicked() {
                                *selected_row = Some(dll.index);
                            }
                        });
                        row.col(|ui| {
                            let response = ui.selectable_label(is_selected, &dll.dll_name);
                            if response.clicked() {
                                *selected_row = Some(dll.index);
                            }
                        });
                        row.col(|ui| {
                            let response = ui.selectable_label(is_selected, &dll.dll_arch);
                            if response.clicked() {
                                *selected_row = Some(dll.index);
                            }
                        });
                        row.col(|ui| {
                            let response = ui.selectable_label(is_selected, &dll.dll_path);
                            if response.clicked() {
                                *selected_row = Some(dll.index);
                            }
                        });
                    });
                }
            }
        });


        ui.label(format!("Selected Row: {:?}", selected_row));
        ui.label(if selected_row.is_some() { format!("{:#?}", &c[selected_row.unwrap() - 1usize]) } else { format!("{:#?}", &DllInfo::default()) });
}



impl eframe::App for InjectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {


                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let resp1 = ui.add(EmojiLabelWidget::new("⚙ Process:\t\t\t\t\t\t\t\t\t\t\t\t"));
                        if resp1.hovered() && self.radio_button_proc_sel_meth == ByProcessName {
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

                        if ui.radio(self.radio_button_proc_sel_meth == ByProcessName,
                                    "".to_owned()).clicked()
                        {
                            self.radio_button_proc_sel_meth = ByProcessName;
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
                                    ).clicked() && self.radio_button_proc_sel_meth == ByProcessName {

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
                        if cb1_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ByProcessName
                        {
                            self.process_list = get_process_list();
                        }
                    });
                    ui.horizontal(|ui| {
                        //ui.label("\u{2699} PID:\t\t\t");

                        let resp2 = ui.add(EmojiLabelWidget::new("⚙ PID:\t\t\t\t\t\t\t\t\t\t\t\t"));
                        if resp2.hovered() && self.radio_button_proc_sel_meth == ByPID {
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


                        if ui.radio(self.radio_button_proc_sel_meth == ByPID,
                                    "".to_owned()).clicked()
                        {
                            self.radio_button_proc_sel_meth = ByPID;
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
                                    ).clicked() && self.radio_button_proc_sel_meth == ByPID
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

                        if cb2_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ByPID
                        {
                            self.process_list = get_process_list();
                        }
                    });
                    ui.horizontal(|ui| {
                        //ui.label("📝\u{2699} PID input:\t");
                        let resp3 = ui.add(EmojiLabelWidget::new("⚙📝 PID input::\t\t\t\t\t\t\t\t\t\t\t\t"));
                        if resp3.hovered() && self.radio_button_proc_sel_meth == ByPIDInput {
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



                        if ui.radio(self.radio_button_proc_sel_meth == ByPIDInput,
                                    "".to_owned()).clicked()
                        {
                            self.radio_button_proc_sel_meth = ByPIDInput;
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

                        if self.radio_button_proc_sel_meth == ByPIDInput
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

                            for dll in &self.dll_list_vector {
                                if dll.switch {
                                    info!("Injecting DLL: {}", dll.dll_name);
                                    match inject_dll(&self.process_list[self.current_process_selected_index], &dll.dll_path) {
                                        Ok(_) => info!("Successfully injected: {}", dll.dll_name),
                                        Err(e) => info!("Failed to inject {}: {}", dll.dll_name, e),
                                    }
                                }
                            }

                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Inject list");
                    ui.horizontal(|ui| {


                    ui.vertical(|ui| {
                        open_file_dialog_and_add_dll(ui, &mut self.dll_list_vector, &mut self.show_popup_error_dll_already_added);
                        enable_disable_dll(ui, &mut self.dll_list_vector, &self.selected_row);
                        remove_selected_dll(ui, &mut self.dll_list_vector, &mut self.selected_row);
                        clear_all_dlls(ui, &mut self.dll_list_vector, &mut self.selected_row);
                    });
                    ui.vertical(|ui| {
                        dll_list_table(ui, &mut self.selected_row, &mut self.dll_list_vector);
                    });
                    });
                });

            });
        });
    }
}
use dll_syringe::Syringe;
fn inject_dll(process: &Process, dll_path: &String) -> Result<(), String> {
    info!("DLL path: {}", dll_path);
    match OwnedProcess::from_pid(process.pid)
    {
        Ok(target_process) => {
            let syringe = Syringe::for_process(target_process);
            match syringe.inject(dll_path)
            {
                Ok(_) => {
                    info!("Successfully injected: {}", dll_path);
                }
                Err(e) => {
                    error!("Failed to inject {}: {}", dll_path, e);
                }
            }
        }
        Err(e) => {
            error!("Failed to open process {}: {}", process.pid, e);
        }
    }
    Ok(())
}