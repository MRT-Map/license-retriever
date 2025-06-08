use license_retriever::{Config, LicenseRetriever};
use test_log::test;

#[test]
fn test() {
    unsafe { std::env::set_var("OUT_DIR", "target"); std::env::set_var("RUST_LOG", "debug"); }
    let config = Config::default();
    let lr = LicenseRetriever::from_config(&config).unwrap();
    assert_eq!(
        lr,
        LicenseRetriever::from_bytes(&lr.to_bytes().unwrap()).unwrap()
    );
    for (p, l) in lr {
        println!("{}: {} ({:?})", p.name, l.len(), p.license);
    }
}
