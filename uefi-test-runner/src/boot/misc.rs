use uefi::prelude::*;
use uefi::table::boot::{BootServices, EventType, Tpl, TimerTrigger};

pub fn test(bt: &BootServices) {
    info!("Testing timer...");
    test_timer(bt);
    info!("Testing watchdog...");
    test_watchdog(bt);
}

fn test_watchdog(bt: &BootServices) {
    // Disable the UEFI watchdog timer
    bt.set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");
}

fn test_timer(bt: &BootServices) {
    let timer_event = unsafe { bt.create_event(EventType::TIMER, Tpl::APPLICATION, None) }
        .expect_success("Failed to create TIMER event");
    let mut events = [timer_event];
    bt.set_timer(timer_event, TimerTrigger::Relative(5_0/*00 ns */))
        .expect_success("Failed to set timer");
    bt.wait_for_event(&mut events)
        .expect_success("Wait for event failed");
}
