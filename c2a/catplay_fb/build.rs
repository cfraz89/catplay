fn main() {
    #[cfg(feature = "libyuv")]
    println!("cargo:rustc-link-lib=yuv");
}
