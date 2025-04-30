// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi::c_void;
use core::ptr::{self, NonNull};

use uefi::boot::{
    EventType, OpenProtocolAttributes, OpenProtocolParams, SearchType, TimerTrigger, Tpl,
};
use uefi::mem::memory_map::MemoryType;
use uefi::proto::unsafe_protocol;
use uefi::{boot, guid, system, Event, Guid, Identify};

pub fn test() {
    test_tpl();
    info!("Testing timer...");
    test_timer();
    info!("Testing events...");
    test_check_event();
    test_callback_with_ctx();
    test_signal_event();
    info!("Testing watchdog...");
    test_watchdog();
    info!("Testing protocol handler services...");
    test_register_protocol_notify();
    test_install_protocol_interface();
    test_reinstall_protocol_interface();
    test_uninstall_protocol_interface();
    test_install_configuration_table();
    info!("Testing crc32...");
    test_calculate_crc32();
}

fn test_tpl() {
    info!("Testing watchdog...");
    // There's no way to query the TPL, so we can't assert that this does anything.
    let _guard = unsafe { boot::raise_tpl(Tpl::NOTIFY) };
}

fn test_check_event() {
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

fn test_timer() {
    let timer_event =
        unsafe { boot::create_event_ex(EventType::TIMER, Tpl::CALLBACK, None, None, None) }
            .unwrap();
    let mut events = unsafe { [timer_event.unsafe_clone()] };
    boot::set_timer(&timer_event, TimerTrigger::Relative(5_0 /*00 ns */)).unwrap();
    assert_eq!(boot::wait_for_event(&mut events).unwrap(), 0);

    boot::close_event(timer_event).unwrap();
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

fn test_signal_event() {
    let mut data = 123u32;

    extern "efiapi" fn callback(_event: Event, ctx: Option<NonNull<c_void>>) {
        info!("Inside the signal event callback");
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
            EventType::NOTIFY_SIGNAL,
            Tpl::CALLBACK,
            Some(callback),
            Some(ctx),
        )
        .expect("Failed to create event with context")
    };

    boot::signal_event(&event).expect("Failed to signal event");

    // Check that `data` was updated inside the event callback.
    assert_eq!(data, 456);
}

fn test_watchdog() {
    // There's no way to check the watchdog timer value, so just test setting it.

    // Disable the UEFI watchdog timer.
    boot::set_watchdog_timer(0, 0x10000, None).expect("Could not set watchdog timer");
}

/// Dummy protocol for tests
#[unsafe_protocol("1a972918-3f69-4b5d-8cb4-cece2309c7f5")]
struct TestProtocol {
    data: u32,
}

fn test_register_protocol_notify() {
    unsafe extern "efiapi" fn callback(_event: Event, _context: Option<NonNull<c_void>>) {
        info!("in callback for test_register_protocol_notify")
    }

    let protocol = &TestProtocol::GUID;
    let event = unsafe {
        boot::create_event(EventType::NOTIFY_SIGNAL, Tpl::NOTIFY, Some(callback), None).unwrap()
    };

    boot::register_protocol_notify(protocol, &event)
        .expect("Failed to register protocol notify fn");
}

fn test_install_protocol_interface() {
    info!("Installing TestProtocol");

    let alloc: *mut TestProtocol =
        boot::allocate_pool(MemoryType::BOOT_SERVICES_DATA, size_of::<TestProtocol>())
            .unwrap()
            .cast()
            .as_ptr();
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

        boot::free_pool(NonNull::new(interface_ptr.cast()).unwrap()).unwrap();
    }
}

fn test_install_configuration_table() {
    // Get the current number of entries.
    let initial_table_count = system::with_config_table(|t| t.len());

    // Create the entry data.
    let config: NonNull<u8> = boot::allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, 1).unwrap();
    unsafe { config.write(123u8) };

    // Install the table.
    const TABLE_GUID: Guid = guid!("4bec53c4-5fc1-48a1-ab12-df214907d29f");
    unsafe {
        boot::install_configuration_table(&TABLE_GUID, config.as_ptr().cast()).unwrap();
    }

    // Verify the installation.
    assert_eq!(
        initial_table_count + 1,
        system::with_config_table(|t| t.len())
    );
    system::with_config_table(|t| {
        let config_entry = t.iter().find(|ct| ct.guid == TABLE_GUID).unwrap();
        assert_eq!(unsafe { *config_entry.address.cast::<u8>() }, 123);
    });

    // Uninstall the table and free the memory.
    unsafe {
        boot::install_configuration_table(&TABLE_GUID, ptr::null()).unwrap();
        boot::free_pool(config).unwrap();
    }
}

fn test_calculate_crc32() {
    let data = "uefi-rs";

    let crc = boot::calculate_crc32(data.as_bytes()).unwrap();

    assert_eq!(crc, 0xcfc96a3e);
}
