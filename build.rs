extern crate bindgen;

use autotools::Config;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // build wolfSSL

    let wolfssl_src = out_dir.join(PathBuf::from("wolfssl"));

    Command::new("cp")
        .arg("-r")
        .arg("wolfssl-src")
        .arg(wolfssl_src.clone())
        .output()
        .expect("Unable to copy wolfssl");
    
    let mut conf = Config::new(wolfssl_src);

    conf.reconf("-ivf")
        .disable("examples", None)
        .disable("filesystem", None)
        .enable("singlethreaded", None)
        .disable_shared()
        .enable_static()
        .cflag("-fPIC");
    
    let wolfssl_dst = conf.build();

    println!("cargo:rustc-link-search=native={}", wolfssl_dst.display());
    println!("cargo:rustc-link-lib=static=wolfssl");

    // build wolfTPM

    let wolftpm_src = out_dir.join(PathBuf::from("wolftpm"));

    Command::new("cp")
        .arg("-r")
        .arg("wolftpm-src")
        .arg(wolftpm_src.clone())
        .output()
        .expect("Unable to copy wolftpm");

    let mut conf = Config::new(wolftpm_src);

    conf.reconf("-ivf")
        .disable("examples", None)
        .disable("firmware", None)
        .enable("advio", None)
        .enable("mmio", None)
        .disable_shared()
        .enable_static()
        .cflag("-DNO_THREAD_LS")
        .cflag("-fno-stack-protector")
        .cflag("-fPIC");
    
    let wolftpm_dst = conf.build();

    println!("cargo:rustc-link-search=native={}", wolftpm_dst.join("lib").display());
    println!("cargo:rustc-link-lib=static=wolftpm");

    // Write bindings

    let builder = bindgen::Builder::default()
        .allowlist_file(wolftpm_dst.join(PathBuf::from("include/wolftpm/tpm2_wrap.h")).to_str().unwrap())
        .clang_arg(format!("-I{}/", wolftpm_dst.join("include").display()))
        .header("wrapper.h")
        .use_core()
        .derive_default(true);

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings.");

    bindings
        .write_to_file(PathBuf::from("src/bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
