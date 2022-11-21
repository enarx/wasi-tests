#[link_section = ".test.data"]
pub static DATA: [u8; 32] = *b"
stdout = '|.|..|'
dirs = ['/']
";

use std::io::Write;
use std::mem::{size_of, transmute};

fn main() {
    const LEN: usize = size_of::<wasi::Dirent>();

    let mut buf = [0u8; 4096];

    let n = unsafe { wasi::fd_readdir(3, buf.as_mut_ptr(), 1, 0) }.unwrap();
    assert_eq!(n, 1);

    let n = unsafe { wasi::fd_readdir(3, buf.as_mut_ptr(), buf.len(), 0) }.unwrap();
    assert_eq!(n, LEN * 2 + 3);

    print!("|");
    let mut offset = 0;
    while offset < n {
        // Get dirent.
        let mut bytes = [0u8; LEN];
        bytes.copy_from_slice(&buf[offset..][..LEN]);
        let dirent: wasi::Dirent = unsafe { transmute(bytes) };
        offset += LEN;

        // Get name.
        let namelen = dirent.d_namlen as usize;
        let name = &buf[offset..][..namelen];
        offset += namelen;

        std::io::stdout().write_all(name).unwrap();
        print!("|");
    }
}
