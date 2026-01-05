use crate::arch::{PAGE_SIZE, PageMapErr};
use crate::mem::{PhysicalAddr, VirtualAddr, VirtualMemoryFlags};

use crate::{boot, log};
use core::usize;
use limine::paging::Mode;

pub fn map_page(
    root_page_table_addr: PhysicalAddr,
    virtual_addr: VirtualAddr,
    physical_addr: PhysicalAddr,
    size: usize,
    flags: VirtualMemoryFlags,
) -> Result<(), PageMapErr> {
    if !virtual_addr.is_aligned_with(PAGE_SIZE) {
        return Err(PageMapErr::UnalignedVirtualAddr);
    }

    if !physical_addr.is_aligned_with(PAGE_SIZE) {
        return Err(PageMapErr::UnalignedSize);
    }

    let mut va = virtual_addr;
    let mut pa = physical_addr;
    let mut remaining = size as u64;

    let one_gib = 1.gibibytes();
    let two_mib = 2.mebibytes();

    while remaining != 0 {
        let page_type = if remaining >= one_gib
            && va.is_aligned_with(one_gib.into())
            && pa.is_aligned_with(one_gib.into())
        {
            PageType::OneGiB
        } else if remaining >= two_mib
            && va.is_aligned_with(two_mib.into())
            && pa.is_aligned_with(two_mib.into())
        {
            PageType::TwoMiB
        } else {
            PageType::FourKiB
        };

        map_page_impl(root_page_table_addr, va, pa, page_type, flags)?;

        let step: u64 = page_type.in_bytes().into();

        va = VirtualAddr::new(va.addr() + step);
        pa = PhysicalAddr::new(pa.addr() + step);
        remaining -= step;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum PageType {
    OneGiB,
    TwoMiB,
    FourKiB,
}

impl PageType {
    fn in_bytes(&self) -> ubyte::ByteUnit {
        match self {
            PageType::FourKiB => 4.kibibytes(),
            PageType::TwoMiB => 2.mebibytes(),
            PageType::OneGiB => 1.gibibytes(),
        }
    }
}

fn map_page_impl(
    root_page_table_addr: PhysicalAddr,
    virtual_addr: VirtualAddr,
    physical_addr: PhysicalAddr,
    page_type: PageType,
    flags: VirtualMemoryFlags,
) -> Result<(), PageMapErr> {
    let alignment = page_type.in_bytes().into();

    debug_assert!(virtual_addr.is_aligned_with(alignment));
    debug_assert!(physical_addr.is_aligned_with(alignment));

    let mut indices = [0u16; 5];
    for (i, shift) in [12u16, 21, 30, 39, 48].iter().enumerate() {
        let addr = virtual_addr.addr();
        indices[i] = ((addr >> shift) & 0x1FF) as u16;
    }

    let boot_info = boot::BOOT_INFO.get().unwrap();

    let top_level = match boot_info.paging_mode {
        Mode::SV57 => 4,
        Mode::SV48 => 3,
        Mode::SV39 => 2,
        _ => unreachable!(),
    };

    let leaf_level = match page_type {
        PageType::FourKiB => 0,
        PageType::TwoMiB => 1,
        PageType::OneGiB => 2,
    };

    let root_page_table = root_page_table_addr.as_virtual_by_offset(boot_info.hhdm_offset);
    let mut page_table = PageTable::from_addr(root_page_table);

    for level in (leaf_level + 1..=top_level).rev() {
        let vpn = indices[level] as usize;
        let pte = &mut page_table.entries[vpn];

        if !pte.flags().contains(PageTableFlags::VALID) {
            let new_page = crate::mem::allocate_pages(1, /*zeroed=*/ true)
                .map_err(|_| PageMapErr::PageFrameAllocError)?;
            let ppn = PPN::from_physical_addr(new_page);
            *pte = PageTableEntry::new(ppn, PageTableFlags::VALID);
        }

        let next_table_vaddr = pte
            .ppn()
            .as_physical_addr()
            .as_virtual_by_offset(boot_info.hhdm_offset);
        page_table = PageTable::from_addr(next_table_vaddr);
    }

    let vpn = indices[leaf_level] as usize;
    let flags = PageTableFlags::new(flags);
    let ppn = PPN::from_physical_addr(physical_addr);

    page_table.entries[vpn] = PageTableEntry::new(ppn, flags);

    Ok(())
}

use bitflags::bitflags;
use ubyte::ToByteUnit;

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
        let ptr = virtual_addr.as_mut_ptr::<PageTable>();
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
