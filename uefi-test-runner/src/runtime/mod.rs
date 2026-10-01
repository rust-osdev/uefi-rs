// SPDX-License-Identifier: MIT OR Apache-2.0

mod vars;

use uefi::Status;
use uefi::runtime::{self, Daylight, Time, TimeParams};

pub fn test() {
    info!("Testing runtime services");
    vars::test();
    test_time();
    test_monotonic_count();
}

fn test_time() {
    // Print the current time and time capabilities.
    info!(
        "Time with caps: {:?}",
        runtime::get_time_and_caps().unwrap()
    );

    // Set the time.
    let time = Time::new(TimeParams {
        year: 2020,
        month: 1,
        day: 2,
        hour: 3,
        minute: 4,
        second: 5,
        nanosecond: 6,
        time_zone: None,
        daylight: Daylight::ADJUST_DAYLIGHT,
    })
    .unwrap();
    unsafe { runtime::set_time(&time).unwrap() };

    // Print the new time and check that the year was successfully changed.
    let now = runtime::get_time().unwrap();
    info!("After setting time: {now}");
    assert_eq!(now.year(), 2020);
}

fn test_monotonic_count() {
    match runtime::get_next_high_monotonic_count() {
        Ok(high_count) => info!("High monotonic count: {high_count}"),
        Err(err) if matches!(err.status(), Status::UNSUPPORTED) => {
            info!("High monotonic count not available: {}", err.status());
        }
        Err(err) => panic!("failed to get high monotonic count: {err:?}"),
    }
}
