fn main() {
    if let Ok(vars) = dotenvy::from_filename_iter("../.env") {
        for(key, value) in vars.flatten() {
            println!("cargo:rustc-env={key}={value}");
        }
    }
    println!("cargo:rerun-if-changed=../.env");
    tauri_build::build()
}
