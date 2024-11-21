use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get OUT_DIR environment variable
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let templates_dir = Path::new(&out_dir).join("templates");

    // Create templates directory in OUT_DIR
    fs::create_dir_all(&templates_dir).unwrap();

    // Get the project root directory (where Cargo.toml lives)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_templates = Path::new(&manifest_dir).join("templates");
    println!("{manifest_dir}");
    println!("{}", project_templates.display());
    // List of template files to copy
    let template_files = [
        "config.toml",
        "example.md",
        "base.html",
        "index.html",
        "post.html",
        "style.css",
        "terminal_velocity.png",
    ];

    // Copy each template file
    for &file in &template_files {
        let src = project_templates.join(file);
        let dest = templates_dir.join(file);

        fs::copy(&src, &dest)
            .unwrap_or_else(|_| panic!("Failed to copy template file: {}", src.display()));
    }

    // Tell Cargo to rerun this build script if templates change
    println!("cargo:rerun-if-changed=templates/");
}
