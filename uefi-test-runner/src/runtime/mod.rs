mod vars;

use uefi::runtime::{self, Daylight, Time, TimeParams};
use uefi::table::runtime::RuntimeServices;

pub fn test(rt: &RuntimeServices) {
    info!("Testing runtime services");
    vars::test(rt);
    test_time();
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
    info!("After setting time: {}", now);
    assert_eq!(now.year(), 2020);
}
