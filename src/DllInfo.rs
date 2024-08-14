use std::ffi::OsStr;
use std::path::Path;
use std::thread::sleep;
use egui::{Button, Ui, Vec2, Window};
use pelite::{FileMap, PeFile};
use pelite::image::IMAGE_FILE_HEADER;
use pelite::resources::FindError::Pe;
use rfd::FileDialog;

#[derive(Debug, Clone)]
pub struct DllInfo {
    pub(crate) switch: bool,
    pub(crate) dll_name: String,
    pub(crate) dll_path: String,
    pub(crate) dll_arch: String,
    pub(crate) index: usize
}

impl DllInfo {
    // Example method to create a new DllInfo instance
    pub fn new(switch: bool, dll_name: String, dll_path: String, dll_arch: String, index: usize) -> Self {
        DllInfo {
            switch,
            dll_name,
            dll_path,
            dll_arch,
            index
        }
    }
}

// Implementing the Default trait for DllInfo
impl Default for DllInfo {
    fn default() -> Self {
        DllInfo {
            switch: false,
            dll_name: String::from("undefined"),
            dll_path: String::from("undefined"),
            dll_arch: String::from("undefined"),
            index: 0usize
        }
    }
}

use pelite::image::{IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_IA64, IMAGE_FILE_32BIT_MACHINE};
// Function to determine DLL architecture using pelite
pub fn get_dll_architecture(path: &Path) -> String {
    let file_map = FileMap::open(path).expect("Failed to open the file.");
    let pe = PeFile::from_bytes(&file_map).expect("Failed to parse the PE file.");
    match pe.file_header().Machine {
        IMAGE_FILE_MACHINE_AMD64 => "x64/AMD64".to_string(),
        IMAGE_FILE_MACHINE_I386 => "x86/I386".to_string(),
        IMAGE_FILE_MACHINE_IA64 => "x64/IA64".to_string(),
        IMAGE_FILE_32BIT_MACHINE => "x32/32BIT".to_string(),
        _ => "unknown".to_string(),
    }
}


pub fn remove_selected_dll(ui: &mut Ui, dll_list_vector: &mut Vec<DllInfo>, selected_row: &mut Option<usize>) {
    let remove_resp = ui.add(Button::new("Remove").min_size(Vec2::from([140.0f32, 0.0f32])));

    if remove_resp.clicked() {
        if let Some(selected_index) = selected_row {
            // Remove the selected DLL
            dll_list_vector.retain(|dll| dll.index != *selected_index);

            // Clear the selected row
            *selected_row = None;

            // Reassign indexes to maintain order
            for (i, dll) in dll_list_vector.iter_mut().enumerate() {
                dll.index = i + 1;
            }
        }
    }
}

pub fn clear_all_dlls(ui: &mut Ui, dll_list_vector: &mut Vec<DllInfo>, selected_row: &mut Option<usize>) {
    let clear_resp = ui.add(Button::new("Clear").min_size(Vec2::from([140.0f32, 0.0f32])));

    if clear_resp.clicked() {
        // Clear the vector of DLLs
        dll_list_vector.clear();

        // Clear the selected row
        *selected_row = None;
    }
}


pub fn enable_disable_dll(ui: &mut Ui, dll_list_vector: &mut Vec<DllInfo>, selected_row: &Option<usize>)
{
    let enable_disable_resp = ui.add(Button::new("Enable/Disable").min_size(Vec2::from([140.0f32, 0.0f32])));
    if enable_disable_resp.clicked() {
        if let Some(selected_index) = *selected_row {
            if let Some(dll) = dll_list_vector.iter_mut().find(|dll| dll.index == selected_index) {
                dll.switch = !dll.switch; // Toggle the switch state
            }
        }
    }
}


pub fn open_file_dialog_and_add_dll(ui: &mut Ui, dll_list_vector: &mut Vec<DllInfo>, show_popup: &mut bool) {
    let add_dll_resp = ui.add(Button::new("Add DLL").min_size(Vec2::from([140.0f32, 0.0f32])));

    if add_dll_resp.clicked() {
        if let Some(path) = FileDialog::new()
            .add_filter("DLL Files", &["dll"])
            .pick_file()
        {
            let file_name = path.file_name().unwrap_or_else(|| OsStr::new("undefined")).to_string_lossy().into_owned();
            let file_path = path.to_string_lossy().into_owned();

            // Check if the DLL is already in the list by comparing paths
            let already_exists = dll_list_vector.iter().any(|dll| dll.dll_path == file_path);
            if !already_exists {
                let file_arch = get_dll_architecture(&path);

                dll_list_vector.push(DllInfo::new(
                    false,
                    file_name,
                    file_path,
                    file_arch,
                    dll_list_vector.len() + 1,
                ));
            } else {
                // Show the popup if the DLL is already in the list
                *show_popup = true;
            }
        }
    }

    // Display the popup if needed
    if *show_popup {
        Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("This DLL is already added.");
                if ui.button("OK").clicked() {
                    *show_popup = false;
                }
            });
    }
}