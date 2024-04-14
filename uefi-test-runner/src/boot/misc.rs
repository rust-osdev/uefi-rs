use core::ffi::c_void;
use core::ptr::{self, NonNull};

use core::mem;
use uefi::proto::unsafe_protocol;
use uefi::table::boot::{
    EventType, MemoryType, OpenProtocolAttributes, OpenProtocolParams, SearchType, TimerTrigger,
    Tpl,
};
use uefi::{boot, system};
use uefi::{guid, Event, Guid, Identify};

pub fn test() {
    info!("Testing timer...");
    test_timer();
    info!("Testing events...");
    test_event_callback();
    test_callback_with_ctx();
    info!("Testing watchdog...");
    test_watchdog();
    info!("Testing protocol handler services...");
    test_register_protocol_notify();
    test_install_protocol_interface();
    test_reinstall_protocol_interface();
    test_uninstall_protocol_interface();
    test_install_configuration_table();
}

fn test_timer() {
    let timer_event = unsafe { boot::create_event(EventType::TIMER, Tpl::APPLICATION, None, None) }
        .expect("Failed to create TIMER event");
    let mut events = unsafe { [timer_event.unsafe_clone()] };
    boot::set_timer(&timer_event, TimerTrigger::Relative(5_0 /*00 ns */))
        .expect("Failed to set timer");
    boot::wait_for_event(&mut events).expect("Wait for event failed");
}

fn test_event_callback() {
    extern "efiapi" fn callback(_event: Event, _ctx: Option<NonNull<c_void>>) {
        info!("Inside the event callback");
    }

    let event =
        unsafe { boot::create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(callback), None) }
            .expect("Failed to create custom event");
    boot::check_event(event).expect("Failed to check event");
}

fn test_callback_with_ctx() {
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
        boot::create_event(
            EventType::NOTIFY_WAIT,
            Tpl::CALLBACK,
            Some(callback),
            Some(ctx),
        )
        .expect("Failed to create event with context")
    };

    boot::check_event(event).expect("Failed to check event");

    // Check that `data` was updated inside the event callback.
    assert_eq!(data, 456);
}

fn test_watchdog() {
    // Disable the UEFI watchdog timer
    boot::set_watchdog_timer(0, 0x10000, None).expect("Could not set watchdog timer");
}

/// Dummy protocol for tests
#[unsafe_protocol("1a972918-3f69-4b5d-8cb4-cece2309c7f5")]
struct TestProtocol {
    data: u32,
}

unsafe extern "efiapi" fn _test_notify(_event: Event, _context: Option<NonNull<c_void>>) {
    info!("Protocol was (re)installed and this function notified.")
}

fn test_register_protocol_notify() {
    let protocol = &TestProtocol::GUID;
    let event = unsafe {
        boot::create_event(
            EventType::NOTIFY_SIGNAL,
            Tpl::NOTIFY,
            Some(_test_notify),
            None,
        )
        .expect("Failed to create an event")
    };

    boot::register_protocol_notify(protocol, event).expect("Failed to register protocol notify fn");
}

fn test_install_protocol_interface() {
    info!("Installing TestProtocol");

    let alloc: *mut TestProtocol = boot::allocate_pool(
        MemoryType::BOOT_SERVICES_DATA,
        mem::size_of::<TestProtocol>(),
    )
    .unwrap()
    .as_ptr()
    .cast();
    unsafe { alloc.write(TestProtocol { data: 123 }) };

    let _ = unsafe {
        boot::install_protocol_interface(None, &TestProtocol::GUID, alloc.cast())
            .expect("Failed to install protocol interface")
    };

    let _ = boot::locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol after it was installed");
}

fn test_reinstall_protocol_interface() {
    info!("Reinstalling TestProtocol");
    let handle = boot::locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol to uninstall")[0];

    unsafe {
        let _ = boot::reinstall_protocol_interface(
            handle,
            &TestProtocol::GUID,
            ptr::null_mut(),
            ptr::null_mut(),
        );
    }
}

fn test_uninstall_protocol_interface() {
    info!("Uninstalling TestProtocol");

    let handle = boot::locate_handle_buffer(SearchType::from_proto::<TestProtocol>())
        .expect("Failed to find protocol to uninstall")[0];

    unsafe {
        // Uninstalling a protocol interface requires knowing the interface
        // pointer. Open the protocol to get that pointer, making sure to drop
        // the `ScopedProtocol` _before_ uninstalling the protocol interface.
        let interface_ptr: *mut TestProtocol = {
            let mut sp = boot::open_protocol::<TestProtocol>(
                OpenProtocolParams {
                    handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .unwrap();
            assert_eq!(sp.data, 123);
            &mut *sp
        };

        boot::uninstall_protocol_interface(handle, &TestProtocol::GUID, interface_ptr.cast())
            .expect("Failed to uninstall protocol interface");

        boot::free_pool(interface_ptr.cast()).unwrap();
    }
}

fn test_install_configuration_table() {
    let config = boot::allocate_pool(MemoryType::ACPI_RECLAIM, 1)
        .expect("Failed to allocate config table")
        .as_ptr();
    unsafe { config.write(42) };

    let count = system::with_config_table(|t| t.len());
    const ID: Guid = guid!("3bdb3089-5662-42df-840e-3922ed6467c9");

    unsafe {
        boot::install_configuration_table(&ID, config.cast())
            .expect("Failed to install configuration table");
    }

    assert_eq!(count + 1, system::with_config_table(|t| t.len()));
    let entry_addr = system::with_config_table(|t| {
        t.iter()
            .find(|ct| ct.guid == ID)
            .expect("Failed to find test config table")
            .address
    });
    assert_eq!(unsafe { *(entry_addr as *const u8) }, 42);
}
