use clap::CommandFactory;
use std::fs;
use std::path::PathBuf;

include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    // Set build-time environment variables for version info
    println!("cargo:rustc-env=BUILD_DATE={}", chrono::Utc::now().format("%Y-%m-%d"));
    println!("cargo:rustc-env=BUILD_TARGET={}", std::env::var("TARGET").unwrap());

    // Generate man page using clap_mangen
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let man_dir = out_dir.join("../../../man");

    fs::create_dir_all(&man_dir)?;

    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    fs::write(man_dir.join("memwatch.1"), buffer)?;

    println!("cargo:warning=Man page generated at {:?}", man_dir.join("memwatch.1"));

    Ok(())
}
