use std::{env, fs, path::Path};
fn collect(root: &Path, dir: &Path, out: &mut String) {
    let mut entries = fs::read_dir(dir)
        .expect("Export the UI before building the desktop app")
        .map(|v| v.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            let name = format!(
                "/{}",
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
            );
            let mime = match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
                "html" => "text/html; charset=utf-8",
                "js" => "text/javascript; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "png" => "image/png",
                "svg" => "image/svg+xml",
                "ico" => "image/x-icon",
                "ttf" => "font/ttf",
                "woff2" => "font/woff2",
                "json" => "application/json",
                _ => "application/octet-stream",
            };
            out.push_str(&format!(
                "({name:?},{mime:?},include_bytes!({:?})),\n",
                path.canonicalize().unwrap().to_str().unwrap()
            ));
        }
    }
}
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut out = String::from("pub static ASSETS: &[(&str,&str,&[u8])] = &[\n");
    collect(&root, &root, &mut out);
    out.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        out,
    )
    .unwrap();
}
