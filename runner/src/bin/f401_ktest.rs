use failure::format_err;
use ktest::{read_ktest, KTEST};

use probe_rs::{
    config::registry::{Registry, SelectionStrategy},
    coresight::access_ports::AccessPortError,
    coresight::memory::MI,
    flash::download::{
        download_file, download_file_with_progress_reporting, FileDownloadError, Format,
    },
    probe::{stlink, DebugProbe, DebugProbeError, DebugProbeType, MasterProbe, WireProtocol},
    session::Session,
    target::info::{self, ChipInfo},
};

// don't look at this, its just testing some stuff...

// le byte order
fn main() {
    println!("read ktest file");

    let ktest = read_ktest("resources/test000001.ktest").unwrap();
    println!("ktest {:?}", ktest);

    let mut probe = open_probe();
    println!("probe connected");

    let strategy = SelectionStrategy::ChipInfo(ChipInfo::read_from_rom_table(&mut probe).unwrap());
    println!("strategy {:?}", strategy);

    let strategy = SelectionStrategy::TargetIdentifier("stm32f411".into());

    let registry = Registry::from_builtin_families();

    let target = registry.get_target(strategy).unwrap();
    println!("target {:?}", target);

    let mut session = Session::new(target, probe);

    let mm = session.target.memory_map.clone();

    let path_str = "../target/thumbv7em-none-eabihf/debug/examples/f401_ktest";
    // programming

    // print!("flashing...");
    // download_file(
    //     &mut session,
    //     std::path::Path::new(&path_str.to_string().as_str()),
    //     Format::Elf,
    //     &mm,
    // )
    // .map_err(|e| format_err!("failed to flash {}: {}", path_str, e))
    // .unwrap();

    // println!("... done");

    // let data = session.probe.read32(0x0000_0000).unwrap();
    // println!("stack 0x{:08x}", data);

    // let data = session.probe.read32(0x0000_0004).unwrap();
    // println!("reset 0x{:08x}", data);

    // run_to_halt(&mut session);

    // cycnt_enable(&mut session);
    // cycnt_reset(&mut session);

    // let cyccnt = cycnt_read(&mut session);
    // println!("cyccnt {}", cyccnt);

    // run_to_halt(&mut session);
    // let cyccnt = cycnt_read(&mut session);
    // println!("cyccnt {}", cyccnt);

    // run_to_halt(&mut session);

    // session
    //     .target
    //     .core
    //     .wait_for_core_halted(&mut session.probe)
    //     .unwrap();
    // println!("Core stopped at address 0x{:08x}", cpu_info.pc);

    // session
    //     .probe
    //     .write_block32(0x2000_0000, &[0x0123_4567, 0x89ab_cdef])
    //     .unwrap();

    // let mut r = [0u32; 2];
    // session.probe.read_block32(0x2000_0000, &mut r).unwrap();

    // println!("0x2000_0000 = 0x{:08x}", r[0]);
    // println!("0x2000_0004 = 0x{:08x}", r[1]);

    // let cpu_info = session.target.core.step(&mut session.probe).unwrap();
    // println!("Core stopped at address 0x{:08x}", cpu_info.pc);

    reset_and_halt(&mut session);
    run_to_halt(&mut session);

    for (name, data) in ktest.objects {
        set_symbolic(&mut session, &data);
        run_to_halt(&mut session);
    }

    println!("done");
    // session.target.core.run(&mut session.probe).unwrap();

    // session
    //     .target
    //     .core
    //     .wait_for_core_halted(&mut session.probe)
    //     .unwrap();
    // println!("Core stopped at address 0x{:08x}", cpu_info.pc);
    // println!("breapoint reached");
}

// resets the target and run
fn reset_and_run(session: &mut Session) {
    session.target.core.reset(&mut session.probe).unwrap();
}

// resets the target and halts
fn reset_and_halt(session: &mut Session) {
    session
        .target
        .core
        .reset_and_halt(&mut session.probe)
        .unwrap();
}

fn read_bkpt(session: &mut Session) -> Option<u8> {
    // try to read the program counter
    let pc_value = session
        .target
        .core
        .read_core_reg(&mut session.probe, session.target.core.registers().PC)
        .unwrap();

    let mut instr16 = [0u8; 2];
    session.probe.read_block8(pc_value, &mut instr16).unwrap();

    match instr16[1] {
        0b10111110 => Some(instr16[0]),
        _ => None,
    }
}

fn step_from_bkpt(session: &mut Session) {
    // try to read the program counter
    let pc_value = session
        .target
        .core
        .read_core_reg(&mut session.probe, session.target.core.registers().PC)
        .unwrap();

    // the bkpt() is a 16 bit instruction, increment pc by 16 bits
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

fn run_to_halt(session: &mut Session) {
    // Continue running
    if read_bkpt(session).is_some() {
        println!("Continue from breakpoint.");
        step_from_bkpt(session);
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
// index is the oject number
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

fn open_probe() -> MasterProbe {
    let mut devs = stlink::tools::list_stlink_devices();
    // just pick the first one
    let device = devs.get(0).unwrap();
    println!("device {:?}", device);
    let mut link = stlink::STLink::new_from_probe_info(&device).unwrap();

    link.attach(Some(WireProtocol::Swd)).unwrap();

    MasterProbe::from_specific_probe(link)
}

const DWT_CTRL: u32 = 0xe000_1000;
const DWT_CYCCNT: u32 = 0xe000_1004;

fn cycnt_enable(session: &mut Session) {
    session.probe.write32(DWT_CTRL, 0x1).unwrap();
}

fn cycnt_disable(session: &mut Session) {
    session.probe.write32(DWT_CTRL, 0x0).unwrap();
}

fn cycnt_reset(session: &mut Session) {
    // Reset cycle counter to 0
    session.probe.write32(DWT_CYCCNT, 0x0).unwrap();
}

fn cycnt_read(session: &mut Session) -> u32 {
    session.probe.read32(DWT_CYCCNT).unwrap()
}
