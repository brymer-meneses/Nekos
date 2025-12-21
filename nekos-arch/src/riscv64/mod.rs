mod trap;

pub use trap::*;

pub trait CsrWrite {
    unsafe fn write(value: Self);
}
pub trait CsrRead {
    fn read() -> Self;
}

macro_rules! impl_csr_read {
    ($name:ident, $register:literal) => {
        impl CsrRead for $name {
            fn read() -> Self {
                let mut value;
                unsafe {
                    core::arch::asm!(concat!("csrr {}, ", $register), out(reg) value);
                }
                $name(value)
            }
        }
    };
}

macro_rules! impl_csr_write {
    ($name:ident, $register:literal) => {
        impl CsrWrite for $name {
            unsafe fn write(value: Self) {
                unsafe {
                    core::arch::asm!(concat!("csrw ", $register, ", {}"), in(reg) value.0);
                }
            }
        }
    };
}

macro_rules! impl_csr {
    ($name:ident, $register:literal) => {
        impl_csr_write!($name, $register);
        impl_csr_read!($name, $register);
    };
}

pub struct Scause(u64);
impl_csr!(Scause, "scause");

pub struct Stval(u64);
impl_csr!(Stval, "stval");

pub struct Sepc(u64);
impl_csr!(Sepc, "sepc");

pub struct Sstatus(u64);
impl_csr!(Sstatus, "sstatus");

pub struct Stvec(pub u64);
impl_csr!(Stvec, "stvec");

impl Scause {
    pub const fn new(value: u64) -> Scause {
        Scause(value)
    }

    pub const fn interrupt_code(&self) -> InterruptCode {
        let code = self.code();
        match code {
            1 => InterruptCode::SupervisorSoftwareInterrupt,
            2 => InterruptCode::VirtualSupervisorSoftwareInterrupt,
            3 => InterruptCode::MachineSoftwareInterrupt,

            5 => InterruptCode::SupervisorTimerInterrupt,
            6 => InterruptCode::VirtualSupervisorTimerInterrupt,
            7 => InterruptCode::MachineTimerInterrupt,

            9 => InterruptCode::SupervisorExternalInterrupt,
            10 => InterruptCode::VirtualSupervisorExternalInterrupt,
            11 => InterruptCode::MachineExternalInterrupt,
            12 => InterruptCode::SupervisorGuestExternalInterrupt,

            _ => InterruptCode::Unknown,
        }
    }

    pub const fn exception_code(&self) -> ExceptionCode {
        let code = self.code();
        match code {
            0 => ExceptionCode::InstructionAddressMisaligned,
            1 => ExceptionCode::InstructionAccessFault,
            2 => ExceptionCode::IllegalInstruction,
            3 => ExceptionCode::Breakpoint,
            4 => ExceptionCode::LoadAddressMisaligned,
            5 => ExceptionCode::LoadAccessFault,
            6 => ExceptionCode::StoreAmoAddressMisaligned,
            7 => ExceptionCode::StoreAmoAccessFault,
            8 => ExceptionCode::EnvironmentCallFromUserMode,
            9 => ExceptionCode::EnvironmentCallFromHypervisorMode,
            10 => ExceptionCode::EnvironmentCallFromVirtualSupervisorMode,
            11 => ExceptionCode::EnvironmentCallFromMachineMode,
            12 => ExceptionCode::InstructionPageFault,
            13 => ExceptionCode::LoadPageFault,
            15 => ExceptionCode::StoreAmoPageFault,
            16 => ExceptionCode::DoubleTrap,
            18 => ExceptionCode::SoftwareCheck,
            19 => ExceptionCode::HardwareError,
            20 => ExceptionCode::InstructionGuestPageFault,
            21 => ExceptionCode::LoadGuestPageFault,
            22 => ExceptionCode::VirtualInstruction,
            23 => ExceptionCode::StoreAmoGuestPageFault,
            _ => ExceptionCode::Unknown,
        }
    }

    pub const fn is_interrupt(&self) -> bool {
        return (self.0 >> 63) == 1;
    }

    const fn code(&self) -> u64 {
        return self.0 & !(1 << 63);
    }
}
