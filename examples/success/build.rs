use ioc_scan::{
    Scanner,
};

fn main() {
    println!("cargo::rustc-link-arg=-znostart-stop-gc");

    let scanner = Scanner::new("src/main.rs");

    let vec = scanner.types_with_derive("Bean").expect("exty");

    for path in vec {
        std::fs::write("src/test", format!("{path:?}\n")).expect("");
    }
}