// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;
use core::ffi::c_void;
use uefi::boot;
use uefi::proto::debug::{DebugPort, DebugSupport, ExceptionType, ProcessorArch, SystemContext};

pub fn test() {
    test_debug_support();
    test_debug_port();
}

fn test_debug_port() {
    info!("Running UEFI debug port protocol test");
    if let Ok(handles) = boot::find_handles::<DebugPort>() {
        for handle in handles {
            if let Ok(debug_port) = boot::open_protocol_exclusive::<DebugPort>(handle) {
                let timeout = 1000;

                debug_port
                    .reset()
                    .expect("Error while resetting debug port");
                let data: Vec<_> = r##"
                                ..    .=-     .
                          .    :##+  .*##=  -*#+    ..
                         =#*=:.*####+#####**####-.-*##
                   .=-.  *############################:  :-:
                   -###**##############################*###*
               :.  -#######################################*  .::
              =##################################################
              .#################################################=
          -====#################################################+====.
          =##########################################################.
           +########################################################.
       -==+#########################################################*+=-.
       +######################=:=@@%#########+:-%@%####################*:
        =####################+   +@@@%######%.  -@@@##################*.
      :+#####################@-.:%@@@@#####%@*::#@@@%###################+:
     +#######################@@@@@@@@@######@@@@@@@@%#####################-
    -#########################@@@@@@@########@@@@@@%#######################
    -#######%%%##################%##############################%%%%######*
     +########%%%%############################################*%%%%#######:
      =######+=%%%#####***##%%#####################%#**++####=:%%%#######:
       :*#####:.*%#-####+==*########+-::::::=*#######*=+###*- *%*:-####*.
         -####*  .+. -*###############=   .*#############*-  .*:  *###+
           =###:        *##############+ .##############+        .###=
            .+#*        -######*=+++**** =###***++######-        :#*.
              .-         -######-               .*#####-         .-
                           =*####*.            =####+-
                              .:--:           ::::.
"##
                .bytes()
                .collect();

                debug_port
                    .write(timeout, &data)
                    .expect("Error while writing to debug port");

                debug_port.poll().expect("Error while polling debug port");

                let mut data = Vec::with_capacity(4096);

                debug_port
                    .read(timeout, &mut data)
                    .expect("Error while reading from debug port");
            }
        }
    }
}

fn test_debug_support() {
    if cfg!(not(feature = "debug_support")) {
        return;
    }

    info!("Running UEFI debug connection protocol test");
    let handles = boot::find_handles::<DebugSupport>().expect("missing DebugSupport protocol");
    for handle in handles {
        if let Ok(mut debug_support) = boot::open_protocol_exclusive::<DebugSupport>(handle) {
            // make sure that the max processor index is a sane value, i.e. it works
            let maximum_processor_index = debug_support.get_maximum_processor_index();
            assert_ne!(
                maximum_processor_index,
                usize::MAX,
                "get_maximum_processor_index() returning garbage, unless you really have 18,446,744,073,709,551,615 processors"
            );

            info!("- Architecture: {:?}", debug_support.arch());
            info!("- Maximum Processor Index: {maximum_processor_index:?}");

            match debug_support.arch() {
                // This arm is the only match when testing on QEMU w/ OVMF, regardless of the machine arch.
                // The released OVMF builds don't implement the Debug Support Protocol Interface for the
                // machine arch, only EBC.
                ProcessorArch::EBC => unsafe {
                    info!("Registering periodic callback");
                    debug_support
                        .register_periodic_callback(0, Some(periodic_callback))
                        .expect("Error while registering periodic callback");
                    info!("Deregistering periodic callback");
                    debug_support
                        .register_periodic_callback(0, None)
                        .expect("Error while deregistering periodic callback");
                    // for the EBC virtual CPU, there are already exception callbacks registered
                    info!("Deregistering exception callback");
                    debug_support
                        .register_exception_callback(0, None, ExceptionType::EXCEPT_EBC_DEBUG)
                        .expect("Error while deregistering exception callback");
                    info!("Registering exception callback");
                    debug_support
                        .register_exception_callback(
                            0,
                            Some(exception_callback),
                            ExceptionType::EXCEPT_EBC_DEBUG,
                        )
                        .expect("Error while registering exception callback");
                },
                #[cfg(target_arch = "x86_64")]
                ProcessorArch::X86_64 => unsafe {
                    info!("Registering exception callback");
                    debug_support
                        .register_exception_callback(
                            0,
                            Some(exception_callback),
                            ExceptionType::EXCEPT_X64_DEBUG,
                        )
                        .expect("Error while registering exception callback");
                    info!("Deregistering exception callback");
                    debug_support
                        .register_exception_callback(0, None, ExceptionType::EXCEPT_X64_DEBUG)
                        .expect("Error while deregistering exception callback");
                },
                #[cfg(target_arch = "aarch64")]
                ProcessorArch::AARCH_64 => unsafe {
                    info!("Registering exception callback");
                    debug_support
                        .register_exception_callback(
                            0,
                            Some(exception_callback),
                            ExceptionType::EXCEPT_AARCH64_SERROR,
                        )
                        .expect("Error while registering exception callback");
                    info!("Deregistering exception callback");
                    debug_support
                        .register_exception_callback(0, None, ExceptionType::EXCEPT_AARCH64_SERROR)
                        .expect("Error while deregistering exception callback");
                },
                // if we reach this, we're running on an arch that `cargo xtask run` doesn't support
                // TODO: Add match arms as we support testing on more archs
                _ => unreachable!(),
            }

            test_invalidate_instruction_cache(&mut debug_support);
        }
    }
}

fn test_invalidate_instruction_cache(debug_support: &mut DebugSupport) {
    info!("Invalidating instruction cache");
    let mut addr = 0x0;
    let ptr = &mut addr as *mut _ as *mut c_void;

    unsafe {
        debug_support
            .invalidate_instruction_cache(0, ptr, 64)
            // Should always pass, since the spec says this always returns EFI_SUCCESS
            .expect("Error occurred while invalidating instruction cache");
    }
}

extern "efiapi" fn periodic_callback(context: SystemContext) {
    let _ = context;
}

extern "efiapi" fn exception_callback(exception_type: ExceptionType, context: SystemContext) {
    let _ = exception_type;
    let _ = context;
}
