use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_tests_toml::Environment;
use wasmtime::{AsContextMut, Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{add_to_linker, WasiCtxBuilder};

#[derive(Default, Clone)]
pub struct Buffer(Arc<RwLock<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Buffer {
    pub fn pipe(&self) -> WritePipe<Self> {
        WritePipe::new(self.clone())
    }

    pub fn into_string(&self) -> Result<String, std::str::Utf8Error> {
        Ok(std::str::from_utf8(&self.0.read().unwrap())?.to_string())
    }
}

fn runner(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read the .wasm file.
    let wasm = std::fs::read(path)?;

    // Extract the test environment from the wasm file.
    let env = Environment::load(&wasm)?;

    // Prepare wasmtime for execution.
    let config = Config::new();
    let engine = Engine::new(&config)?;
    let module = Module::from_binary(&engine, &wasm)?;
    let mut linker = Linker::new(&engine);
    add_to_linker(&mut linker, |s| s)?;

    // Instantiate the wasm module.
    let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
    let instance = linker.instantiate(&mut store, &module)?;
    let mut ctx = store.as_context_mut();

    // Set up stdio.
    let stdin = Cursor::new(env.stdin.into_bytes());
    let stdout = Buffer::default();
    let stderr = Buffer::default();
    ctx.data_mut().set_stdin(Box::new(ReadPipe::new(stdin)));
    ctx.data_mut().set_stdout(Box::new(stdout.pipe()));
    ctx.data_mut().set_stderr(Box::new(stderr.pipe()));

    // Set up the arguments.
    for arg in env.args {
        ctx.data_mut().args.push(arg.clone())?;
    }

    // Set up the environment variables.
    for (k, v) in env.vars {
        ctx.data_mut().env.push(format!("{}={}", k, v))?;
    }

    // Set up the preopened directories.
    let mut dirs = BTreeMap::new();
    for path in env.dirs {
        dirs.insert(path, tempfile::tempdir()?);
    }
    for (path, dir) in dirs.iter() {
        let file = std::fs::File::open(dir.path())?;
        let dir = wasmtime_wasi::Dir::from_std_file(file);
        let dir = wasmtime_wasi::sync::dir::Dir::from_cap_std(dir);
        ctx.data_mut().push_preopened_dir(Box::new(dir), path)?;
    }

    // Run the _start() function.
    let func = instance.get_func(&mut store, "_start").unwrap();
    let exit = func.call(&mut store, &[], &mut []);

    // Validate output.
    assert_eq!(env.stderr, stderr.into_string().unwrap());
    assert_eq!(env.stdout, stdout.into_string().unwrap());

    // Verify the exit code.
    match exit {
        Ok(()) => assert_eq!(env.exit, 0),
        Err(error) => match error.downcast_ref::<wasmtime::Trap>() {
            Some(trap) if trap.i32_exit_status() == Some(env.exit) => {}
            _ => panic!("unexpected error error: {:?}", error),
        },
    }

    // Manually drop the temporary directories.
    // This ensures they stay open throughout the test.
    for (.., dir) in dirs {
        dir.close().unwrap();
    }

    Ok(())
}

fn main() {
    use libtest_mimic::{Arguments, Trial};

    const PREFIX: &str = "CARGO_BIN_FILE_WASI_TESTS_BINS_";

    let args = Arguments::from_args();

    let mut tests = Vec::new();
    for (k, v) in std::env::vars() {
        if let Some(suffix) = k.strip_prefix(PREFIX) {
            let name = format!("{}::{}", module_path!(), &suffix);
            tests.push(Trial::test(name, move || {
                runner(v.as_ref()).map_err(|e| e.into())
            }));
        }
    }

    libtest_mimic::run(&args, tests).exit()
}
