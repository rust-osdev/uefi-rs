use core::ffi::c_void;
use core::ptr::{self, NonNull};

use uefi::proto::unsafe_protocol;
use uefi::table::boot::{BootServices, EventType, MemoryType, SearchType, TimerTrigger, Tpl};
use uefi::table::{Boot, SystemTable};
use uefi::{guid, Event, Guid, Identify};

pub fn test(st: &SystemTable<Boot>) {
    let bt = st.boot_services();
    info!("Testing timer...");
    test_timer(bt);
    info!("Testing events...");
    test_event_callback(bt);
    test_callback_with_ctx(bt);
    info!("Testing watchdog...");
    test_watchdog(bt);
    info!("Testing protocol handler services...");
    test_register_protocol_notify(bt);
    test_install_protocol_interface(bt);
    test_reinstall_protocol_interface(bt);
    test_uninstall_protocol_interface(bt);
    test_install_configuration_table(st);
}

fn test_timer(bt: &BootServices) {
    let timer_event = unsafe { bt.create_event(EventType::TIMER, Tpl::APPLICATION, None, None) }
        .expect("Failed to create TIMER event");
    let mut events = unsafe { [timer_event.unsafe_clone()] };
    bt.set_timer(&timer_event, TimerTrigger::Relative(5_0 /*00 ns */))
        .expect("Failed to set timer");
    bt.wait_for_event(&mut events)
        .expect("Wait for event failed");
}

fn test_event_callback(bt: &BootServices) {
    extern "efiapi" fn callback(_event: Event, _ctx: Option<NonNull<c_void>>) {
        info!("Inside the event callback");
    }

    let event =
        unsafe { bt.create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(callback), None) }
            .expect("Failed to create custom event");
    bt.check_event(event).expect("Failed to check event");
}

fn test_callback_with_ctx(bt: &BootServices) {
    let mut data = 123u32;

    extern "efiapi" fn callback(_event: Event, ctx: Option<NonNull<c_void>>) {
        info!("Inside the event callback with context");
        // Safety: this callback is run within the parent function's
        // scope, so the context pointer is still valid.
        unsafe {
            let ctx = ctx.unwrap().as_ptr().cast::<u32>();
            *ctx = 456;
        }
    }

    let ctx: *mut u32 = &mut data;
    let ctx = NonNull::new(ctx.cast::<c_void>()).unwrap();

    let event = unsafe {
        bt.create_event(
            EventType::NOTIFY_WAIT,
            Tpl::CALLBACK,
            Some(callback),
            Some(ctx),
        )
        .expect("Failed to create event with context")
    };

    bt.check_event(event).expect("Failed to check event");

    // Check that `data` was updated inside the event callback.
    assert_eq!(data, 456);
}

fn test_watchdog(bt: &BootServices) {
    // Disable the UEFI watchdog timer
    bt.set_watchdog_timer(0, 0x10000, None)
        .expect("Could not set watchdog timer");
}

/// Dummy protocol for tests
#[unsafe_protocol("1a972918-3f69-4b5d-8cb4-cece2309c7f5")]
struct TestProtocol {}

unsafe extern "efiapi" fn _test_notify(_event: Event, _context: Option<NonNull<c_void>>) {
    info!("Protocol was (re)installed and this function notified.")
}

fn test_register_protocol_notify(bt: &BootServices) {
    let protocol = &TestProtocol::GUID;
    let event = unsafe {
        bt.create_event(
            EventType::NOTIFY_SIGNAL,
            Tpl::NOTIFY,
            Some(_test_notify),
            None,
        )
        .expect("Failed to create an event")
    };

    bt.register_protocol_notify(protocol, event)
        .expect("Failed to register protocol notify fn");
}

fn test_install_protocol_interface(bt: &BootServices) {
    info!("Installing TestProtocol");

    let _ = unsafe {
        bt.install_protocol_interface(None, &TestProtocol::GUID, ptr::null_mut())
            .expect("Failed to install protocol interface")
    };

    let _ = bt
        .locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol after it was installed");
}

fn test_reinstall_protocol_interface(bt: &BootServices) {
    info!("Reinstalling TestProtocol");
    let handle = bt
        .locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol to uninstall")[0];

    unsafe {
        let _ = bt.reinstall_protocol_interface(
            handle,
            &TestProtocol::GUID,
            ptr::null_mut(),
            ptr::null_mut(),
        );
    }
}

fn test_uninstall_protocol_interface(bt: &BootServices) {
    info!("Uninstalling TestProtocol");
    let handle = bt
        .locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol to uninstall")[0];

    unsafe {
        bt.uninstall_protocol_interface(handle, &TestProtocol::GUID, ptr::null_mut())
            .expect("Failed to uninstall protocol interface");
    }
}

fn test_install_configuration_table(st: &SystemTable<Boot>) {
    let config = st
        .boot_services()
        .allocate_pool(MemoryType::ACPI_RECLAIM, 1)
        .expect("Failed to allocate config table");
    unsafe { config.write(42) };

    let count = st.config_table().len();
    const ID: Guid = guid!("3bdb3089-5662-42df-840e-3922ed6467c9");

    unsafe {
        st.boot_services()
            .install_configuration_table(&ID, config.cast())
            .expect("Failed to install configuration table");
    }

    assert_eq!(count + 1, st.config_table().len());
    let config_entry = st
        .config_table()
        .iter()
        .find(|ct| ct.guid == ID)
        .expect("Failed to find test config table");
    assert_eq!(unsafe { *(config_entry.address as *const u8) }, 42);
}
