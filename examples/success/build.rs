use std::fs::File;
use std::io::Write;
use ioc_scan::{
    Scanner,
};
use ioc_scan::syn::__private::quote::quote;

fn main() {
    println!("cargo::rustc-link-arg=-znostart-stop-gc");

    let scanner = Scanner::new("src/main.rs");

    let vec = scanner.types_with_derive("Bean").expect("exty");


    let mut file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("test.txt")
        .expect("");
    for path in vec {
        let code = quote! {#path}.to_string();
        file.write(format!("{code}\n").as_bytes()).expect("");
    }
}