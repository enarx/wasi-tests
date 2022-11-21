#[link_section = ".test.data"]
pub static DATA: [u8; 16] = *b"
stdout = 'foo'
";

use std::io::Write;

fn main() {
    std::io::stdout().write_all(b"foo").unwrap();
}
