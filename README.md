# license-retriever

![Crates.io Version](https://img.shields.io/crates/v/license-retriever)
![Github Version](https://img.shields.io/github/v/release/MRT-Map/license-retriever)
![Crates.io MSRV](https://img.shields.io/crates/msrv/license-retriever)
![docs.rs](https://img.shields.io/docsrs/license-retriever)
![GitHub License](https://img.shields.io/github/license/MRT-Map/license-retriever)

![GitHub code size](https://img.shields.io/github/languages/code-size/MRT-Map/license-retriever)
![GitHub repo size](https://img.shields.io/github/repo-size/MRT-Map/license-retriever)
![GitHub last commit (branch)](https://img.shields.io/github/last-commit/mrt-map/license-retriever/dev)
![GitHub commits since latest release (branch)](https://img.shields.io/github/commits-since/mrt-map/license-retriever/latest/dev?include_prereleases)
![GitHub Release Date](https://img.shields.io/github/release-date/MRT-Map/license-retriever)
![Libraries.io dependency status for GitHub repo](https://img.shields.io/librariesio/github/MRT-Map/license-retriever)

![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/license-retriever)
![Crates.io Total Downloads](https://img.shields.io/crates/d/license-retriever)

Retrieves licenses of all Rust dependencies. Originally written for [stencil2](https://github.com/MRT-Map/stencil2) but is now separated into its own project.

## How
Licenses are retrieved by searching in the following order:
* Folder that `Cargo.toml` is in
* Crate cache in `~/.cargo`
* GitHub repository
* Text from `spdx` with identifier in `Cargo.toml`

## Usage
```rust
    // build.rs ==========
    fn main() {
        let config = license_retriever::Config {
            // options...
            ..Config::default()
        };
        license_retriever::LicenseRetriever::from_config(&config)?.save_in_out_dir("licenses")?;
    }

    // main project ==========
    fn main() {
        let licenses = license_retriever::license_retriever_data!("licenses").unwrap(); // Vec<(Package, Vec<String>)>
    }
```