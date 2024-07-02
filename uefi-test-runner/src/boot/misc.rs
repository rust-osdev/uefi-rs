use core::ffi::c_void;
use core::ptr::NonNull;

use uefi::table::boot::{BootServices, EventType, TimerTrigger, Tpl};
use uefi::Event;

pub fn test(bt: &BootServices) {
    info!("Testing timer...");
    test_timer(bt);
    info!("Testing events...");
    test_event_callback(bt);
    test_callback_with_ctx(bt);
    info!("Testing watchdog...");
    test_watchdog(bt);
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
