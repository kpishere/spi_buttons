//use std::process::Command;
//use std::path::Path;
//use std::env;

fn main() {
/*
    // Rerun if PRU source files or main.rs change
    println!("cargo:rerun-if-changed=pru-spi-master.p");
    println!("cargo:rerun-if-changed=pru-spi-common.ph");
    println!("cargo:rerun-if-changed=src/main.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Change to manifest directory for pasm
    env::set_current_dir(&manifest_dir).expect("Failed to change directory");

    // Compile PRU firmware
    let output = Command::new("pasm")
        .args(&["-V2", "-L", "-b", "pru-spi-master.p"])
        .output()
        .expect("Failed to run pasm. Make sure TI PRU compiler tools are installed.");

    if !output.status.success() {
        panic!("pasm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Copy the generated bin file to OUT_DIR for runtime loading
    let out_dir = env::var("OUT_DIR").unwrap();
    let src_bin = Path::new("pru-spi-master.bin");
    let dest_bin = Path::new(&out_dir).join("pru-spi-master.bin");

    if src_bin.exists() {
        std::fs::copy(&src_bin, &dest_bin).expect("Failed to copy PRU bin file");
        println!("cargo:warning=PRU firmware built and copied to {}", dest_bin.display());
    } else {
        panic!("pru-spi-master.bin not found after compilation");
    }
 */
}
