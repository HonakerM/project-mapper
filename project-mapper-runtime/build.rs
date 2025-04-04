fn generate_gl_bindings() {
    let dest = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap());
    let mut file = std::fs::File::create(dest.join("gl_bindings.rs")).unwrap();
    gl_generator::Registry::new(
        gl_generator::Api::Gles2,
        (3, 0),
        gl_generator::Profile::Core,
        gl_generator::Fallbacks::All,
        [],
    )
    .write_bindings(gl_generator::StructGenerator, &mut file)
    .unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    generate_gl_bindings();

    // https://github.com/rust-lang/cargo/issues/5077#issuecomment-1284482987
    #[cfg(all(not(docsrs), target_os = "macos"))]
    match system_deps::Config::new().probe() {
        Ok(deps) => {
            let usr = std::path::Path::new("/usr/lib");
            let usr_local = std::path::Path::new("/usr/local/lib");
            for dep in deps.all_link_paths() {
                if dep != &usr && dep != &usr_local {
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{:?}", dep.as_os_str());
                }
            }
        }
        Err(s) => {
            println!("cargo:warning={s}");
            std::process::exit(1);
        }
    }
}
