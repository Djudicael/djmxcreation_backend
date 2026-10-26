use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn embed(root: &Path, dir: &Path, entries: &mut Vec<String>) {
    let mut paths: Vec<_> = fs::read_dir(dir)
        .expect("read frontend assets")
        .map(|p| p.unwrap().path())
        .collect();
    paths.sort();
    for path in paths {
        assert!(!path.is_symlink(), "frontend assets must not be symlinks");
        if path.is_dir() {
            embed(root, &path, entries);
        } else if path.extension().is_none_or(|ext| ext != "map") {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            entries.push(format!(
                "({relative:?}, include_bytes!({:?})),",
                path.to_str().unwrap()
            ));
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut generated = String::new();
    for app in ["admin", "portfolio"] {
        let root = manifest.join(format!("../../front/{app}/dist"));
        println!("cargo:rerun-if-changed={}", root.display());
        for required in ["index.html", "js/index.js", "style.css"] {
            assert!(
                root.join(required).is_file(),
                "missing {app}/{required}: run npm run build before building the frontend server"
            );
        }
        let mut entries = Vec::new();
        embed(&root, &root, &mut entries);
        generated.push_str(&format!(
            "static {}: &[(&str, &[u8])] = &[{}];\n",
            app.to_uppercase(),
            entries.join("\n")
        ));
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("assets.rs"),
        generated,
    )
    .unwrap();
}
