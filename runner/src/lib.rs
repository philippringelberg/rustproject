use ktest::{read_ktest, KTEST};

pub mod common;

use probe_rs::{
    config::registry::{Registry, SelectionStrategy},
    coresight::memory::MI,
    flash::download::{
        download_file, download_file_with_progress_reporting, FileDownloadError, Format,
    },
    probe::{stlink, DebugProbe, DebugProbeError, DebugProbeType, MasterProbe, WireProtocol},
    session::Session,
    target::info::{self, ChipInfo},
};

/// Returns first found stlink probe as MasterProbe
pub fn open_probe() -> MasterProbe {
    let devs = stlink::tools::list_stlink_devices();
    // just pick the first one
    let device = devs.get(0).unwrap();
    println!("device {:?}", device);
    let mut link = stlink::STLink::new_from_probe_info(&device).unwrap();

    link.attach(Some(WireProtocol::Swd)).unwrap();

    MasterProbe::from_specific_probe(link)
}

/// Returns a Session from first found stlink probe
pub fn open_session() -> Session {
    let mut probe = open_probe();
    println!("probe connected");

    let strategy = SelectionStrategy::ChipInfo(ChipInfo::read_from_rom_table(&mut probe).unwrap());
    println!("strategy {:?}", strategy);

    let strategy = SelectionStrategy::TargetIdentifier("stm32f411".into());

    let registry = Registry::from_builtin_families();

    let target = registry.get_target(strategy).unwrap();
    println!("target {:?}", target);

    Session::new(target, probe)
}

/// resets the target and run
pub fn reset_and_run(session: &mut Session) {
    session.target.core.reset(&mut session.probe).unwrap();
}

/// resets the target and halts
pub fn reset_and_halt(session: &mut Session) {
    session
        .target
        .core
        .reset_and_halt(&mut session.probe)
        .unwrap();
}

/// read current instruction and returns
/// Some(n)     where n is the breakpoint numbr
/// None        if its not a breakpoint instruction
pub fn read_bkpt(session: &mut Session) -> Option<u8> {
    // try to read the program counter
    let pc_value = session
        .target
        .core
        .read_core_reg(&mut session.probe, session.target.core.registers().PC)
        .unwrap();

    let mut instr16 = [0u8; 2];
    session.probe.read_block8(pc_value, &mut instr16).unwrap();

    match instr16[1] {
        0b10111110 => Some(instr16[0]), // 0b10111110 is the binary repr of `bkpt #n`
        _ => None,
    }
}

/// increments the pc by 2 (useful to step away from breakpoint)
pub fn increment_pc(session: &mut Session) {
    // try to read the program counter
    let pc_value = session
        .target
        .core
        .read_core_reg(&mut session.probe, session.target.core.registers().PC)
        .unwrap();

    // the bkpt() is a 16 bit instruction, increment pc by 16 bits (i.e. 2 bytes)
    let new_pc_value = pc_value + 0x2;
    session
        .target
        .core
        .write_core_reg(
            &mut session.probe,
            session.target.core.registers().PC,
            new_pc_value,
        )
        .unwrap();
}

/// continue execution until target halted or Timeout reached
pub fn run_to_halt(session: &mut Session) {
    // check if contineing from breakpoint
    if read_bkpt(session).is_some() {
        println!("Continue from breakpoint.");
        increment_pc(session);
    } else {
        println!("Continue");
    }
    session.target.core.run(&mut session.probe).unwrap();
    println!("running");
    match session.target.core.wait_for_core_halted(&mut session.probe) {
        Ok(_) => {
            print!("Hit breakpoint :",);
        }
        Err(DebugProbeError::Timeout) => {
            print!("Timeout :");
        }
        Err(err) => panic!("internal error:{:?}", err),
    }

    let cpu_info = session.target.core.halt(&mut session.probe).unwrap();
    println!("Core stopped at address 0x{:08x}", cpu_info.pc);
}

/// set synbolic values at address of R0
fn set_symbolic(session: &mut Session, data: &[u8]) {
    let r0 = session
        .target
        .core
        .read_core_reg(&mut session.probe, 0.into())
        .unwrap();

    println!("r0 0x{:08x}", r0);
    println!("object {:?}", data);
    // session.target.core.step(&mut session.probe).unwrap();
    let r0 = session.probe.write_block8(r0, data).unwrap();
}

/// DWT_CTRL control register
const DWT_CTRL: u32 = 0xe000_1000;
/// DWT_CTRL cycle counter register
const DWT_CYCCNT: u32 = 0xe000_1004;

/// enable the cycle counter
pub fn cycnt_enable(session: &mut Session) {
    session.probe.write32(DWT_CTRL, 0x1).unwrap();
}

/// stop the cycle counter
pub fn cycnt_disable(session: &mut Session) {
    session.probe.write32(DWT_CTRL, 0x0).unwrap();
}

/// reset the cyclecounter to 0
pub fn cycnt_reset(session: &mut Session) {
    // Reset cycle counter to 0
    session.probe.write32(DWT_CYCCNT, 0x0).unwrap();
}

/// read cycle counter into u32
pub fn cycnt_read(session: &mut Session) -> u32 {
    session.probe.read32(DWT_CYCCNT).unwrap()
}
