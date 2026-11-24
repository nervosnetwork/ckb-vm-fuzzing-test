use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::decoder::{DefaultDecoder, InstDecoder};
use ckb_vm::elf::parse_elf;
use ckb_vm::machine::asm::traces::SimpleFixedTraceDecoder;
use ckb_vm::machine::asm::{AsmCoreMachine, AsmDefaultMachineBuilder, AsmMachine};
use ckb_vm::machine::{VERSION0, VERSION1, VERSION2};
use ckb_vm::memory::{FLAG_DIRTY, FLAG_EXECUTABLE};
use ckb_vm::registers::{S5, S6};
use ckb_vm::snapshot2::{DataSource, Snapshot2Context};
use ckb_vm::{
    Bytes, CoreMachine, DefaultCoreMachine, DefaultMachineRunner, Error, ISA_B, ISA_IMC, ISA_MOP,
    Memory, RISCV_PAGESIZE, RustDefaultMachineBuilder, SparseMemory, SupportMachine, WXorXMemory,
};

const SIMPLE: &[u8] = include_bytes!("../../tests/programs/simple64");
const MULW: &[u8] = include_bytes!("../../tests/programs/mulw64");
const MEMORY_SIZE: usize = 512 * 1024;
const LIMIT: u64 = 200_000;
const PAGE: u64 = RISCV_PAGESIZE as u64;
// Between the fixtures' PT_LOAD segments and the VM stack.
const DATA: u64 = 0x20_000;
type Core = DefaultCoreMachine<u64, WXorXMemory<SparseMemory<u64>>>;
type Machine = ckb_vm::DefaultMachine<Core, DefaultDecoder>;

fn version_and_isa(selector: u8) -> (u32, u8) {
    match selector % 3 {
        0 => (VERSION0, ISA_IMC),
        1 => (VERSION1, ISA_IMC | ISA_B | ISA_MOP),
        _ => (VERSION2, ISA_IMC | ISA_B | ISA_MOP),
    }
}

fn interpreter(version: u32, isa: u8) -> Machine {
    RustDefaultMachineBuilder::new(Core::new_with_memory(isa, version, LIMIT, MEMORY_SIZE))
        .instruction_cycle_func(Box::new(estimate_cycles))
        .build()
}

fn input<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut out = [0; N];
    let n = N.min(data.len());
    out[..n].copy_from_slice(&data[..n]);
    out
}

pub fn backend_steps(data: &[u8]) {
    let b = input::<18>(data);
    let (version, isa) = version_and_isa(b[0]);
    let left = u64::from_le_bytes(b[2..10].try_into().unwrap());
    let right = u64::from_le_bytes(b[10..18].try_into().unwrap());
    let product = (left as i32).wrapping_mul(right as i32) as i64 as u64;
    let mut rust = interpreter(version, isa);
    let asm_core = AsmCoreMachine::new_with_memory(isa, version, LIMIT, MEMORY_SIZE);
    let mut asm = AsmMachine::new(
        AsmDefaultMachineBuilder::new(asm_core)
            .instruction_cycle_func(Box::new(estimate_cycles))
            .build(),
    );
    let program = Bytes::from_static(MULW);
    rust.load_program(&program, std::iter::empty()).unwrap();
    asm.load_program(&program, std::iter::empty()).unwrap();
    let mut rd = DefaultDecoder::new::<u64>(isa, version);
    let mut ad = SimpleFixedTraceDecoder::new::<u64>(isa, version);
    rust.set_running(true);
    asm.machine.set_running(true);
    // The pinned mulw.S initializes s5/s6 in the first two instructions.
    for step in 0..16 {
        if step == 2 {
            for (register, value) in [(S5, left), (S6, right)] {
                rust.set_register(register, value);
                asm.machine.set_register(register, value);
            }
            if b[1] & 1 != 0 {
                let budget = rust.cycles();
                rust.set_max_cycles(budget);
                asm.set_max_cycles(budget);
                assert_eq!(rust.step(&mut rd), Err(Error::CyclesExceeded));
                assert_eq!(asm.step(&mut ad), Err(Error::CyclesExceeded));
                assert_eq!((rust.cycles(), asm.machine.cycles()), (budget, budget));
                assert_eq!(rust.pc(), asm.machine.pc());
                return;
            }
        }
        rust.step(&mut rd).unwrap();
        asm.step(&mut ad).unwrap();
        assert_eq!(rust.pc(), asm.machine.pc());
        assert_eq!(rust.registers(), asm.machine.registers());
        assert_eq!(rust.cycles(), asm.machine.cycles());
        assert_eq!(rust.running(), asm.machine.running());
        if step == 2 {
            assert_eq!(rust.registers()[S5], product);
        }
        if !rust.running() {
            assert!(step >= 2);
            assert_eq!(rust.exit_code(), i8::from(product != u64::MAX));
            assert_eq!(rust.exit_code(), asm.machine.exit_code());
            return;
        }
    }
    panic!("mulw64 did not exit within its instruction bound");
}

#[derive(Clone)]
struct Pages(Bytes);

impl DataSource<u8> for Pages {
    fn load_data(&self, id: &u8, offset: u64, length: u64) -> Option<(Bytes, u64)> {
        if *id != 1 {
            return None;
        }
        let start = offset.min(self.0.len() as u64) as usize;
        let remaining = self.0.len() - start;
        let count = if length == 0 {
            remaining
        } else {
            length.min(remaining as u64) as usize
        };
        Some((self.0.slice(start..start + count), remaining as u64))
    }
}

fn same_state(a: &mut Machine, b: &mut Machine) {
    assert_eq!(a.registers(), b.registers());
    assert_eq!(
        (a.pc(), a.cycles(), a.max_cycles()),
        (b.pc(), b.cycles(), b.max_cycles())
    );
    for page in 0..(MEMORY_SIZE / RISCV_PAGESIZE) as u64 {
        let flags = a.memory_mut().fetch_flag(page).unwrap();
        assert_eq!(flags, b.memory_mut().fetch_flag(page).unwrap());
        // Compare guest writes (including the stack) and the clean source-backed page.
        if flags & FLAG_DIRTY != 0 || page == DATA / PAGE {
            assert_eq!(
                a.memory_mut().load_bytes(page * PAGE, PAGE).unwrap(),
                b.memory_mut().load_bytes(page * PAGE, PAGE).unwrap()
            );
        }
    }
}

pub fn snapshot2_continue(data: &[u8]) {
    let b = input::<16>(data);
    let (version, isa) = version_and_isa(b[0]);
    let source = Pages(
        (0..3 * RISCV_PAGESIZE)
            .map(|i| (i as u8).wrapping_add(b[4]))
            .collect::<Vec<_>>()
            .into(),
    );
    let start = || {
        let mut vm = interpreter(version, isa);
        vm.load_program(
            &Bytes::from_static(SIMPLE),
            [Ok(Bytes::copy_from_slice(&b[8..]))].into_iter(),
        )
        .unwrap();
        let mut ctx = Snapshot2Context::new(source.clone());
        ctx.store_bytes(&mut vm, DATA, &1, 0, 3 * PAGE, DATA + 3 * PAGE)
            .unwrap();
        let address = DATA + 2 * PAGE - 1 - u64::from(b[5] % 4);
        vm.memory_mut().store_bytes(address, &b[8..16]).unwrap();
        (vm, ctx)
    };
    let (mut baseline, _) = start();
    let (mut current, mut ctx) = start();
    let expected = baseline.run().unwrap();
    assert_eq!(expected, 0);
    let pauses = 1 + usize::from(b[1] & 1);
    for &count in &b[2..2 + pauses] {
        let mut decoder = DefaultDecoder::new::<u64>(isa, version);
        current.set_running(true);
        for _ in 0..1 + count % 32 {
            current.step(&mut decoder).unwrap();
            assert!(current.running());
        }
        let snap = ctx.make_snapshot(&mut current).unwrap();
        assert!(
            snap.pages_from_source
                .iter()
                .any(|&(addr, _, id, _, len)| addr == DATA && id == 1 && len == PAGE)
        );
        for index in 1..=2 {
            assert_ne!(
                current
                    .memory_mut()
                    .fetch_flag(DATA / PAGE + index)
                    .unwrap()
                    & FLAG_DIRTY,
                0
            );
        }
        let mut restored = interpreter(version, isa);
        ctx = Snapshot2Context::new(source.clone());
        ctx.resume(&mut restored, &snap).unwrap();
        same_state(&mut current, &mut restored);
        current = restored;
    }
    assert_eq!(current.run().unwrap(), expected);
    same_state(&mut baseline, &mut current);
}

pub fn elf_page_edges(data: &[u8]) {
    let b = input::<16>(data);
    let (version, isa) = version_and_isa(b[0]);
    let original = Bytes::from_static(if b[1] & 1 == 0 { SIMPLE } else { MULW });
    let metadata = parse_elf::<u64>(&original, version).unwrap();
    let end = metadata
        .actions
        .iter()
        .map(|s| s.source.end as usize)
        .max()
        .unwrap();
    // Drop the optional section table along with its ELF64 header references.
    let mut bytes = original[..end].to_vec();
    bytes[40..48].fill(0); // e_shoff
    bytes[58..64].fill(0); // e_shentsize, e_shnum, e_shstrndx
    let exact = Bytes::from(bytes);
    assert_eq!(parse_elf::<u64>(&exact, version).unwrap(), metadata);
    let short = exact.slice(..end - 1 - usize::from(b[2] % 8));
    assert!(matches!(
        parse_elf::<u64>(&short, version),
        Err(Error::ElfSegmentAddrOrSizeError(_))
    ));
    let mut vm = interpreter(version, isa);
    vm.load_program(&exact, std::iter::empty()).unwrap();
    let memory = vm.memory_mut();
    let boundary = DATA + PAGE;
    let start = boundary - u64::from(b[3] % 9);
    let size = 1usize << (b[4] % 4);
    let executable = |page: u64| {
        if page == DATA / PAGE {
            b[5] & 1 != 0
        } else {
            b[5] & 2 != 0
        }
    };
    for page in DATA / PAGE..=boundary / PAGE {
        if executable(page) {
            memory.set_flag(page, FLAG_EXECUTABLE).unwrap();
        }
    }
    let pages = |n: usize| start / PAGE..=(start + n as u64 - 1) / PAGE;
    let result = memory.store_bytes(start, &b[8..8 + size]);
    let actual = memory.load_bytes(start, size as u64).unwrap();
    if let Some(page) = pages(size).find(|&p| executable(p)) {
        assert_eq!(result, Err(Error::MemWriteOnExecutablePage(page)));
        assert_eq!(actual.as_ref(), &([0; 8])[..size]);
    } else {
        result.unwrap();
        assert_eq!(actual.as_ref(), &b[8..8 + size]);
    }
    let fetch = memory.execute_load16(start);
    if let Some(page) = pages(2).find(|&p| !executable(p)) {
        assert_eq!(fetch, Err(Error::MemWriteOnExecutablePage(page)));
    } else {
        assert_eq!(fetch.unwrap(), 0);
    }
}
