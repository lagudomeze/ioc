#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct BeanId {
    id: usize,
}


impl BeanId {
    pub fn new(value: usize) -> Self {
        Self { id: value }
    }
}