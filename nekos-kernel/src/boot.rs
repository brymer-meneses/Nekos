use crate::arch::PAGE_SIZE;
use crate::log;
use crate::mem::{PhysicalAddr, VirtualAddr};

use spin::Once;
use ubyte::ToByteUnit;

use limine::BaseRevision;
use limine::memory_map::{Entry, EntryType};
use limine::paging::Mode;
use limine::request::{ExecutableAddressRequest, HhdmRequest, MemoryMapRequest, PagingModeRequest};

#[unsafe(link_section = ".requests")]
static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new();

#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(link_section = ".requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

pub static BOOT_INFO: Once<BootInfo> = Once::new();

pub fn init() {
    assert!(
        BASE_REVISION.is_supported(),
        "Limine base revision is not supported"
    );

    BOOT_INFO.call_once(|| BootInfo {
        hhdm_offset: HHDM_REQUEST.get_response().unwrap().offset(),
        paging_mode: PAGING_MODE_REQUEST.get_response().unwrap().mode(),
        memory_map_entries: MEMORY_MAP_REQUEST.get_response().unwrap().entries(),
        kernel_address: PhysicalAddr::new(
            EXECUTABLE_ADDRESS_REQUEST
                .get_response()
                .unwrap()
                .physical_base(),
        ),
    });

    let boot_info = unsafe { BOOT_INFO.get_unchecked() };

    log::debug!("HHDM_OFFSET at {}", VirtualAddr::new(boot_info.hhdm_offset));

    let paging_mode = match boot_info.paging_mode {
        Mode::SV39 => "SV39",
        Mode::SV48 => "SV48",
        Mode::SV57 => "SV57",
        _ => unreachable!("Invalid paging mode"),
    };

    log::debug!("Paging Mode at {}", paging_mode);

    let entry_type_to_str = |entry_type: EntryType| match entry_type {
        EntryType::USABLE => "Usable",
        EntryType::RESERVED => "Reserved",
        EntryType::ACPI_RECLAIMABLE => "ACPI Reclaimable",
        EntryType::ACPI_NVS => "ACPI NVS",
        EntryType::BAD_MEMORY => "Bad Memory",
        EntryType::BOOTLOADER_RECLAIMABLE => "Bootloader Reclaimable",
        EntryType::EXECUTABLE_AND_MODULES => "Kernel and Modules",
        EntryType::FRAMEBUFFER => "Framebuffer",
        _ => unreachable!(),
    };

    for entry in boot_info.memory_map_entries.iter() {
        let entry_type = entry_type_to_str(entry.entry_type);

        log::debug!(
            "`{}' Memory at {}-{} {}",
            entry_type,
            PhysicalAddr::new(entry.base),
            PhysicalAddr::new(entry.base + entry.length),
            entry.length.bytes()
        );
    }
}

pub struct BootInfo<'a> {
    pub hhdm_offset: u64,
    pub paging_mode: Mode,
    pub memory_map_entries: &'a [&'a Entry],
    pub kernel_address: PhysicalAddr,
}

unsafe extern "C" {
    #[link_name = "__kernel_blob_begin"]
    pub static KERNEL_BLOB_BEGIN: u8;

    #[link_name = "__kernel_blob_end"]
    pub static KERNEL_BLOB_END: u8;

    #[link_name = "__kernel_rodata_begin"]
    pub static KERNEL_RODATA_BEGIN: u8;

    #[link_name = "__kernel_rodata_end"]
    pub static KERNEL_RODATA_END: u8;

    #[link_name = "__kernel_data_begin"]
    pub static KERNEL_DATA_BEGIN: u8;

    #[link_name = "__kernel_data_end"]
    pub static KERNEL_DATA_END: u8;

    #[link_name = "__kernel_code_begin"]
    pub static KERNEL_CODE_BEGIN: u8;

    #[link_name = "__kernel_code_end"]
    pub static KERNEL_CODE_END: u8;
}
