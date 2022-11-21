#[link_section = ".test.data"]
pub static DATA: [u8; 16] = *b"
stderr = 'foo'
";

use std::io::Write;

fn main() {
    std::io::stderr().write_all(b"foo").unwrap();
}
