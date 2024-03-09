pub mod gdt;
pub mod idt;
pub mod pit;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);


#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct TablePointer {
    pub limit: u16,
    pub base: VirtualAddress,
}