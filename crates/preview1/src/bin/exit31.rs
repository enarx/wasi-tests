#[link_section = ".test.data"]
pub static DATA: [u8; 11] = *b"
exit = 31
";

fn main() {
    unsafe { wasi::proc_exit(31) };
}
