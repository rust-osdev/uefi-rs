use core::ffi::c_void;
use uefi::proto::debug::{DebugSupport, ExceptionType, ProcessorArch, SystemContext};
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running UEFI debug connection protocol test");
    if let Ok(handles) = bt.find_handles::<DebugSupport>() {
        let handles = handles.expect("Problem encountered while querying handles for DebugSupport");

        for handle in handles {
            if let Ok(debug_support) = bt.handle_protocol::<DebugSupport>(handle) {
                let debug_support = debug_support
                    .expect("Warnings encountered while opening debug support protocol");
                let debug_support = unsafe { &mut *debug_support.get() };

                // make sure that the max processor index is a sane value, i.e. it works
                let maximum_processor_index = debug_support.get_maximum_processor_index();
                assert_ne!(
                    maximum_processor_index,
                    usize::MAX,
                    "get_maximum_processor_index() returning garbage or not working"
                );

                info!("- Architecture: {:?}", debug_support.arch());
                info!("- Maximum Processor Index: {:?}", maximum_processor_index);

                test_register_periodic_callback(debug_support);
                test_deregister_periodic_callback(debug_support);

                match debug_support.arch() {
                    ProcessorArch::EBC => {
                        // for the EBC Debug Support Protocol, there are already exception callbacks registered
                        test_deregister_exception_callback(debug_support);
                        test_register_exception_callback(debug_support);
                    }
                    _ => {
                        test_register_exception_callback(debug_support);
                        test_deregister_exception_callback(debug_support);
                    }
                }

                test_invalidate_instruction_cache(debug_support);
            }
        }
    } else {
        warn!("Debug protocol is not supported");
    }
}

#[allow(unused_must_use)]
fn test_register_periodic_callback(debug_support: &mut DebugSupport) {
    info!("Registering periodic callback");
    unsafe {
        debug_support
            .register_periodic_callback(0, Some(periodic_callback))
            .expect("Error while registering periodic callback");
    }
}

#[allow(unused_must_use)]
fn test_deregister_periodic_callback(debug_support: &mut DebugSupport) {
    info!("Deregistering periodic callback");
    unsafe {
        debug_support
            .register_periodic_callback(0, None)
            .expect("Error while deregistering periodic callback");
    }
}

#[allow(unused_must_use)]
fn test_register_exception_callback(debug_support: &mut DebugSupport) {
    info!("Registering exception callback");
    unsafe {
        debug_support
            .register_exception_callback(0, Some(exception_callback), 1)
            .expect("Error while registering exception callback");
    }
}

#[allow(unused_must_use)]
fn test_deregister_exception_callback(debug_support: &mut DebugSupport) {
    info!("Deregistering exception callback");
    unsafe {
        debug_support
            .register_exception_callback(0, None, 1)
            .expect("Error while deregistering exception callback");
    }
}

#[allow(unused_must_use)]
/// Should always pass, since the spec says this always returns EFI_SUCCESS
fn test_invalidate_instruction_cache(debug_support: &mut DebugSupport) {
    info!("Invalidating instruction cache");
    let mut addr = 0x0;
    let ptr = &mut addr as *mut _ as *mut c_void;

    unsafe {
        debug_support
            .invalidate_instruction_cache(0, ptr, 64)
            .expect("Error occured while invalidating instruction cache");
    }
}

// FIXME: Maybe turn into a closure?
extern "efiapi" fn periodic_callback(context: SystemContext) {
    let _ = context;
}

// FIXME: Maybe turn into a closure?
extern "efiapi" fn exception_callback(exception_type: ExceptionType, context: SystemContext) {
    let _ = exception_type;
    let _ = context;
}
