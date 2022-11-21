#[link_section = ".test.data"]
pub static DATA: [u8; 10] = *b"
exit = 1
";

fn main() {
    unsafe { wasi::proc_exit(1) };
}
