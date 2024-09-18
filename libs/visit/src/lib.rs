use std::path::Path;
use syn::{visit::Visit as SynVisit, ItemImpl, ItemStruct};

mod scan;
mod ext;

use scan::Scanner;

pub type ItemStructExt<'a> = ext::ItemExt<'a, ItemStruct>;
pub type ItemImplExt<'a> = ext::ItemExt<'a, ItemImpl>;

pub trait Visit {
    fn item_struct(&mut self, _: &ItemStructExt<'_>) {}

    fn item_impl(&mut self, _: &ItemImplExt<'_>) {}
}

pub fn scan<T: Visit>(visit: &mut T, file: &Path) {
    let mut scanner = Scanner::root(file)
        .with(visit);

    let result = std::fs::read_to_string(&file)
        .expect(&format!("Unable to read {}", file.display()));
    let file = syn::parse_file(&result)
        .expect(&format!("Unable to parse {}", file.display()));

    scanner.visit_file(&file)
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
