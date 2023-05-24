//This file contains everything needed for our mid-leve IR
//the general idea is that we will take guest ops (mips opcodes) and turn them into an IrOp
//then we can turn the IrOp into a host opcode in a platform agnostic manner
//this host opcode is what will then be added into an executablebuffer considerd a compiled block

use crate::cpu::GPR;

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

#[derive(Debug)]
pub enum Op {
    Load {
        width: usize,
        //this needs to be an enum because of our LWC and SWC ops
        dest: GPRorCoPGPR,
        base: GPR,
        offset: u16,
        condtional: bool,
        aligned: bool,
    },
    Store {
        widt: usize,
        //this needs to be an enum because of our LWC and SWC ops
        src: GPRorCoPGPR,
        base: GPR,
        offset: u16,
        conditional: bool,
        aligned: bool,
    },

    //computational instructions that use the alu
    AluOp {
        op_type: AluOps,
        //false = 32 bit, true = 64 bit (doubleword ops)
        mode: bool,
        val_src: AluOpSrc,
        dst: GPR,
        imm_src: Option<u16>,
        //TODO: WHAT IS THIS FOR CHRIS?!?!?!?
        reg_rc: Option<GPR>,
        shamt: Option<shamt>,
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

#[derive(Debug)]
pub enum SystemOp {
    Cache,
    Syscall,
    Break,
    Sync,
    Trap { condition: ControlConditionalType },
    Tlb,
    Eret,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub enum AluOpSrc {
    Imm(u16),
    Reg(GPR),
}

#[derive(Debug)]
pub enum shamt {
    Imm(u8),
    Variable(GPR),
}

#[derive(Debug)]
pub enum GPRorCoPGPR {
    gpr(GPR),
    cop,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum ControlDestType {
    Absolute { dest: usize },
    Relative { offset: i64 },
}

//this function takes a guest machine code basic block as parsed elsewhere and returns an ir basic block
pub fn lift(block: Vec<u32>) -> Vec<Op> {
    let mut ir_block = Vec::new();

    for instr in block {
        ir_block.push(guest_to_ir(instr).unwrap());
    }

    ir_block
}

#[derive(Debug)]
pub enum LiftError {
    InvalidOpcodeError,
    ReservedOpcodeError,
}

pub fn guest_to_ir(instr: u32) -> Result<Op, LiftError> {
    //top 5 bits are always the opcode, with the caveat that a full 0 opcode needs more decoding
    let opcode = (instr & 0xFC00_0000) >> 26;

    //lets just always mask out all our field so they are available for use if we want
    let r_op_rs = (0x03E0_0000 >> 21) as u8;
    let r_op_rt = (0x001F_0000 >> 16) as u8;
    let r_op_rd = (0x0000_F800 >> 11) as u8;
    let r_op_shamt = (0x0000_07C0 >> 6) as u8;
    let r_sub_op = 0x0000_003F as u8;

    match opcode {
        //SPECIAL decoding
        0x0 => {
            let sub_opcode = instr & 0x0000_003F;
            match sub_opcode {
                //SLL
                0x00 => {
                    return Ok(Op::AluOp {
                        op_type: AluOps::SLL,
                        mode: false,
                        val_src: AluOpSrc::Reg(r_op_rt.into()),
                        dst: r_op_rd.into(),
                        imm_src: None,
                        reg_rc: None,
                        shamt: Some(shamt::Imm(r_op_shamt)),
                    });
                }
                //RESERVED INSTRUCTION EXCEPTION
                0x01 => return Ok(Op::MalformedOp),
                //SRL
                0x02 => {
                    return Ok(Op::AluOp {
                        op_type: AluOps::SRL,
                        mode: false,
                        val_src: AluOpSrc::Reg(r_op_rt.into()),
                        dst: r_op_rd.into(),
                        imm_src: None,
                        reg_rc: None,
                        shamt: Some(shamt::Imm(r_op_shamt)),
                    })
                }
                //SRA
                0x03 => {
                    return Ok(Op::AluOp {
                        op_type: AluOps::SRA,
                        mode: false,
                        val_src: AluOpSrc::Reg(r_op_rt.into()),
                        dst: r_op_rd.into(),
                        imm_src: None,
                        reg_rc: None,
                        shamt: Some(shamt::Imm(r_op_shamt)),
                    })
                }
                //SLLV
                0x04 => {
                    return Ok(Op::AluOp {
                        op_type: AluOps::SLLV,
                        mode: false,
                        val_src: AluOpSrc::Reg(r_op_rt.into()),
                        dst: r_op_rd.into(),
                        imm_src: None,
                        reg_rc: None,
                        shamt: Some(shamt::Variable(r_op_shamt.into())),
                    });
                }
                //RESERVED INSTRUCTION EXCEPTION
                0x05 => {}
                //SRLV
                0x06 => {}
                //SRAV
                0x07 => {}
                _ => unreachable!("decoded impossible operation in SPECIAL decoding"),
            }
        }
        //REGIMM decoding
        0x1 => {}

        _ => unreachable!("main lift match bad pattern"),
    }

    Err(LiftError::InvalidOpcodeError)
}
