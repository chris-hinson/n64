//This file contains everything needed for our mid-leve IR
//the general idea is that we will take guest ops (mips opcodes) and turn them into an IrOp
//then we can turn the IrOp into a host opcode in a platform agnostic manner
//this host opcode is what will then be added into an executablebuffer considerd a compiled block

use crate::cpu::GPR;
use colored::Colorize;

/*pub enum IrOp {
    //load/store ops
    //load byte
    //compute ops
    //jump/branch ops (headaches)

    //special ops (these will be the hardest to translate, probably involve getting a raw pointer into our cpu struct and using a host load/store)
    //cop instrs (similar to special ops)
    //cop0 ops (system control)
}*/

//Load
//Store
//do math
//control flow
//system control

#[derive(Debug, Clone)]
pub enum Op {
    Load {
        width: usize,
        //this needs to be an enum because of our LWC and SWC ops
        dest: GPRorCoPGPR,
        base: Option<GPR>,
        offset: Option<u16>,
        condtional: bool,
        aligned: bool,
        imm_src: Option<usize>,
    },
    Store {
        width: usize,
        //this needs to be an enum because of our LWC and SWC ops
        src: GPRorCoPGPR,
        base: Option<GPR>,
        offset: Option<u16>,
        conditional: bool,
        aligned: bool,
        imm_src: Option<usize>,
    },

    //computational instructions that use the alu
    AluOp {
        op_type: AluOps,
        //false = 32 bit, true = 64 bit (doubleword ops)
        //NOTE: we are eliminating this field bc it is an arbitrary distinction that does not matter until runtime, as we will act based upon whatever mode the cpu is in at that time
        //mode: bool,
        dst: GPR,
        //imm_src: Option<u16>,
        src_1: Option<GPR>,
        src_2: AluOpSrc,
        //shamt: Option<shamt>,
    },

    //TODO: how the FUCK do we want to handle the coprocessor conditionals??
    ControlFlow {
        conditional: ControlConditionalType,
        //NOTE: we say destination and not offset because even for branch co
        destination: ControlDestType,
        register: Option<GPRorCoPGPR>,
        //fuck you mips. need this field so we know if we are dealing with delay slots or not
        likely: bool,
        link: bool,
    },

    Move {
        src: GPRorCoPGPR,
        dest: GPRorCoPGPR,
    },

    System {
        opcode: SystemOp,
    },

    //this WILL fire a
    MalformedOp,
}

#[derive(Debug, Clone)]
pub enum SystemOp {
    Cache,
    Syscall,
    Break,
    Sync,
    Trap { condition: ControlConditionalType },
    Tlb,
    Eret,
}

#[derive(Debug, Clone)]
pub enum AluOps {
    //immeadiate ops
    //These are all if I-type form
    ADDI,
    ADDIU,
    SLTI,
    SLTIU,
    ANDI,
    ORI,
    XORI,
    LUI,
    DADDI,
    DADDIU,
    //3-op ops
    //these are all of R-type form
    ADD,
    ADDU,
    SUB,
    SUBU,
    SLT,
    SLTU,
    AND,
    OR,
    XOR,
    NOR,
    DADD,
    DADDU,
    DSUB,
    DSUBU,
    SLL,
    SRL,
    SRA,
    SLLV,
    SRLV,
    SRAV,
    DSLL,
    DSRL,
    DSRA,
    DSLLV,
    DSRLV,
    DSRAV,
    DSLL32,
    DSRL32,
    DSRA32,
    //mult/div
    //these are also 3-op R-type forms
    //this is integer mult and div,
    MULT,
    MULTU,
    DIV,
    DIVU,
    DMULT,
    DMULTU,
    DDIV,
    DDIVU,
    //NOTE: should we move this to memory ops?
    MFHI,
    MFLO,
    MTHI,
    MTLO,
}
#[derive(Debug, Clone)]
pub enum AluOpSrc {
    Imm(u16),
    Reg(GPR),
}

#[derive(Debug, Clone)]
pub enum shamt {
    Imm(u8),
    Variable(GPR),
}

#[derive(Debug, Clone)]
pub enum GPRorCoPGPR {
    gpr(GPR),
    cop,
}

#[derive(Debug, Clone)]
pub enum ControlConditionalType {
    Unconditional,
    CopZFalse { cop: usize },
    CopZTrue { cop: usize },
    Eq { reg2: GPR },
    Ne { reg2: GPR },
    GEZ,
    GTZ,
    LEZ,
    LTZ,
}

#[derive(Debug, Clone)]
pub enum ControlDestType {
    Absolute { dest: usize },
    Relative { offset: i64 },
}

//this function takes a guest machine code basic block as parsed elsewhere and returns an ir basic block
pub fn lift(block: Vec<(u64, u32)>) -> Vec<(Op, u64)> {
    let disas = mipsasm::Mipsasm::new();

    let mut ir_block: Vec<(Op, u64)> = Vec::new();

    for instr in block {
        log::info!(
            "{}",
            format!(
                "lifting {:#x}: {}\t{:#x}",
                instr.0,
                disas.disassemble(&[instr.1])[0],
                instr.1
            )
            .blue()
        );
        let ir_op = guest_to_ir(instr.1).unwrap();
        log::info!("{}\n", format!("{:?}", ir_op).blue());

        ir_block.push((ir_op, instr.0));
    }

    ir_block
}

#[derive(Debug, Clone)]
pub enum LiftError {}

//we always return opcodes in an Ok variant, regardless of if they are valid opcodes or not
//catching invalid opcodes is a runtime issue
//the Err vatiant here is used for language err handling (catching panics and such)
pub fn guest_to_ir(instr: u32) -> Result<Op, LiftError> {
    //DEBUG: need this to pretty print disasembly
    let disas = mipsasm::Mipsasm::new();

    //top 5 bits are always the opcode, with the caveat that a full 0 opcode needs more decoding
    let opcode = (instr & 0xFC00_0000) >> 26;

    //lets just always mask out all our field so they are available for use if we want
    let r_op_rs = ((instr & 0x03E0_0000) >> 21) as u8;
    let r_op_rt = ((instr & 0x001F_0000) >> 16) as u8;
    let r_op_rd = ((instr & 0x0000_F800) >> 11) as u8;
    let r_op_shamt = ((instr & 0x0000_07C0) >> 6) as u8;
    let r_sub_op = (instr & 0x0000_003F) as u8;

    let i_op_rs = ((instr & 0x03E0_0000) >> 21) as u8;
    let i_op_rt = ((instr & 0x001F_0000) >> 16) as u8;
    let i_op_imm = (instr & 0x0000_FFFF) as u16;

    return match opcode {
        //SPECIAL decoding
        0x0 => {
            let sub_opcode = instr & 0x0000_003F;
            match sub_opcode {
                //SLL
                0x00 => Ok(Op::AluOp {
                    op_type: AluOps::SLL,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Imm(r_op_shamt.into()),
                }),

                //RESERVED INSTRUCTION EXCEPTION
                0x01 => Ok(Op::MalformedOp),
                //SRL
                0x02 => Ok(Op::AluOp {
                    op_type: AluOps::SRL,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Imm(r_op_shamt.into()),
                }),

                //SRA
                0x03 => Ok(Op::AluOp {
                    op_type: AluOps::SRA,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Imm(r_op_shamt.into()),
                }),

                //SLLV
                0x04 => Ok(Op::AluOp {
                    op_type: AluOps::SLLV,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Reg(r_op_rs.into()),
                }),

                //RESERVED INSTRUCTION EXCEPTION
                0x05 => {
                    return Ok(Op::MalformedOp);
                }
                //SRLV
                0x06 => Ok(Op::AluOp {
                    op_type: AluOps::SLLV,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Reg(r_op_rs.into()),
                }),
                //SRAV
                0x07 => Ok(Op::AluOp {
                    op_type: AluOps::SRAV,
                    dst: r_op_rd.into(),
                    src_1: Some(r_op_rt.into()),
                    src_2: AluOpSrc::Reg(r_op_rs.into()),
                }),
                _ => unimplemented!(
                    "decoded unimplemented opcode in SPECIAL decoding. bit pattern: {:x},  {}",
                    instr,
                    disas.disassemble(&[instr])[0]
                ),
            }
        }
        //REGIMM decoding
        //0x1 => {}

        //BNE
        0x5 => Ok(Op::ControlFlow {
            conditional: ControlConditionalType::Ne {
                reg2: i_op_rt.into(),
            },
            destination: ControlDestType::Relative {
                offset: i_op_imm.into(),
            },
            register: Some(GPRorCoPGPR::gpr(i_op_rs.into())),
            likely: false,
            link: false,
        }),

        //ANDI
        0xC => Ok(Op::AluOp {
            op_type: AluOps::ANDI,
            dst: i_op_rt.into(),
            src_1: Some(i_op_rt.into()),
            src_2: AluOpSrc::Imm(i_op_imm),
        }),

        //ORI
        0xD => Ok(Op::AluOp {
            op_type: AluOps::ORI,
            dst: i_op_rt.into(),
            src_1: Some(i_op_rs.into()),
            src_2: AluOpSrc::Imm(i_op_imm),
        }),

        //LUI
        0xF => Ok(Op::Load {
            width: 16,
            dest: GPRorCoPGPR::gpr(i_op_rt.into()),
            base: None,
            offset: None,
            condtional: false,
            aligned: true,
            imm_src: Some((i_op_imm as usize) << 16),
        }),

        //LW
        0x23 => Ok(Op::Load {
            width: 32,
            dest: GPRorCoPGPR::gpr(i_op_rt.into()),
            base: Some(i_op_rs.into()),
            offset: Some(i_op_imm),
            condtional: false,
            aligned: true,
            imm_src: None,
        }),

        //SW
        0x2B => Ok(Op::Store {
            width: 32,
            src: GPRorCoPGPR::gpr(i_op_rt.into()),
            base: Some(i_op_rs.into()),
            offset: Some(i_op_imm),
            conditional: false,
            aligned: true,
            imm_src: None,
        }),
        _ => unimplemented!(
            "decoded unimplemented opcode in main decoding. bit pattern: {:x},  {}",
            instr,
            disas.disassemble(&[instr])[0]
        ),
    };

    //Err(LiftError::InvalidOpcodeError)
}
