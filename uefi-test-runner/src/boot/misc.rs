use core::ffi::c_void;
use core::ptr::{self, NonNull};

use core::mem;
use uefi::mem::memory_map::MemoryType;
use uefi::proto::unsafe_protocol;
use uefi::table::boot::{
    BootServices, EventType, OpenProtocolAttributes, OpenProtocolParams, SearchType, TimerTrigger,
    Tpl,
};
use uefi::table::{Boot, SystemTable};
use uefi::{boot, guid, system, Event, Guid, Identify, Status};

pub fn test(st: &SystemTable<Boot>) {
    let bt = st.boot_services();
    test_tpl();
    info!("Testing timer...");
    test_timer(bt);
    info!("Testing events...");
    test_check_event_freestanding();
    test_timer_freestanding();
    test_event_callback(bt);
    test_callback_with_ctx(bt);
    info!("Testing watchdog...");
    test_watchdog(bt);
    info!("Testing protocol handler services...");
    test_register_protocol_notify(bt);
    test_protocol_interface_management();
    test_install_protocol_interface(bt);
    test_reinstall_protocol_interface(bt);
    test_uninstall_protocol_interface(bt);
    test_install_configuration_table(st);
    test_install_configuration_table_freestanding();
}

fn test_tpl() {
    info!("Testing watchdog...");
    // There's no way to query the TPL, so we can't assert that this does anything.
    let _guard = unsafe { boot::raise_tpl(Tpl::NOTIFY) };
}

fn test_check_event_freestanding() {
    extern "efiapi" fn callback(_event: Event, _ctx: Option<NonNull<c_void>>) {
        info!("Callback triggered by check_event");
    }

    let event =
        unsafe { boot::create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(callback), None) }
            .unwrap();

    let event_clone = unsafe { event.unsafe_clone() };
    let is_signaled = boot::check_event(event_clone).unwrap();
    assert!(!is_signaled);

    boot::close_event(event).unwrap();
}

fn test_timer_freestanding() {
    let timer_event =
        unsafe { boot::create_event(EventType::TIMER, Tpl::CALLBACK, None, None) }.unwrap();
    let mut events = unsafe { [timer_event.unsafe_clone()] };
    boot::set_timer(&timer_event, TimerTrigger::Relative(5_0 /*00 ns */)).unwrap();
    assert_eq!(boot::wait_for_event(&mut events).unwrap(), 0);

    boot::close_event(timer_event).unwrap();
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
    // There's no way to check the watchdog timer value, so just test setting it.

    // Disable the UEFI watchdog timer.
    bt.set_watchdog_timer(0, 0x10000, None)
        .expect("Could not set watchdog timer");

    // Set the timer with the freestanding function.
    boot::set_watchdog_timer(240, 0x10000, None).expect("Could not set watchdog timer");
}

/// Dummy protocol for tests
#[unsafe_protocol("1a972918-3f69-4b5d-8cb4-cece2309c7f5")]
struct TestProtocol {
    data: u32,
}

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

fn test_protocol_interface_management() {
    let mut interface = TestProtocol { data: 123 };
    let interface_ptr: *mut _ = &mut interface;

    // Install the protocol.
    let handle = unsafe {
        boot::install_protocol_interface(None, &TestProtocol::GUID, interface_ptr.cast())
    }
    .unwrap();

    // Verify the handle was installed.
    assert_eq!(
        &*boot::locate_handle_buffer(SearchType::from_proto::<TestProtocol>()).unwrap(),
        [handle]
    );

    // Re-install the protocol.
    unsafe {
        boot::reinstall_protocol_interface(
            handle,
            &TestProtocol::GUID,
            interface_ptr.cast(),
            interface_ptr.cast(),
        )
    }
    .unwrap();

    // Uninstall the protocol.
    unsafe {
        boot::uninstall_protocol_interface(handle, &TestProtocol::GUID, interface_ptr.cast())
    }
    .unwrap();

    // Verify the protocol was uninstalled.
    assert_eq!(
        boot::locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
            .unwrap_err()
            .status(),
        Status::NOT_FOUND
    );
}

fn test_install_protocol_interface(bt: &BootServices) {
    info!("Installing TestProtocol");

    let alloc: *mut TestProtocol = bt
        .allocate_pool(
            MemoryType::BOOT_SERVICES_DATA,
            mem::size_of::<TestProtocol>(),
        )
        .unwrap()
        .cast()
        .as_ptr();
    unsafe { alloc.write(TestProtocol { data: 123 }) };

    let _ = unsafe {
        bt.install_protocol_interface(None, &TestProtocol::GUID, alloc.cast())
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
        // Uninstalling a protocol interface requires knowing the interface
        // pointer. Open the protocol to get that pointer, making sure to drop
        // the `ScopedProtocol` _before_ uninstalling the protocol interface.
        let interface_ptr: *mut TestProtocol = {
            let mut sp = boot::open_protocol::<TestProtocol>(
                OpenProtocolParams {
                    handle,
                    agent: bt.image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .unwrap();
            assert_eq!(sp.data, 123);
            &mut *sp
        };

        bt.uninstall_protocol_interface(handle, &TestProtocol::GUID, interface_ptr.cast())
            .expect("Failed to uninstall protocol interface");

        bt.free_pool(interface_ptr.cast()).unwrap();
    }
}

fn test_install_configuration_table_freestanding() {
    // Get the current number of entries.
    let count = system::with_config_table(|t| t.len());

    // Create the entry data.
    let config = boot::allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, 1)
        .unwrap()
        .as_ptr();
    unsafe { config.write(42) };

    // Install the table.
    const ID: Guid = guid!("4bec53c4-5fc1-48a1-ab12-df214907d29f");
    unsafe {
        boot::install_configuration_table(&ID, config.cast()).unwrap();
    }

    // Verify the installation.
    assert_eq!(count + 1, system::with_config_table(|t| t.len()));
    system::with_config_table(|t| {
        let config_entry = t.iter().find(|ct| ct.guid == ID).unwrap();
        assert_eq!(unsafe { *(config_entry.address as *const u8) }, 42);
    });

    // Uninstall the table.
    unsafe {
        boot::install_configuration_table(&ID, ptr::null()).unwrap();
    }
}

fn test_install_configuration_table(st: &SystemTable<Boot>) {
    let config = st
        .boot_services()
        .allocate_pool(MemoryType::ACPI_RECLAIM, 1)
        .expect("Failed to allocate config table")
        .as_ptr();
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
