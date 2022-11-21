#[link_section = ".test.data"]
pub static DATA: [u8; 42] = *b"
stdout = '|/|/foo|'
dirs = ['/', '/foo']
";

fn main() {
    use std::io::Write;

    print!("|");
    for fd in 3.. {
        // Attempt to get the length of the prestat name.
        let prestat = match unsafe { wasi::fd_prestat_get(fd) } {
            Ok(prestat) => prestat,
            Err(errno) if errno == wasi::ERRNO_BADF => break,
            Err(errno) => panic!("unexpected error: {}", errno),
        };

        // Attempt to get the name itself.
        assert_eq!(prestat.tag, wasi::PREOPENTYPE_DIR.raw());
        let mut name = vec![0u8; unsafe { prestat.u.dir.pr_name_len } as usize];
        unsafe { wasi::fd_prestat_dir_name(fd, name.as_mut_ptr(), name.len()) }
            .expect("failed to get prestat name");

        // Dump the name to stdout.
        std::io::stdout().write_all(&name).unwrap();
        print!("|");
    }
}
