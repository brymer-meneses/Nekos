use crate::arch::{PAGE_SIZE, PageDirectory, PageMapErr};
use crate::mem::{PhysicalAddr, VirtualAddr, VirtualMemoryFlags};

use crate::boot;
use core::usize;
use limine::paging::Mode;

pub fn map_page<T: PageDirectory>(
    directory: &T,
    virtual_addr: VirtualAddr,
    physical_addr: PhysicalAddr,
    flags: VirtualMemoryFlags,
) -> Result<(), PageMapErr> {
    debug_assert!(virtual_addr.is_aligned_with(PAGE_SIZE));
    debug_assert!(physical_addr.is_aligned_with(PAGE_SIZE));

    let pte_flags = PageTableFlags::new(flags);
    let mode = *boot::PAGING_MODE.get().expect("Boot not initialized yet.");
    let virtual_page_numbers = VPN::parse(mode, virtual_addr);

    let mut page_table = PageTable::from_addr(directory.root_page_table_addr());

    if let Some(vpn4) = virtual_page_numbers.vpn4 {
        page_table = get_next_level(directory, page_table, vpn4, true)?;
    }

    if let Some(vpn3) = virtual_page_numbers.vpn3 {
        page_table = get_next_level(directory, page_table, vpn3, true)?;
    }

    page_table = get_next_level(directory, page_table, virtual_page_numbers.vpn2, true)?;
    page_table = get_next_level(directory, page_table, virtual_page_numbers.vpn1, true)?;

    let ppn = PPN::from_physical_addr(physical_addr);

    page_table.entries[virtual_page_numbers.vpn0 as usize] = PageTableEntry::new(ppn, pte_flags);

    Ok(())
}

fn get_next_level<'a, T: PageDirectory>(
    directory: &T,
    page_table: &'a mut PageTable,
    vpn: u16,
    allocate: bool,
) -> Result<&'a mut PageTable, PageMapErr> {
    let pte = &mut page_table.entries[vpn as usize];
    let flags = pte.flags();

    if !flags.contains(PageTableFlags::VALID) && !allocate {
        return Err(PageMapErr);
    }

    if !flags.contains(PageTableFlags::VALID) {
        let new_page = crate::mem::allocate_pages(1).map_err(|_| PageMapErr)?;
        let virtual_addr = directory.translate(new_page);

        unsafe { core::ptr::write_bytes(virtual_addr.as_mut_ptr(), 0, PAGE_SIZE as usize) };

        let ppn = PPN::from_physical_addr(new_page);

        *pte = PageTableEntry::new(ppn, PageTableFlags::VALID)
    }

    Ok(PageTable::from_addr(
        directory.translate(pte.ppn().as_physical_addr()),
    ))
}

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PageTableFlags: u64 {
        const VALID      = 1 << 0;
        const READABLE   = 1 << 1;
        const WRITABLE   = 1 << 2;
        const EXECUTABLE = 1 << 3;
        const USER       = 1 << 4;
        const GLOBAL     = 1 << 5;
        const ACCESSED   = 1 << 6;
        const DIRTY      = 1 << 7;
    }
}

impl PageTableFlags {
    pub fn new(virtual_memory_flags: VirtualMemoryFlags) -> Self {
        let mut flags = PageTableFlags::READABLE | PageTableFlags::VALID;

        if virtual_memory_flags.contains(VirtualMemoryFlags::Writeable) {
            flags |= PageTableFlags::WRITABLE;
        }

        if virtual_memory_flags.contains(VirtualMemoryFlags::Executable) {
            flags |= PageTableFlags::EXECUTABLE;
        }

        if virtual_memory_flags.contains(VirtualMemoryFlags::UserAccessible) {
            flags |= PageTableFlags::USER;
        }

        flags
    }
}

#[repr(align(4096))]
struct PageTable {
    entries: [PageTableEntry; 512],
}

impl<'a> PageTable {
    pub fn from_addr(virtual_addr: VirtualAddr) -> &'a mut Self {
        let ptr = virtual_addr.as_ptr() as *mut PageTable;
        debug_assert!(ptr.is_aligned(), "{}", virtual_addr);

        unsafe { ptr.as_mut().expect("Null ptr") }
    }
}

#[derive(Clone, Copy, Debug)]
struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn new(ppn: PPN, flags: PageTableFlags) -> Self {
        let entry = (ppn.value() << 10) | flags.bits();
        PageTableEntry(entry)
    }

    pub const fn ppn(&self) -> PPN {
        PPN::new((self.0 >> 10) & 0x0FFF_FFFF_FFFF)
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.0 = (self.0 & !0xFF) | flags.bits();
    }

    pub fn has_flag(&self, flag: PageTableFlags) -> bool {
        self.flags().contains(flag)
    }
}

struct VPN {
    vpn0: u16,
    vpn1: u16,
    vpn2: u16,
    vpn3: Option<u16>,
    vpn4: Option<u16>,
}

impl VPN {
    const MASK: u64 = 0x1FF;

    const fn parse(mode: Mode, virtual_addr: VirtualAddr) -> Self {
        let mut indices = Self {
            vpn0: Self::extract(virtual_addr, 12),
            vpn1: Self::extract(virtual_addr, 21),
            vpn2: Self::extract(virtual_addr, 30),
            vpn3: None,
            vpn4: None,
        };

        match mode {
            Mode::SV39 => {}
            Mode::SV48 => indices.vpn3 = Some(Self::extract(virtual_addr, 39)),
            Mode::SV57 => {
                indices.vpn3 = Some(Self::extract(virtual_addr, 39));
                indices.vpn4 = Some(Self::extract(virtual_addr, 48));
            }
            _ => unreachable!(),
        }

        indices
    }

    #[inline]
    const fn extract(virtual_addr: VirtualAddr, shift: u64) -> u16 {
        let addr = virtual_addr.addr();
        ((addr >> shift) & Self::MASK) as u16
    }
}

pub struct PPN(u64);

impl PPN {
    pub const fn new(value: u64) -> Self {
        PPN(value)
    }

    pub const fn from_physical_addr(addr: PhysicalAddr) -> Self {
        PPN::new(addr.addr() >> 12)
    }

    pub const fn as_physical_addr(&self) -> PhysicalAddr {
        PhysicalAddr::new(self.0 << 12)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}
