use std::{env, path::PathBuf, process::Command};

fn main() {
    // Download mGBA source.
    let mgba_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("mgba");
    if !mgba_path.join("CMakeLists.txt").exists() {
        Command::new("git")
            .args(&["submodule", "update", "--init"])
            .current_dir(mgba_path.clone())
            .status()
            .expect("unable to use git to fetch mgba source");
    }

    // Build mGBA.
    Command::new("rm")
        .args(&["-rf", "build"])
        .current_dir(mgba_path.clone())
        .status()
        .expect("could not remove old build directory");
    Command::new("mkdir")
        .args(&["build"])
        .current_dir(mgba_path.clone())
        .status()
        .expect("could not make mgba build directory");
    let build_path = mgba_path.join("build");
    dbg!(&build_path);
    Command::new("cmake")
        .args(&[
            "-S ..",
            "-DBUILD_STATIC=ON",
            "-DBUILD_SHARED=OFF",
            "-DDISABLE_FRONTENDS=ON",
            "-DBUILD_GL=OFF",
            "-DBUILD_GLES2=OFF",
            "-DBUILD_GLES3=OFF",
            "-DUSE_GDB_STUB=OFF",
            "-DUSE_FFMPEG=OFF",
            "-DUSE_ZLIB=OFF",
            "-DUSE_MINIZIP=OFF",
            "-DUSE_PNG=OFF",
            "-DUSE_LIBZIP=OFF",
            "-DUSE_SQLITE3=OFF",
            "-DUSE_ELF=ON",
            "-DM_CORE_GBA=ON",
            "-DM_CORE_GB=OFF",
            "-DUSE_LZMA=OFF",
            "-DUSE_DISCORD_RPC=OFF",
            "-DENABLE_SCRIPTING=OFF",
            "-DCMAKE_BUILD_TYPE=Debug",
            "-DUSE_EPOXY=OFF",
        ])
        .current_dir(build_path.clone())
        .status()
        .expect("could not run cmake");
    Command::new("make")
        .current_dir(build_path.clone())
        .status()
        .expect("could not run make");

    // Build C++ binary to interact with mgba.
    cc::Build::new()
        .file("c/runner.c")
        .include(&mgba_path.join("include"))
        .static_flag(true)
        .debug(true)
        .compile("runner");

    // Link it all together.
    println!(
        "cargo:rustc-link-search={}",
        PathBuf::from(env::var("OUT_DIR").unwrap()).display()
    );
    println!(
        "cargo:rustc-link-search={}",
        mgba_path.join("build").display()
    );
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=/System/Library/Frameworks");
    println!("cargo:rustc-link-lib=static=mgba");
    println!("cargo:rustc-link-lib=elf");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=runner");
    println!("cargo:rerun-if-changed=c/runner.c");
    println!("cargo:rerun-if-changed=c/runner.h");

    // Generate bindings.
    let bindings = bindgen::Builder::default()
        .header("c/runner.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("unable to obtain output path"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write mgba bindings");
}
