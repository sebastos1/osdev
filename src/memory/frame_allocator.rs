use super::PAGE_SIZE;

#[derive(Debug, Clone, Copy)]
struct Frame {
    number: usize,
}

impl Frame {
    fn containing_address(address: usize) -> Self {
        Frame { number: address / PAGE_SIZE }
    }
}