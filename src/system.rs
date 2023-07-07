use crate::cart::Cart;
use crate::cpu::Cpu;
use crate::cpu::GPR;
use crate::ir::ControlConditionalType;
use crate::ir::{AluOps::*, Op};
use crate::pi::PI;
use crate::rcp::Rcp;
use crate::rdram::Rdram;
use colored::Colorize;
use log::trace;
use log::{debug, info};
//use std::borrow::BorrowMut;
//use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Shl;
use std::rc::Rc;

use mipsasm::Mipsasm;

pub struct System {
    //every single hardware piece goes here
    //every piece of hardware gets an RF to every other piece of hardware that it can talk to
    pub cpu: Rc<RefCell<Cpu>>,
    pub cart: Rc<RefCell<Cart>>,
    pub rcp: Rc<RefCell<Rcp>>,
    pub pi: Rc<RefCell<PI>>,
    pub rdram: Rc<RefCell<Rdram>>,
}

#[derive(Debug)]
pub enum SystemResult {
    Graceful,
    Errored,
}

const BRANCH_OR_JUMP_OPS: [u8; 17] = [
    0b0100_00, 0b0100_01, 0b0100_10, 0b0100_11, //these are all branch on cop
    0b0001_00, //BEQ
    0b0101_00, //BEQL
    0b0000_01, //BGEZ, BGEZAL, BGEZALL, BGEZL
    0b0001_11, //BGTZ,
    0b0101_11, //BGTZL
    0b0001_10, //BLEZ
    0b0101_10, //BLEZL
    0b0000_01, //BLTZ, BLTZAL, BLTZALL, BLTZL
    0b0001_01, //BNE
    0b0101_01, //BNEL
    0b0000_10, //J
    0b0000_11, //JAL
    0b0000_00, //JALR (special), JR
];

#[derive(Debug)]
pub enum ExecutionError {}

impl System {
    pub fn run(&mut self) -> Result<SystemResult, SystemResult> {
        info!("system run start");

        //ipl1
        //PC starts off at  0xBFC00000, which is in PIF-rom
        //we can skip this by just emulating the side effects of it
        //most significantly, it bootstraps ipl2 into IMEM
        //jumps to the start of it at 0x04001000

        debug!("start ipl1");
        self.ipl1();
        debug!("done ipl1");

        //ipl2
        //starts at 0x04001000
        //bootstraps ipl3 into DMEM (first 0x1000 from game cart)
        debug!("start ipl2");
        self.ipl2();
        debug!("done ipl2");

        //ipl3
        //does some shit and then starts executing
        /*let cpu_ref = self.cpu.borrow_mut();
        let cur_pc = cpu_ref.rf.PC;
        let cur_instr = self.read(cpu_ref.rf.PC.try_into().unwrap(), 4).unwrap();
        println!(
            "PC is at {:#x}, and sees: {:02x}{:02x}{:02x}{:02x} ",
            cur_pc, cur_instr[0], cur_instr[1], cur_instr[2], cur_instr[3]
        );*/

        let disas = mipsasm::Mipsasm::new();

        loop {
            let block_base = self.cpu.borrow().rf.PC;
            let block = self.find_next_basic_block();
            println!("found basic block:");
            for (idx, instr) in block.iter().enumerate() {
                println!(
                    "\t{:#x}, {:#x}: {}",
                    //(block_base + (idx as u64 * 4)),
                    instr.0,
                    instr.1,
                    disas.disassemble(&[instr.1])[0]
                );
            }
            let ir_block = crate::ir::lift(block);
            for op in &ir_block {
                self.execute_IR(op.0.clone(), op.1).unwrap();
                self.cpu.borrow_mut().rf.PC += 4;
            }
            drop(ir_block)
        }

        Ok(SystemResult::Graceful)
    }

    pub fn execute_IR(&mut self, op: Op, addr: u64) -> Result<usize, ExecutionError> {
        info!(
            "{}",
            format!("executing ir op {:#x},{:?}", addr, op).green()
        );
        match op {
            Op::Load {
                width,
                dest,
                base,
                offset,
                condtional,
                aligned,
                imm_src,
            } => {
                if aligned == false {
                    unimplemented!("implement unaligned memory accesses")
                }
                if condtional == true {
                    unimplemented!("implement load conditional")
                }

                //calculate the value we are loading
                let value = match base {
                    Some(r) => {
                        let base_addr = self.cpu.borrow_mut().rf[r];
                        let final_addr = base_addr + offset.unwrap_or(0) as u64;
                        let bytes = self.read(final_addr as u32, width / 8).unwrap();

                        if width != 32 {
                            panic!("chris you gotta go implement reads for multiple widths")
                        }
                        bytes[0] as u32
                            | (bytes[1] as u32) << 8
                            | (bytes[2] as u32) << 16
                            | (bytes[3] as u32) << 24
                    }
                    None => imm_src.unwrap() as u32,
                };

                //write to destination
                match dest {
                    crate::ir::GPRorCoPGPR::gpr(r) => self.cpu.borrow_mut().rf[r] = value as u64,
                    crate::ir::GPRorCoPGPR::cop => {
                        unimplemented!("implement load to coprocessor")
                    }
                }
            }
            Op::Store {
                width,
                src,
                base,
                offset,
                conditional,
                aligned,
                imm_src,
            } => {
                if aligned == false {
                    unimplemented!("implement unaligned memory accesses in store")
                }
                if conditional == true {
                    unimplemented!("implement store conditional")
                }
                if width != 32 {
                    unimplemented!("implement stores for variable width")
                }

                let val = if imm_src.is_none() {
                    match src {
                        crate::ir::GPRorCoPGPR::gpr(r) => self.cpu.borrow_mut().rf[r] as u32,
                        crate::ir::GPRorCoPGPR::cop => {
                            unimplemented!("implement stores from coprocessor")
                        }
                    }
                } else {
                    imm_src.unwrap() as u32
                };

                let address =
                    self.cpu.borrow_mut().rf[base.unwrap()] as u32 + offset.unwrap_or(0) as u32;
                self.write(address, val.to_le_bytes().to_vec());
            }
            Op::AluOp {
                op_type,
                dst,
                src_1,
                src_2,
            } => {
                //get a closure to represent the actual operation
                let function = match op_type {
                    //ADD => |num1: u32, num2: u32| -> u32 {num1.wrapping_add(num2)},
                    ORI => |num1: u64, num2: u64| -> u64 { num1 | num2 },
                    ANDI => |num1: u64, num2: u64| -> u64 { num1 & num2 },
                    SLL => |num1: u64, num2: u64| -> u64 { num1.shl(num2) },
                    _ => unimplemented!(
                        "PANIC: this alu opcode does not have an cloosure implemented for it yet"
                    ),
                };
                //figure out where we are getting our values from
                let a = match src_1 {
                    Some(r) => self.cpu.borrow().rf[r],
                    None => panic!(
                        "panicking executing an AluOP, no a src. Does this make sense in context?"
                    ),
                };

                let b = match src_2 {
                    crate::ir::AluOpSrc::Imm(v) => v as u64,
                    crate::ir::AluOpSrc::Reg(r) => self.cpu.borrow().rf[r],
                };

                //execute the closure for this operation
                let result = function(a, b);

                //writeback to the cpu destination
                self.cpu.borrow_mut().rf[dst] = result;
            }
            Op::ControlFlow {
                conditional,
                destination,
                register,
                likely,
                link,
            } => {
                if likely {
                    unimplemented!("hit a likely control flow. need to add support for this");
                }
                if link {
                    unimplemented!("hit a link control flow. need to add support for this");
                }

                let reg1 = match register {
                    Some(v) => match v {
                        crate::ir::GPRorCoPGPR::cop => {
                            unimplemented!("got a cop conditional in control flow")
                        }
                        crate::ir::GPRorCoPGPR::gpr(r) => Some(r),
                    },
                    None => None,
                };

                let operation = match conditional {
                    ControlConditionalType::Ne { reg2 } => || -> bool {
                        let rf = &self.cpu.borrow_mut().rf;
                        //self.cpu.borrow_mut().rf[reg2] != self.cpu.borrow_mut().rf[reg1.unwrap()]
                        rf[reg1.unwrap()] != rf[reg2]
                    },
                    _ => unimplemented!("hit a conditional condition we havent implemented yet"),
                };
                let take = operation();
                //drop(operation);

                if take {
                    match destination {
                        crate::ir::ControlDestType::Absolute { dest } => {
                            self.cpu.borrow_mut().rf.PC = dest as u64;
                        }
                        crate::ir::ControlDestType::Relative { offset } => {
                            self.cpu.borrow_mut().rf.PC =
                                self.cpu.borrow_mut().rf.PC.wrapping_add_signed(offset);
                        }
                    }
                }

                //unimplemented!("control flow opcodes not implemented yet");
            }
            Op::Move { src, dest } => {
                unimplemented!("moce opcodes not implemented yet");
            }
            Op::System { opcode } => {
                unimplemented!("System opcodes not implemented yet");
            }
            Op::MalformedOp => {
                panic!("malformed op in execution function!")
            }
        }

        Ok(0)
    }

    //returns a block of addresses and the opcodes at those addresses
    pub fn find_next_basic_block(&self) -> Vec<(u64, u32)> {
        //uhhhh. search forward from pc until we see a branch instruction!
        //dont forget to always include that pesky delay slot!

        let mut base_pc = self.cpu.borrow().rf.PC;

        //TODO: this might need to become u64s if we are for some reason running in 64 bit mode
        let mut block_vec: Vec<(u64, u32)> = Vec::new();

        loop {
            let next_instr = self.read(base_pc.try_into().unwrap(), 4).unwrap();
            let next_instr = (next_instr[0] as u32) << 24
                | (next_instr[1] as u32) << 16
                | (next_instr[2] as u32) << 8
                | next_instr[3] as u32;

            if BRANCH_OR_JUMP_OPS.contains(&(((next_instr & 0xFC00_0000) >> 26) as u8))
                || ((next_instr & 0xFC00_0000) >> 26 == 0
                    && (((next_instr & 0b111111) == 0b001000)
                        || ((next_instr & 0b111111) == 0b001001)))
            {
                //push the branch instruction here
                block_vec.push((base_pc, next_instr));

                base_pc += 4;

                //PUSH THE FUCKING DELAY SLOT INSTRUCTION HERE
                let next_instr = self.read(base_pc.try_into().unwrap(), 4).unwrap();
                let next_instr = (next_instr[0] as u32) << 24
                    | (next_instr[1] as u32) << 16
                    | (next_instr[2] as u32) << 8
                    | next_instr[3] as u32;
                block_vec.push((base_pc, next_instr));
                break;
            } else {
                block_vec.push((base_pc, next_instr));
            }

            base_pc += 4;
        }

        block_vec
    }

    pub fn new() -> Self {
        //System {}
        debug!("constructing cpu");
        //construct resources
        let ram = Rc::new(RefCell::new(Rdram::default()));
        let cart = Rc::new(RefCell::new(Cart::new("./roms/basic_simpleboot.z64")));
        //construct computational units
        let cpu = Rc::new(RefCell::new(Cpu::new(ram.clone(), cart.clone())));
        let rcp = Rc::new(RefCell::new(Rcp::new()));
        let pi = Rc::new(RefCell::new(PI::default()));
        let rdram = Rc::new(RefCell::new(Rdram::default()));

        //construct the actuall system
        Self {
            cpu,
            cart,
            rcp,
            pi,
            rdram,
        }
    }

    //ipl1 boot sequence
    //taken from https://n64brew.dev/wiki/Initial_Program_Load#IPL1
    pub fn ipl1(&mut self) {
        //segment 1
        //status_register = 0x34000000
        self.cpu.borrow_mut().cop0.Status = 0x34000000.into();

        //config_register  = 0x0006E463
        //cpu.cop0.Config = 0x0006E463.into();
        self.cpu.borrow_mut().cop0.Config = 0x0006E463.into();

        ///////////segment 2/////////////////////
        //we basically just set a bunch of stuff to 0 here so we skip it
        /////////////////////////////////////////

        //////////segment 3//////////////////////
        // copy ipl2 into imem
        //ignore this for now since we are just emulating the side effects of ipl2

        //uh technically we are supposed to set SP here so lets do it in case we need the side effect
        self.cpu.borrow_mut().rf[GPR::sp] = 0xA4001FF0;
        /////////////////////////////////////////
    }

    //ipl2 boot sequence
    //going based off of: https://n64brew.dev/wiki/PIF-NUS#Console_startup
    pub fn ipl2(&mut self) {
        //there is fuck all documentation for whats actually going on here
        //we are initializing some hw (WHAT FUCKING HARDWARE)
        //doing some cic checks (fuck that i trust u ig)

        //copying ipl3 to DMEM
        //Load IPL3 from the cartridge ROM (offset 0x40-0x1000) into the RSP DMEM
        unsafe {
            let mut src_ptr = self.cart.borrow_mut().rom.as_ptr();
            //src_ptr = src_ptr.add(0x40);
            let dst_ptr = self.rcp.borrow_mut().rsp.borrow_mut().DMEM.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, 0x1000);
        }

        //jumping to ipl3 (this is where we will start actually executing instead of just fuzzing the same effects)
        //base of dmem is 0x04000000(phys) which ASSUMING we are kseg1, is then a virtual address of 0x04000000 + 0xA0000000 = 0xA4000000
        self.cpu.borrow_mut().rf.PC = 0xA4000040;
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////

    //lets assume that we are always passing virtual addresses here, and then we can handle all mmu
    //translations here and return or write data to physical addresses
    //length is in bytes
    pub fn read(&self, addr: u32, len: usize) -> Result<Vec<u8>, String> {
        let phys = self.virt_to_phys(addr);

        /*trace!(
            "in system::read, converted virt {:#x} to phys {:#x}",
            addr,
            phys
        );*/

        match phys {
            //RDRAM
            0x0000_0000..=0x03FFFFFF => self.rdram.borrow_mut().read(addr, len),

            //0x04000000 	0x04000FFF 	RSP DMEM 	RSP Data Memory
            //0x04001000 	0x04001FFF 	RSP IMEM 	RSP Instruction Memory
            //0x04002000 	0x0403FFFF 	RSP DMEM/IMEM Mirrors 	Mirrors of DMEM and IMEM (repeat every 8Kb)
            0x04000000..=0x0403FFFF => {
                let sub_address = (phys - 0x04000000) % 8192;
                match sub_address {
                    0x0..=0xFFF => {
                        return Ok(self.rcp.borrow().rsp.borrow().DMEM
                            [sub_address as usize..=sub_address as usize + len]
                            .to_vec());
                    } //DMEM
                    0x1000..=0x1FFF => {
                        return Ok(self.rcp.borrow().rsp.borrow().IMEM
                            [sub_address as usize..=sub_address as usize + len]
                            .to_vec());
                    } //IMEM
                    _ => unreachable!("calculated an impossible imem or dmem addr"),
                }
            }
            0x04600000..=0x046FFFFF => {
                return self.pi.borrow().read(phys, len);
            }
            _ => {
                panic!(
                    "trying to read to a physical address we havent mapped yet: {:#x}",
                    phys
                );
            }
        }
    }
    pub fn write(&mut self, addr: u32, val: Vec<u8>) {
        let phys = self.virt_to_phys(addr);

        match phys {
            //RCP PI address space NOT EXTERNAL BUS
            0x04600000..=0x046FFFFF => {
                let possible_dma = self.pi.borrow_mut().write(phys, val);
                if possible_dma.is_some() {
                    debug!("begin PI DMA");
                    let dma_packet = possible_dma.unwrap();
                    debug!("PI DMA packet: {}", dma_packet);

                    //execute the dma

                    let len = dma_packet.len;
                    let from = dma_packet.from;
                    let to = dma_packet.to;
                    /*let data = self.read(from, len).unwrap();

                    let to = dma_packet.to;
                    self.write(to, data);*/
                    //let mut from_ptr = std::ptr::null();
                    //let mut to_ptr = std::ptr::null();

                    //coming FROM cart to rdram
                    if dma_packet.from >= 0x10000000 {
                        let data = self.cart.borrow_mut().rom[(from - 0x10000000) as usize
                            ..((from - 0x10000000) as usize + len) as usize]
                            .to_vec();
                        self.rdram.borrow_mut().write(to, data).unwrap();
                    }
                    //coming FROM rdram to cart
                    else {
                        //to_ptr = self.cart.borrow_mut().rom.as_mut_ptr();
                        let mut data = self.rdram.borrow().read(from, len).unwrap();
                        //self.
                        unsafe {
                            let mut to_ptr = self.cart.borrow_mut().rom.as_mut_ptr();
                            to_ptr = to_ptr.add(to as usize);

                            let from_ptr = data.as_mut_ptr();

                            std::ptr::copy_nonoverlapping(from_ptr, to_ptr, len);
                        }
                    }

                    debug!("end PI DMA");
                }
            }
            _ => panic!("trying to write to a physical address we havent mapped yet: {phys:#x}"),
        }
    }

    pub fn virt_to_phys(&self, virt: u32) -> u32 {
        match virt {
            0x0000_0000..=0x7FFF_FFFF => {
                panic!("tried to convert a virtual address in KUSEG {virt:#x}")
            } //KUSEG
            0x8000_0000..=0x9FFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG0 {virt:#x}")
            } //KSEG0
            0xA000_0000..=0xBFFF_FFFF => {
                //trace!("in system::virt_to_phys, virt is {:#x}", virt);

                let conversion = virt.checked_sub(0xA000_0000);

                //trace!("after sub: {:?}", conversion);
                match conversion {
                    Some(v) => {
                        //trace!("value is {:#x}", v);
                        return v;
                    }
                    None => panic!("error converting address in KSEG1 {virt:#x}"),
                }
            } //KSEG1
            0xC000_0000..=0xDFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG2 {virt:#x}")
            } //KSEG2
            0xE000_0000..=0xFFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG3 {virt:#x}")
            } //KSEG3
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
}

//virtual map
/*
0x00000000 	0x7FFFFFFF 	KUSEG 	User segment, TLB mapped
0x80000000 	0x9FFFFFFF 	KSEG0 	Kernel segment 0, directly mapped, cached
0xA0000000 	0xBFFFFFFF 	KSEG1 	Kernel segment 1, directly mapped, uncached
0xC0000000 	0xDFFFFFFF 	KSSEG 	Kernel supervisor segment, TLB mapped
0xE0000000 	0xFFFFFFFF 	KSEG3 	Kernel segment 3, TLB mapped*/

//physical map
/*
Bus / Device 	Address Range 	Name 	Description
RDRAM
0x00000000 	0x03EFFFFF 	RDRAM memory-space 	RDRAM memory. See RDRAM_Interface#Memory_addressing and RDRAM#RDRAM_addressing for details about their mapping.
0x03F00000 	0x03F7FFFF 	RDRAM Registers 	RDRAM registers. See RDRAM_Interface#Memory_addressing and RDRAM#RDRAM_addressing for details about their mapping.
0x03F80000 	0x03FFFFFF 	RDRAM Registers (broadcast) 	Write-only. All connected RDRAM will act on this register write request. See RDRAM_Interface#Memory_addressing and RDRAM#RDRAM_addressing for details.
RCP
0x04000000 	0x04000FFF 	RSP DMEM 	RSP Data Memory
0x04001000 	0x04001FFF 	RSP IMEM 	RSP Instruction Memory
0x04002000 	0x0403FFFF 	RSP DMEM/IMEM Mirrors 	Mirrors of DMEM and IMEM (repeat every 8Kb)
0x04040000 	0x040BFFFF 	RSP Registers 	RSP DMAs, status, semaphore, program counter, IMEM BIST status
0x040C0000 	0x040FFFFF 	Unmapped 	This area is completely ignored by the RCP. Any access in this area will freeze the CPU as the RCP will ignore the read/write and the CPU will never receive a reply.
0x04100000 	0x041FFFFF 	RDP Command Registers 	RDP DMAs, clock counters for: clock, buffer busy, pipe busy, and TMEM load
0x04200000 	0x042FFFFF 	RDP Span Registers 	TMEM BIST status, DP Span testing mode
0x04300000 	0x043FFFFF 	MIPS Interface (MI) 	Init mode, ebus test mode, RDRAM register mode, hardware version, interrupt status, interrupt masks
0x04400000 	0x044FFFFF 	Video Interface (VI) 	Video control registers
0x04500000 	0x045FFFFF 	Audio Interface (AI) 	Audio DMAs, Audio DAC clock divider
0x04600000 	0x046FFFFF 	Peripheral Interface (PI) 	Cartridge port DMAs, status, Domain 1 and 2 speed/latency/page-size controls
0x04700000 	0x047FFFFF 	RDRAM Interface (RI) 	Operating mode, current load, refresh/select config, latency, error and bank status
0x04800000 	0x048FFFFF 	Serial Interface (SI) 	SI DMAs, PIF status
0x04900000 	0x04FFFFFF 	Unmapped 	This area is completely ignored by the RCP. Any access in this area will freeze the CPU as the RCP will ignore the read/write and the CPU will never receive a reply.

PI external bus
0x05000000 	0x05FFFFFF 	N64DD Registers 	Contains the N64DD I/O registers.

Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 1" configuration set. When not present, this is a PI open bus area.
0x06000000 	0x07FFFFFF 	N64DD IPL ROM 	Contains the N64DD ROM used during boot, sometimes called IPL4. This is executed whenever the console is turned on with a N64DD connected, in place of the IPL3.

Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 1" configuration set. When not present, this is a PI open bus area.
0x08000000 	0x0FFFFFFF 	Cartridge SRAM 	When the cartridge uses a SRAM for save games, this is conventionally exposed at this address range.

Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 2" configuration set. This is one of the few address ranges which are in Domain 2 probably because it is common to access a SRAM with a different (slower) protocol. When not present, this is a PI open bus area.
0x10000000 	0x1FBFFFFF 	Cartridge ROM 	The cartridges expose the ROM at this address. Normally, games will load assets and overlays via PI DMA for speed concerns, but the ROM is nonetheless memory mapped. Notice that cache accesses are not allowed here (and in all PI external bus accesses, see below for details), so while it is possible to run code directly from ROM, it will be extremely slow as it would not leverage the instruction cache.

Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 1" configuration set. When not present (eg: when booting a disk-only N64DD game without a cartridge), this is a PI open bus area.
SI external bus
0x1FC00000 	0x1FC007BF 	PIF ROM (IPL1/2) 	Executed on boot
0x1FC007C0 	0x1FC007FF 	PIF RAM 	Controller and EEPROM communication, and during IPL1/2 is used to read startup data from the PIF
0x1FC00800 	0x1FCFFFFF 	Reserved 	Unknown usage

PI external bus
0x1FD00000 	0x1FFFFFFF 	Unused 	Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 1" configuration set.

No known PI device uses this range, so it will normally be a PI open bus area.
0x20000000 	0x7FFFFFFF 	Unused 	Accesses here are forwarded to the PI bus, with the same address within the PI address space, using the "Domain 1" configuration set.

No known PI device uses this range, so it will normally be a PI open bus area.

NOTE: this range can be accessed by CPU only via TLBs or via direct 64-bit addressing, using the directly mapped, uncached segment (virtual 64-bit address: 0x9000_0000_nnnn_nnnn).
    0x80000000 	0xFFFFFFFF 	Unmapped 	This area is completely ignored by the RCP. Any access in this area will freeze the CPU as the RCP will ignore the read/write and the CPU will never receive a reply.
*/
