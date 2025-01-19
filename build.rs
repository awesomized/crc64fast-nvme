// build.rs

fn main() {
    #[cfg(feature = "c-bindings")]
    {
        extern crate cbindgen;
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        cbindgen::generate(crate_dir)
            .expect("Unable to generate C bindings.")
            .write_to_file("crc64fast_nvme.h");
    }
}
