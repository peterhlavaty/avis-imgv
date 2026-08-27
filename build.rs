//! Finds LibRaw for the `libraw` feature.
//!
//! LibRaw is a C++ library with a C API, and it is not vendored: distributions
//! ship it, and building it here would drag in its own dependencies. Three
//! ways of finding it are tried, in the order that gives the clearest answer
//! when it goes wrong.

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=LIBRAW_LIB_DIR");
    println!("cargo::rerun-if-env-changed=LIBRAW_LIB_NAME");
    println!("cargo::rerun-if-env-changed=VCPKG_ROOT");

    #[cfg(feature = "libraw")]
    libraw::link();
}

#[cfg(feature = "libraw")]
mod libraw {
    use std::path::Path;

    pub fn link() {
        if from_env() || with_vcpkg() || with_pkg_config() {
            return;
        }

        panic!(
            "the `libraw` feature is on but LibRaw was not found.\n\
             Install it (libraw-dev, libraw, or `vcpkg install libraw`), or point \
             LIBRAW_LIB_DIR at the directory holding the library."
        );
    }

    /// An explicit path, which is the escape hatch when the rest fails.
    ///
    /// `LIBRAW_LIB_NAME` names the library without its prefix or extension and
    /// defaults to the thread safe build.
    fn from_env() -> bool {
        let Some(directory) = std::env::var_os("LIBRAW_LIB_DIR") else {
            return false;
        };

        let name = std::env::var("LIBRAW_LIB_NAME").unwrap_or_else(|_| "raw_r".to_string());

        println!(
            "cargo::rustc-link-search=native={}",
            Path::new(&directory).display()
        );
        println!("cargo::rustc-link-lib={name}");
        link_cxx_runtime();

        true
    }

    /// vcpkg, which is how the library is usually installed on Windows.
    ///
    /// It resolves the transitive dependencies too, which for a static LibRaw
    /// means jpeg, lcms2, jasper and zlib.
    fn with_vcpkg() -> bool {
        match vcpkg::Config::new().find_package("libraw") {
            Ok(_) => {
                link_cxx_runtime();
                true
            }
            Err(e) => {
                println!("cargo::warning=vcpkg did not provide LibRaw: {e}");
                false
            }
        }
    }

    /// pkg-config, which is how it is installed everywhere else.
    ///
    /// `libraw_r` is the thread safe build; the decoder develops several
    /// images at once, each on its own worker.
    fn with_pkg_config() -> bool {
        ["libraw_r", "libraw"]
            .iter()
            .any(|package| pkg_config::Config::new().probe(package).is_ok())
    }

    /// LibRaw is C++, so its runtime has to come along. MSVC links it itself.
    fn link_cxx_runtime() {
        let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

        match target.as_str() {
            "macos" | "ios" => println!("cargo::rustc-link-lib=c++"),
            "windows" => {}
            _ => println!("cargo::rustc-link-lib=stdc++"),
        }
    }
}
