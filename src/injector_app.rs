use crate::dll_info::{ui_function, DllInfo};
use crate::emoji_label_widget::EmojiLabelWidget;
use crate::process_selection_method::ProcessSelectionMethod;
use crate::process_selection_method::ProcessSelectionMethod::{ByPID, ByPIDInput, ByProcessName};
use crate::utils::processlist::get_process_list;
use dll_syringe::process::OwnedProcess;
use egui::{ComboBox, Id, PointerButton, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use libmem::Process;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info};

impl Default for InjectorApp {
    fn default() -> Self {
        Self {
            combo_box_process_name: "".to_owned(),
            combo_box_pid: "".to_owned(),
            radio_button_proc_sel_meth: ByProcessName,
            //checkbox_value: false,
            text_edit_value: "".to_owned(),
            //process_architecture: "x64".to_owned(),
            process_list: get_process_list(),
            current_process_selected_index: 0,
            //focused_item_index: Some(0),
            selected_row: None,
            dll_list_vector: Vec::new(),
            show_popup_error_dll_already_added: false,
        }
    }
}

pub struct InjectorApp {
    combo_box_process_name: String,
    combo_box_pid: String,
    radio_button_proc_sel_meth: ProcessSelectionMethod,
    //checkbox_value: bool,
    text_edit_value: String,
    //process_architecture: String,
    process_list: Vec<Process>,
    current_process_selected_index: usize,
    //focused_item_index: Option<usize>,
    selected_row: Option<usize>,
    dll_list_vector: Vec<DllInfo>,
    show_popup_error_dll_already_added: bool,
}

impl InjectorApp {
    fn filter_system_services_and_daemon_processes(&self) -> Vec<&Process> {
        let mut sys32dir = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        sys32dir.push("System32");
        let mut syswow64dir = PathBuf::from(std::env::var("SystemRoot").ok().unwrap());
        syswow64dir.push("SysWOW64");

        let mut unique_processes: HashMap<&str, &Process> = HashMap::new();

        for process in self.process_list.iter().filter(|process| {
            !process
                .path
                .starts_with(sys32dir.as_os_str().to_str().unwrap())
                || process
                    .path
                    .starts_with(syswow64dir.as_os_str().to_str().unwrap())
        }) {
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
}

fn dll_list_table(ui: &mut Ui, selected_row: &mut Option<usize>, dll_list: &mut Vec<DllInfo>) {
    let c = dll_list.to_owned();

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(100.0).at_least(40.0)) // First column
        .column(Column::remainder().resizable(true)) // Second column
        .column(Column::remainder().resizable(true)) // Third column
        .column(Column::remainder().resizable(true)) // Fourth column
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
            } else {
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
    ui.label(if selected_row.is_some() {
        format!("{:#?}", &c[selected_row.unwrap() - 1usize])
    } else {
        format!("{:#?}", &DllInfo::default())
    });
}

impl eframe::App for InjectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {


                ui.vertical(|ui| {

                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(100.0).at_least(100.0)) // First column: Label
                        .column(Column::initial(100.0).at_least(100.0))    // Second column: Radio Button
                        .column(Column::initial(100.0).at_least(100.0))    // Third column: ComboBox
                        .body(|mut body| {
                            body.row(18.0, |mut row| {
                                // First column: Label
                                row.col(|ui| {
                                    let resp1 = ui.add(EmojiLabelWidget::new(obfstr!("⚙ Process:\t\t")));
                                    if resp1.hovered() && self.radio_button_proc_sel_meth == ByProcessName {
                                        let popup_id = Id::new(obfstr!("SelectedProcessNamePopUP"));
                                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                                        let _ = egui::popup_below_widget(ui, popup_id, &resp1, |popup_ui| {
                                            let process = &self.process_list[self.current_process_selected_index];
                                            let process_info = format!(
                                                "PID: {:#?}\nPPID: {:#?}\nArchitecture: {:#?}\nBits: {:#?}\nStart Time: {:#?}\nPath:\n{:#?}\nName: {:#?}",
                                                process.pid,
                                                process.ppid,
                                                process.arch,
                                                process.bits,
                                                process.start_time,
                                                process.path,
                                                process.name
                                            );
                                            popup_ui.label(process_info);
                                        });
                                    }
                                });

                                // Second column: Radio Button
                                row.col(|ui| {
                                    if ui.radio(self.radio_button_proc_sel_meth == ByProcessName, "").clicked() {
                                        self.radio_button_proc_sel_meth = ByProcessName;
                                    }
                                });

                                // Third column: ComboBox
                                row.col(|ui| {
                                    let cb1_resp = ComboBox::from_id_source(obfstr!("ProcessListComboBox"))
                                        .width(400.0)
                                        .selected_text(&self.combo_box_process_name)
                                        .show_ui(ui, |ui| {
                                            let mut filtered_processes: Vec<&Process> = self.filter_system_services_and_daemon_processes();
                                            filtered_processes.sort_by_key(|process| &process.name);

                                            let mut new_selected_process_name = None;
                                            let mut new_selected_process_index = None;

                                            for process in &filtered_processes {
                                                let selectable_text = format!("{}\t{}\t{}", process.name, process.pid, process.ppid);

                                                if ui.selectable_value(
                                                    &mut self.combo_box_process_name.as_str(),
                                                    process.name.as_str(),
                                                    selectable_text.as_str(),
                                                ).clicked() && self.radio_button_proc_sel_meth == ByProcessName {
                                                    new_selected_process_name = Some(process.name.to_owned());
                                                    new_selected_process_index = Some(self.process_list.iter().position(|x| x.pid == process.pid).unwrap());
                                                }
                                            }

                                            if let Some(name) = new_selected_process_name {
                                                self.combo_box_process_name = name;
                                            }

                                            if let Some(index) = new_selected_process_index {
                                                self.current_process_selected_index = index;
                                            }
                                        }).response;

                                    if cb1_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ByProcessName {
                                        self.process_list = get_process_list();
                                    }
                                });
                            });
                        });

                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(100.0).at_least(100.0)) // First column: Label
                        .column(Column::initial(100.0).at_least(100.0)) // Second column: Radio Button
                        .column(Column::initial(100.0).at_least(100.0)) // Third column: ComboBox
                        .body(|mut body| {
                            body.row(18.0, |mut row| {
                                // First column: Label
                                row.col(|ui| {
                                    let resp2 = ui.add(EmojiLabelWidget::new(obfstr!("⚙🆔 PID:\t\t\t\t\t\t")));
                                    if resp2.hovered() && self.radio_button_proc_sel_meth == ByPID {
                                        let popup_id = Id::new(obfstr!("SelectedProcessByPIDPopUP"));
                                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                                        let _ = egui::popup_below_widget(ui, popup_id, &resp2, |popup_ui| {
                                            let process = &self.process_list[self.current_process_selected_index];
                                            let process_info = format!(
                                                "PID: {:#?}\nPPID: {:#?}\nArchitecture: {:#?}\nBits: {:#?}\nStart Time: {:#?}\nPath:\n{:#?}\nName: {:#?}",
                                                process.pid,
                                                process.ppid,
                                                process.arch,
                                                process.bits,
                                                process.start_time,
                                                process.path,
                                                process.name
                                            );
                                            popup_ui.label(process_info);
                                        });
                                    }
                                });

                                // Second column: Radio Button
                                row.col(|ui| {
                                    if ui.radio(self.radio_button_proc_sel_meth == ByPID, "").clicked() {
                                        self.radio_button_proc_sel_meth = ByPID;
                                    }
                                });

                                // Third column: ComboBox
                                row.col(|ui| {
                                    let cb2_resp = ComboBox::from_id_source(obfstr!("PIDListComboBox"))
                                        .width(400.0)
                                        .selected_text(&self.combo_box_pid)
                                        .show_ui(ui, |ui| {
                                            let mut filtered_processes: Vec<&Process> = self.filter_system_services_and_daemon_processes();
                                            filtered_processes.sort_by_key(|process| process.pid);

                                            let mut new_selected_process_pid = None;
                                            let mut new_selected_process_index = None;

                                            for process in &filtered_processes {
                                                let process_pid = process.pid.to_string();
                                                let selectable_text = format!("{}\t{}\t{}", process.pid, process.name, process.ppid);

                                                if ui.selectable_value(
                                                    &mut self.combo_box_pid.as_str(),
                                                    process_pid.as_str(),
                                                    selectable_text.as_str(),
                                                ).clicked() && self.radio_button_proc_sel_meth == ByPID {
                                                    new_selected_process_pid = Some(process_pid);
                                                    new_selected_process_index = Some(self.process_list.iter().position(|x| x.pid == process.pid).unwrap());
                                                }
                                            }

                                            if let Some(pid) = new_selected_process_pid {
                                                self.combo_box_pid = pid;
                                            }

                                            if let Some(index) = new_selected_process_index {
                                                self.current_process_selected_index = index;
                                            }
                                        }).response;

                                    if cb2_resp.clicked_by(PointerButton::Primary) && self.radio_button_proc_sel_meth == ByPID {
                                        self.process_list = get_process_list();
                                    }
                                });
                            });
                        });
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(100.0).at_least(100.0)) // First column: Label
                        .column(Column::initial(100.0).at_least(100.0)) // Second column: Radio Button
                        .column(Column::initial(100.0).at_least(100.0)) // Third column: TextEdit
                        .column(Column::initial(100.0).at_least(100.0)) // Fourth column: Button
                        .body(|mut body| {
                            body.row(18.0, |mut row| {
                                // First column: Label
                                row.col(|ui| {
                                    let resp3 = ui.add(EmojiLabelWidget::new(obfstr!("⚙🆔📝 PID input:\t")));
                                    if resp3.hovered() && self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPIDInput {
                                        let popup_id = Id::new(obfstr!("SelectedProcessByPIDInputPopUP"));
                                        ui.memory_mut(|mem| mem.open_popup(popup_id));
                                        let _ = egui::popup_below_widget(ui, popup_id, &resp3, |popup_ui| {
                                            let process = &self.process_list[self.current_process_selected_index];
                                            let process_info = format!(
                                                "PID: {:#?}\nPPID: {:#?}\nArchitecture: {:#?}\nBits: {:#?}\nStart Time: {:#?}\nPath:\n{:#?}\nName: {:#?}",
                                                process.pid,
                                                process.ppid,
                                                process.arch,
                                                process.bits,
                                                process.start_time,
                                                process.path,
                                                process.name
                                            );
                                            popup_ui.label(process_info);
                                        });
                                    }
                                });

                                // Second column: Radio Button
                                row.col(|ui| {
                                    if ui.radio(self.radio_button_proc_sel_meth == ByPIDInput, "").clicked() {
                                        self.radio_button_proc_sel_meth = ByPIDInput;
                                    }
                                });

                                // Third column: TextEdit
                                row.col(|ui| {
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut self.text_edit_value)
                                            .char_limit(6)
                                            .desired_width(70.0),
                                    );

                                    if resp.has_focus() {
                                        self.text_edit_value = self
                                            .text_edit_value
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .take(6)
                                            .collect::<String>();
                                    }

                                    if self.radio_button_proc_sel_meth == ProcessSelectionMethod::ByPIDInput {
                                        if let Ok(input_pid) = self.text_edit_value.parse::<u32>() {
                                            if let Some(index) = self.process_list.iter().position(|x| x.pid == input_pid) {
                                                self.current_process_selected_index = index;
                                            } else {
                                                // If the PID is not found, refresh the process list
                                                self.process_list = get_process_list();
                                            }
                                        }
                                    }
                                });

                                // Fourth column: Button
                                row.col(|ui| {
                                    // Create an emoji button
                                    let emoji_button_select_process = EmojiButtonWidget::new(obfstr!("⚙📝 Select process"))
                                        .min_size(Vec2::from(&[292.0, 0.0])); // Set the button size

                                    // Add the button to the UI and handle clicks
                                    let response = ui.add(emoji_button_select_process);

                                    if response.clicked() {
                                        println!("Emoji button with custom label was clicked!");
                                        // Perform the action for the button click
                                    }

                                });
                            });
                        });
                    ui.horizontal(|ui| {
                        ui.label("Selected process :\t");
                        ui.label(format!("{:#?}", self.process_list[self.current_process_selected_index]));
                    });
                    ui.horizontal(|ui| {
/*                        if ui.button("🔄📄\u{2699} Update process list").clicked() {
                            self.process_list = get_process_list();
                        }*/

                        TableBuilder::new(ui)
                            .striped(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::initial(100.0).at_least(100.0)) // First column: Update button
                            .column(Column::initial(100.0).at_least(100.0)) // Second column: Inject DLL button
                            .body(|mut body| {
                                body.row(15.0, |mut row| {
                                    // First column: Update Process List Button
                                    row.col(|ui| {
                                        let emoji_button_update_process_list = EmojiButtonWidget::new(obfstr!("🔄📄\u{2699} Update process list"))
                                            .min_size(Vec2::from(&[200.0, 30.0])); // Set the button size

                                        let response = ui.add(emoji_button_update_process_list);

                                        if response.clicked() {
                                            self.process_list = get_process_list();
                                            println!("Emoji button with custom label was clicked! To update process list");
                                            // Perform the action for the button click
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.add_space(100.0f32);
                                    });
                                    // Second column: Inject DLL into Selected Process Button
                                    row.col(|ui| {
                                        let emoji_button_inject_dll_into_proc = EmojiButtonWidget::new(obfstr!("💉\u{2699} Inject DLL into selected process"))
                                            .min_size(Vec2::from(&[200.0, 30.0])); // Set the button size

                                        let response2 = ui.add(emoji_button_inject_dll_into_proc);

                                        if response2.clicked() {
                                            println!("Injecting DLL into selected process");
                                            println!("Process name: {}", self.process_list[self.current_process_selected_index].name);
                                            println!("PID: {}", self.process_list[self.current_process_selected_index].pid);

                                            for dll in &self.dll_list_vector {
                                                if dll.switch {
                                                    println!("Injecting DLL: {}", dll.dll_name);
                                                    match inject_dll(&self.process_list[self.current_process_selected_index], &dll.dll_path) {
                                                        Ok(_) => println!("Successfully injected: {}", dll.dll_name),
                                                        Err(e) => println!("Failed to inject {}: {}", dll.dll_name, e),
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Inject list");
                    ui.horizontal(|ui| {

                        ui_function(ui, &mut self.dll_list_vector, &mut self.selected_row, &mut self.show_popup_error_dll_already_added);
/*                    ui.vertical(|ui| {
                        open_file_dialog_and_add_dll(ui, &mut self.dll_list_vector, &mut self.show_popup_error_dll_already_added);
                        enable_disable_dll(ui, &mut self.dll_list_vector, &self.selected_row);
                        remove_selected_dll(ui, &mut self.dll_list_vector, &mut self.selected_row);
                        clear_all_dlls(ui, &mut self.dll_list_vector, &mut self.selected_row);
                    });*/
                    ui.vertical(|ui| {
                        dll_list_table(ui, &mut self.selected_row, &mut self.dll_list_vector);
                    });
                    });
                });

            });
        });
    }
}
use crate::emoji_button_widget::EmojiButtonWidget;
use dll_syringe::Syringe;
use obfstr::obfstr;

fn inject_dll(process: &Process, dll_path: &String) -> Result<(), String> {
    info!("DLL path: {}", dll_path);
    match OwnedProcess::from_pid(process.pid) {
        Ok(target_process) => {
            let syringe = Syringe::for_process(target_process);
            match syringe.inject(dll_path) {
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
