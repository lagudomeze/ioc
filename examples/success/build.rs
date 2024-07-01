use ioc_scan::build_init_method;

fn main() {
    println!("cargo::rustc-link-arg=-znostart-stop-gc");
    println!("cargo::rerun-if-changed=src/**/*.rs");
    build_init_method();
}