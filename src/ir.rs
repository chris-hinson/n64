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
    /*match instr {
        0x0000_0000 =>

    }*/

    Err(LiftError::InvalidOpcodeError)
}
