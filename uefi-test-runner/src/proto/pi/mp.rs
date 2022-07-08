use core::ffi::c_void;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use uefi::proto::pi::mp::MpServices;
use uefi::table::boot::BootServices;
use uefi::Status;

/// Number of cores qemu is configured to have
const NUM_CPUS: usize = 4;

pub fn test(bt: &BootServices) {
    // These tests break CI. See #103.
    if cfg!(feature = "ci") {
        return;
    }

    info!("Running UEFI multi-processor services protocol test");
    if let Ok(handle) = bt.get_handle_for_protocol::<MpServices>() {
        let mp_support = &bt
            .open_protocol_exclusive::<MpServices>(handle)
            .expect("failed to open multi-processor services protocol");

        test_get_number_of_processors(mp_support);
        test_get_processor_info(mp_support);
        test_startup_all_aps(mp_support, bt);
        test_startup_this_ap(mp_support, bt);
        test_enable_disable_ap(mp_support);
        test_switch_bsp_and_who_am_i(mp_support);
    } else {
        warn!("Multi-processor services protocol is not supported");
    }
}

fn test_get_number_of_processors(mps: &MpServices) {
    let proc_count = mps.get_number_of_processors().unwrap();

    // Ensure we can see all of the requested CPUs
    assert_eq!(proc_count.total, NUM_CPUS);

    // All CPUs should be enabled
    assert_eq!(proc_count.total, proc_count.enabled);
}

fn test_get_processor_info(mps: &MpServices) {
    // Disable second CPU for this test
    mps.enable_disable_ap(1, false, None).unwrap();

    // Retrieve processor information from each CPU
    let cpu0 = mps.get_processor_info(0).unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap();
    let cpu2 = mps.get_processor_info(2).unwrap();

    // Check that processor_id fields are sane
    assert_eq!(cpu0.processor_id, 0);
    assert_eq!(cpu1.processor_id, 1);
    assert_eq!(cpu2.processor_id, 2);

    // Check that only CPU 0 is BSP
    assert!(cpu0.is_bsp());
    assert!(!cpu1.is_bsp());
    assert!(!cpu2.is_bsp());

    // Check that only the second CPU is disabled
    assert!(cpu0.is_enabled());
    assert!(!cpu1.is_enabled());
    assert!(cpu2.is_enabled());

    // Enable second CPU back
    mps.enable_disable_ap(1, true, None).unwrap();
}

extern "efiapi" fn proc_increment_atomic(arg: *mut c_void) {
    let counter: &AtomicUsize = unsafe { &*(arg as *const _) };
    counter.fetch_add(1, Ordering::Relaxed);
}

extern "efiapi" fn proc_wait_100ms(arg: *mut c_void) {
    let bt: &BootServices = unsafe { &*(arg as *const _) };
    bt.stall(100_000);
}

fn test_startup_all_aps(mps: &MpServices, bt: &BootServices) {
    // Ensure that APs start up
    let counter = AtomicUsize::new(0);
    let counter_ptr: *mut c_void = &counter as *const _ as *mut _;
    mps.startup_all_aps(false, proc_increment_atomic, counter_ptr, None)
        .unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), NUM_CPUS - 1);

    // Make sure that timeout works
    let bt_ptr: *mut c_void = bt as *const _ as *mut _;
    let ret = mps.startup_all_aps(
        false,
        proc_wait_100ms,
        bt_ptr,
        Some(Duration::from_millis(50)),
    );
    assert_eq!(ret.map_err(|err| err.status()), Err(Status::TIMEOUT));
}

fn test_startup_this_ap(mps: &MpServices, bt: &BootServices) {
    // Ensure that each AP starts up
    let counter = AtomicUsize::new(0);
    let counter_ptr: *mut c_void = &counter as *const _ as *mut _;
    for i in 1..NUM_CPUS {
        mps.startup_this_ap(i, proc_increment_atomic, counter_ptr, None)
            .unwrap();
    }
    assert_eq!(counter.load(Ordering::Relaxed), NUM_CPUS - 1);

    // Make sure that timeout works for each AP
    let bt_ptr: *mut c_void = bt as *const _ as *mut _;
    for i in 1..NUM_CPUS {
        let ret = mps.startup_this_ap(i, proc_wait_100ms, bt_ptr, Some(Duration::from_millis(50)));
        assert_eq!(ret.map_err(|err| err.status()), Err(Status::TIMEOUT));
    }
}

fn test_enable_disable_ap(mps: &MpServices) {
    // Disable second CPU
    mps.enable_disable_ap(1, false, None).unwrap();

    // Ensure that one CPUs is disabled
    let proc_count = mps.get_number_of_processors().unwrap();
    assert_eq!(proc_count.total - proc_count.enabled, 1);

    // Enable second CPU back
    mps.enable_disable_ap(1, true, None).unwrap();

    // Ensure that all CPUs are enabled
    let proc_count = mps.get_number_of_processors().unwrap();
    assert_eq!(proc_count.total, proc_count.enabled);

    // Mark second CPU as unhealthy and check it's status
    mps.enable_disable_ap(1, true, Some(false)).unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap();
    assert!(!cpu1.is_healthy());

    // Mark second CPU as healthy again and check it's status
    mps.enable_disable_ap(1, true, Some(true)).unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap();
    assert!(cpu1.is_healthy());
}

fn test_switch_bsp_and_who_am_i(mps: &MpServices) {
    // Normally BSP starts on on CPU 0
    let proc_number = mps.who_am_i().unwrap();
    assert_eq!(proc_number, 0);

    // Do a BSP switch
    mps.switch_bsp(1, true).unwrap();

    // We now should be on CPU 1
    let proc_number = mps.who_am_i().unwrap();
    assert_eq!(proc_number, 1);

    // Switch back
    mps.switch_bsp(0, true).unwrap();

    // We now should be on CPU 0 again
    let proc_number = mps.who_am_i().unwrap();
    assert_eq!(proc_number, 0);
}
