pub trait Bootstrap {
    fn run(self);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Print;

    impl Bootstrap for Print {
        fn run(self) {
            println!("Print");
        }
    }

    #[test]
    fn it_works() {
        Print.run();
    }
}
