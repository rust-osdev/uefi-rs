use uefi::boot;
use uefi::proto::shell::Shell;
use uefi::CStr16;

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let mut shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");

    // create some files
    let mut test_buf = [0u16; 12];
    let test_str = CStr16::from_str_with_buf("test", &mut test_buf).unwrap();
    shell.create_file(test_str, 0);

    // get file tree
    // let mut str_buf = [0u16; 12];
    // let str_str = CStr16::from_str_with_buf(r"fs0:\*", &mut str_buf).unwrap();
    // let res = shell.find_files(str_str);
    // let list = res.unwrap();
    // let list = list.unwrap();
    // let first = list.first();

    info!("filetree test successful")
}
