#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod server;
use ainavlog_local_storage::ObservationStore;
use std::{
    path::{Path, PathBuf},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

fn data_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(path).join("aiNavLog").join("Desktop"));
    }
    std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join(".local/share/ainavlog-desktop"))
        .ok_or("No application data directory".into())
}
#[cfg(windows)]
fn open_window(url: &str, directory: &Path) -> Result<Child, String> {
    let mut candidates = Vec::new();
    for key in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
        if let Some(root) = std::env::var_os(key) {
            candidates.push(PathBuf::from(root).join("Microsoft/Edge/Application/msedge.exe"));
        }
    }
    let edge = candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or("Microsoft Edge is required to display this local test app.".to_string())?;
    std::process::Command::new(edge)
        .arg(format!("--app={url}"))
        .arg(format!(
            "--user-data-dir={}",
            directory.join("Browser").display()
        ))
        .args([
            "--no-first-run",
            "--disable-background-mode",
            "--window-size=1400,1000",
        ])
        .spawn()
        .map_err(|e| e.to_string())
}
#[cfg(not(windows))]
fn open_window(_url: &str, _directory: &Path) -> Result<Child, String> {
    Err("Use --serve-only for host testing; the application window targets Windows.".into())
}
#[cfg(windows)]
fn report_error(message: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            window: *mut std::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            kind: u32,
        ) -> i32;
    }
    let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let caption: Vec<u16> = "aiNavLog".encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
    }
}
#[cfg(not(windows))]
fn report_error(message: &str) {
    eprintln!("{message}");
}
fn run() -> Result<(), String> {
    let mut directory = data_directory()?;
    let mut port = 49378u16;
    let mut serve_only = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--serve-only" => serve_only = true,
            "--data-dir" => directory = PathBuf::from(args.next().ok_or("Missing data directory")?),
            "--port" => {
                port = args
                    .next()
                    .ok_or("Missing port")?
                    .parse()
                    .map_err(|_| "Invalid port")?
            }
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    if port == 0 {
        return Err("Choose a nonzero loopback port".into());
    }
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let database = directory.join("ainavlog.sqlite3");
    drop(ObservationStore::open(&database).map_err(|e| format!("Could not open local data: {e}"))?);
    let address = format!("127.0.0.1:{port}");
    let url = format!("http://{address}/");
    let listener = match std::net::TcpListener::bind(&address) {
        Ok(v) => v,
        Err(e) => {
            if !serve_only && server::existing_host(port) {
                let _ = open_window(&url, &directory)?;
                return Ok(());
            }
            return Err(format!("Could not start aiNavLog on {address}: {e}"));
        }
    };
    let token = server::new_token()?;
    println!("aiNavLog ready at {url}\nDatabase: {}", database.display());
    let stop = Arc::new(AtomicBool::new(false));
    if !serve_only {
        let mut window = open_window(&url, &directory)?;
        let done = Arc::clone(&stop);
        std::thread::spawn(move || {
            let _ = window.wait();
            done.store(true, Ordering::Release);
        });
    }
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(e) = server::handle(stream, &database, &address, &token) {
                    eprintln!("Local request: {e}");
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        report_error(&error);
        std::process::exit(1);
    }
}
