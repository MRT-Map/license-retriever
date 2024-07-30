use license_retriever::{config::Config, LicenseRetriever};
use test_log::test;

#[test]
fn test() {
    let config = Config::default();
    let lr = LicenseRetriever::from_config(&config).unwrap();
    assert_eq!(
        lr,
        LicenseRetriever::from_bytes(&lr.to_bytes().unwrap()).unwrap()
    );
    for (p, l) in lr {
        println!("{}: {}", p.name, l.len());
    }
}
