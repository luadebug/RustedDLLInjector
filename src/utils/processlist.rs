use std::{fs, ptr};

use libmem::process::{Process, enum_processes};
use tracing::{error, info};

// info

pub fn get_process_list() -> Vec<Process> {
    match enum_processes() {
        None => {
            error!("Failed to get process list");
            Vec::new()
        },
        Some(processlist) => {
            // info!("Got process list: {:?}", processlist);
            processlist
        },
    }
}

use std::ffi::c_void;

use dinvoke::{close_handle, nt_create_thread_ex};
use iced_x86::IcedError;
// use dinvoke_rs::dinvoke;
// use dinvoke_rs::dinvoke::{close_handle, nt_create_thread_ex};
use iced_x86::code_asm::{CodeAssembler, eax};
use libmem::memory::{alloc_memory_ex, free_memory_ex};
use libmem::module::find_module_ex;
use libmem::{Arch, Prot, load_module_ex, write_memory_ex};
use widestring::U16CString;

fn build_code_x86_fix(
    load_library_w: u32,
    _get_last_error: u32,
    _return_buffer: *mut u32,
    dll_path_addr: *mut u32,
) -> Result<Vec<u8>, IcedError> {
    let mut asm = CodeAssembler::new(32)?;
    // asm.mov(eax, dword_ptr(dll_path_addr as u32))?; // CreateRemoteThread
    // lpParameter
    asm.mov(eax, dll_path_addr as u32)?; // CreateRemoteThread lpParameter
    asm.push(eax)?; // lpLibFileName
    asm.mov(eax, load_library_w)?;
    asm.call(eax)?;
    // asm.mov(dword_ptr(return_buffer as u32), eax)?;
    ///////////////// asm.mov(eax, 0)?;
    // let mut label = asm.create_label();
    // asm.test(eax, eax)?;
    // asm.mov(eax, 0)?;
    // asm.jnz(label)?;
    // asm.mov(eax, get_last_error)?;
    // asm.call(eax)?; // return 0
    // asm.set_label(&mut label)?;
    asm.ret_1(4)?; // Restore stack ptr. (Callee cleanup)
    let code = asm.assemble(0x1234_5678)?;
    debug_assert_eq!(
        code,
        asm.assemble(0x1111_2222)?,
        "LoadLibraryW x86 stub is not location independent"
    );
    Ok(code)
}
use dinvoke::open_process;
use dinvoke_data::{PVOID, PsAttributeList, THREAD_ALL_ACCESS};
// use dinvoke_rs::data::{PsAttributeList, PVOID, THREAD_ALL_ACCESS};
// use dinvoke_rs::dinvoke::open_process;
use windows::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows::Win32::Foundation::HANDLE;
use winsafe::prelude::*;

pub fn inject_dll_test_fix(process: &Process, dll_path: &String) -> Result<(), String> {
    if process.arch == Arch::X64 {
        // Handle x64 injection
        return match load_module_ex(process, dll_path) {
            None => Err("Failed to load DLL in target process.".into()),
            Some(result) => {
                info!("{}", format!("Success!!! {}", result));
                Ok(())
            },
        };
    } else if process.arch == Arch::X86 {
        info!(
            "{}",
            format!("Process is x86, going to adapt injection mechanism: {:#?}", process.arch)
        );

        // Allocate memory in the target process for the DLL path
        let dll_path_wcstr = match U16CString::from_str(format!("{}\u{0}", dll_path)) {
            Err(err) => return Err(format!("Failed to create U16CString: {}", err)),
            Ok(wcstr) => wcstr,
        };
        let dll_path_wcstr_len = dll_path_wcstr.as_slice_with_nul().len();
        // Step 1: Allocate memory for the DLL path in the target process
        let remote_dll_path_memory = match alloc_memory_ex(process, dll_path_wcstr_len, Prot::RW) {
            Some(addr) => addr,
            None => return Err("Failed to allocate memory for DLL path.".into()),
        };

        // Step 2: Write the DLL path to the allocated memory
        match write_memory_ex(process, remote_dll_path_memory, dll_path_wcstr.as_slice_with_nul()) {
            Some(_) => {
                println!("Successfully overwrite remote process memory to store DLL path.");
            },
            None => {
                eprintln!("Failed to overwrite remote process memory to store DLL path.");
                return Err("Failed to write memory.".into());
            },
        }

        // Load the KERNEL32 DLL and get the addresses of the functions
        let kernel_32_dll_module = match find_module_ex(process, "KERNEL32.DLL") {
            Some(module) => module,
            None => return Err("Failed to find KERNEL32.DLL".into()),
        };
        let kernel_32_dll_bytes = match fs::read(kernel_32_dll_module.path) {
            Err(err) => return Err(format!("Failed to read KERNEL32.DLL: {}", err)),
            Ok(bytes) => bytes,
        };
        let kernel32_dll_pe_file = match pelite::PeFile::from_bytes(&kernel_32_dll_bytes) {
            Err(err) => return Err(format!("Failed to parse KERNEL32.DLL: {}", err)),
            Ok(pe) => pe,
        };

        let load_library_w_export = match kernel32_dll_pe_file.get_export_by_name("LoadLibraryW") {
            Err(err) => return Err(format!("Failed to find LoadLibraryW export: {}", err)),
            Ok(export) => export,
        };
        let load_library_w_symbol = match load_library_w_export.symbol() {
            Some(symbol) => symbol,
            None => return Err("Failed to find LoadLibraryW symbol".into()),
        };
        let load_library_w_addr = kernel_32_dll_module.base as u32 + load_library_w_symbol;

        let get_last_error_export = match kernel32_dll_pe_file.get_export_by_name("GetLastError") {
            Err(err) => return Err(format!("Failed to find GetLastError export: {}", err)),
            Ok(export) => export,
        };
        let get_last_error_symbol = match get_last_error_export.symbol() {
            Some(symbol) => symbol,
            None => return Err("Failed to find GetLastError symbol".into()),
        };
        let get_last_error_addr = kernel_32_dll_module.base as u32 + get_last_error_symbol;

        let get_last_error_addr_buffer = get_last_error_addr as *mut u32;
        // Build the shellcode with the address of the remote DLL path
        let shellcode = match build_code_x86_fix(
            load_library_w_addr,
            get_last_error_addr,
            get_last_error_addr_buffer,
            remote_dll_path_memory as *mut u32,
        ) {
            Err(err) => return Err(format!("Failed to build shellcode: {}", err)),
            Ok(code) => code,
        };

        // Step 3: Allocate memory for the shellcode in the target process
        let remote_memory = match alloc_memory_ex(process, shellcode.len(), Prot::XRW) {
            Some(addr) => addr,
            None => return Err("Failed to allocate memory for shellcode.".into()),
        };

        // Write the shellcode to the allocated memory
        match write_memory_ex(process, remote_memory, shellcode.as_slice()) {
            Some(_) => {
                println!("Successfully overwrite remote process memory to store shellcode.");
            },
            None => {
                eprintln!("Failed to overwrite remote process memory to store shellcode.");
                return Err("Failed to write memory.".into());
            },
        }

        let access = THREAD_ALL_ACCESS;
        let process_handle = open_process(access, 0, process.pid);

        let mut thread = HANDLE(0);
        let access: u32 = THREAD_ALL_ACCESS;
        let attributes: *mut OBJECT_ATTRIBUTES = ptr::null_mut();
        let function: PVOID = remote_memory as *mut u8 as PVOID;
        let args: PVOID = ptr::null_mut();
        let flags: u32 = 0;
        let zero: usize = 0;
        let stack: usize = 0;
        let reserve: usize = 0;
        let buffer: *mut PsAttributeList = ptr::null_mut();

        let ntstatus_create_thread = nt_create_thread_ex(
            &mut thread,
            access,
            attributes,
            process_handle,
            function,
            args,
            flags,
            zero,
            stack,
            reserve,
            buffer,
        );
        if (0..=0x3FFFFFFF).contains(&ntstatus_create_thread) {
            println!("Successfully created thread! {:#x}", ntstatus_create_thread);

            let waiteress = unsafe {
                kernel_Hevent::WaitForSingleObject(
                    &winsafe::HEVENT::from_ptr(thread.0 as *mut c_void),
                    Some(4294967295u32),
                )
            };
            match waiteress {
                Ok(waitress_ready) => match waitress_ready.raw() {
                    0x0000_0080 => {
                        eprintln!("WaitForSingleObject has been abandoned!");
                    },
                    0x0000_0000 => {
                        println!("WaitForSingleObject has been finished successfully!");
                    },
                    0x0000_0102 => {
                        eprintln!("WaitForSingleObject has been timed out!");
                    },
                    0xffff_ffff => {
                        eprintln!("WaitForSingleObject has failed!");
                    },
                    _ => {
                        eprintln!("WaitForSingleObject has failed! {:#x}", waitress_ready.raw());
                    },
                },
                Err(e) => {
                    eprintln!("Failed to wait for thread: {:#?}", e);
                },
            }
            close_handle(thread);
            close_handle(process_handle);
            free_memory_ex(process, remote_dll_path_memory, dll_path_wcstr_len);
            free_memory_ex(process, remote_memory, shellcode.len());
        } else {
            println!("Failed to create thread! {:#x}", ntstatus_create_thread);
            close_handle(process_handle);
            free_memory_ex(process, remote_dll_path_memory, dll_path_wcstr_len);
            free_memory_ex(process, remote_memory, shellcode.len());
        }

        println!("Allocated address for shellcode is {:#x}", remote_memory);
        println!("Shellcode = {:?}", shellcode);
        println!("Shellcode.len = {}", shellcode.len());
    } else {
        return Err("Process architecture not supported.".into());
    }
    Ok(())
}
