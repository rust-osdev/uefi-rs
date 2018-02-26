//! Support for QEMU's debug output port.

pub fn output_string(s: &str) {
    for b in s.as_bytes() {
        unsafe {
            asm!("outb %al, $$0xE9" : : "{al}"(*b));
        }
    }
}

pub fn print_ok(ok: &str) {
    output_string("OK : ");
    output_string(ok);
    output_string("\n");
}

pub fn print_err(err: &str) {
    output_string("ERR: ");
    output_string(err);
    output_string("\n");
}
