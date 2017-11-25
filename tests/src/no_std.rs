extern crate rlibc;
extern crate compiler_builtins;

#[lang = "eh_personality"]
fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
#[allow(private_no_mangle_fns)]
pub fn panic_fmt() {}
