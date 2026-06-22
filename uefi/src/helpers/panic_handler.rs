// SPDX-License-Identifier: MIT OR Apache-2.0

use core::time::Duration;

use crate::{boot, println};
use cfg_if::cfg_if;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC]: {}", info);

    // Give the user some time to read the message
    if boot::are_boot_services_active() {
        boot::stall(Duration::from_secs(10));
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            // SAFETY: we own the memory.
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
            // SAFETY: the I/O port matches the one of the QEMU environment.
            let qemu_exit_handle = unsafe { qemu_exit::X86::new(0xF4, custom_exit_success) };
            qemu_exit_handle.exit_failure();
        } else {
            // If the system table is available, use UEFI's standard shutdown mechanism
            if let Some(st) = crate::table::system_table_raw() {
                // SAFETY: The handle is known to point to live storage here.
                if !unsafe { st.as_ref().runtime_services }.is_null() {
                    crate::runtime::reset(crate::runtime::ResetType::SHUTDOWN, crate::Status::ABORTED, None);
                }
            }

            // If we don't have any shutdown mechanism handy, the best we can do is loop
            log::error!("Could not shut down, please power off the system manually...");

            cfg_if! {
                if #[cfg(target_arch = "x86_64")] {
                    loop {
                        // SAFETY: No side-effects on memory.
                        unsafe {
                            // Try to at least keep CPU from running at 100%
                            core::arch::asm!("hlt", options(nomem, nostack));
                        }
                    }
                } else if #[cfg(target_arch = "aarch64")] {
                    loop {
                        // SAFETY: No side-effects on memory.
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
