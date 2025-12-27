#[derive(Debug)]
pub enum InterruptCode {
    SupervisorSoftwareInterrupt,
    VirtualSupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,

    SupervisorTimerInterrupt,
    VirtualSupervisorTimerInterrupt,
    MachineTimerInterrupt,

    SupervisorExternalInterrupt,
    VirtualSupervisorExternalInterrupt,
    MachineExternalInterrupt,
    SupervisorGuestExternalInterrupt,
    LocalCounterOverflowInterrupt,
}

#[derive(Debug)]
pub enum ExceptionCode {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    InstructionPageFault,
    InstructionGuestPageFault,

    LoadPageFault,

    IllegalInstruction,
    Breakpoint,

    LoadAddressMisaligned,
    LoadAccessFault,
    LoadGuestPageFault,

    StoreAmoAddressMisaligned,
    StoreAmoAccessFault,
    StoreAmoPageFault,
    StoreAmoGuestPageFault,

    EnvironmentCallFromUserMode,
    EnvironmentCallFromHypervisorMode,
    EnvironmentCallFromVirtualSupervisorMode,
    EnvironmentCallFromMachineMode,

    DoubleTrap,
    SoftwareCheck,
    HardwareError,
    VirtualInstruction,
}

#[repr(C, packed)]
pub struct TrapFrame {
    pub ra: u64,
    pub gp: u64,
    pub tp: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub sp: u64,
}
