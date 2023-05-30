use license_retriever::{Config, LicenseRetriever};
use test_log::test;

#[test]
fn test() {
    let config = Config::default()
        .panic_if_no_license_found()
        .copy_license("lazy-regex-proc_macros", "lazy-regex")
        .override_license_url(
            "gloo-timers",
            [
                "https://raw.githubusercontent.com/rustwasm/gloo/master/LICENSE-MIT",
                "https://raw.githubusercontent.com/rustwasm/gloo/master/LICENSE-APACHE",
            ],
        )
        .copy_license("stdweb-derive", "stdweb")
        .copy_license("stdweb-internal-macros", "stdweb")
        .copy_license("stdweb-internal-runtime", "stdweb")
        .copy_license("stdweb-derive", "stdweb")
        .copy_license("winapi-i686-pc-windows-gnu", "winapi")
        .copy_license("winapi-x86_64-pc-windows-gnu", "winapi")
        .ignore("license-retriever");
    let lr = LicenseRetriever::from_config(&config).unwrap();
    assert_eq!(
        lr,
        LicenseRetriever::from_bytes(&lr.to_bytes().unwrap()).unwrap()
    );
    for (p, l) in lr {
        println!("{}: {}", p.name, l.len())
    }
}
