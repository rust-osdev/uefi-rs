use uefi::CStr16;
use uefi::prelude::BootServices;
use uefi::proto::shell::Shell;

pub fn test(bt: &BootServices) {
    info!("Running shell protocol tests");

    let handle = bt.get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let mut shell = bt
        .open_protocol_exclusive::<Shell>(handle)
        .expect("Failed to open Shell protocol");

    // create some files
    let mut test_buf = [0u16; 12];
    let test_str = CStr16::from_str_with_buf("test", &mut test_buf).unwrap();
    shell.create_file(test_str, 0);

    // get file tree
    let mut str_buf = [0u16; 12];
    let str_str = CStr16::from_str_with_buf(r"fs0:\*", &mut str_buf).unwrap();
    let res = shell.find_files(str_str);
    let list = res.unwrap();
    let list = list.unwrap();
    let first = list.first();

    info!("filetree test successful")
}