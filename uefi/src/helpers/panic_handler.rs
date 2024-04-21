use crate::helpers::system_table_opt;
use crate::println;
use cfg_if::cfg_if;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC]: {}", info);

    // Give the user some time to read the message
    if let Some(st) = system_table_opt() {
        st.boot_services().stall(10_000_000);
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            unsafe {
                core::ptr::write_volatile(&mut dummy, i);
            }
        }
    }

    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", feature = "qemu"))] {
            // If running in QEMU, use the f4 exit port to signal the error and exit
            use qemu_exit::QEMUExit;
            let custom_exit_success = 3;
            let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
            qemu_exit_handle.exit_failure();
        } else {
            // If the system table is available, use UEFI's standard shutdown mechanism
            if let Some(st) = system_table_opt() {
                use crate::table::runtime::ResetType;
                st.runtime_services()
                    .reset(ResetType::SHUTDOWN, crate::Status::ABORTED, None);
            }

            // If we don't have any shutdown mechanism handy, the best we can do is loop
            log::error!("Could not shut down, please power off the system manually...");

            cfg_if! {
                if #[cfg(target_arch = "x86_64")] {
                    loop {
                        unsafe {
                            // Try to at least keep CPU from running at 100%
                            core::arch::asm!("hlt", options(nomem, nostack));
                        }
                    }
                } else if #[cfg(target_arch = "aarch64")] {
                    loop {
                        unsafe {
                            // Try to at least keep CPU from running at 100%
                            core::arch::asm!("hlt 420", options(nomem, nostack));
                        }
                    }
                } else {
                    loop {
                        // just run forever dammit how do you return never anyway
                    }
                }
            }
        }
    }
}
