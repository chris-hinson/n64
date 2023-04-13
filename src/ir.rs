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

enum Op {
    Load {
        width: usize,
        dest: GPR,
        base: GPR,
        offset: u16,
        condtional: bool,
    },
    Store {
        widt: usize,
        src: GPR,
        base: GPR,
        offset: u16,
        conditional: bool,
    },

    //computational instructions that use the alu
    AluOp {
        op_type: AluOps,
        val_src: AluOpSrc,
        dst: GPR,
        imm_src: Option<u16>,
        reg_rc: Option<GPR>,
        shamt: Option<u8>,
    },

    //TODO: how the FUCK do we want to handle the coprocessor conditionals??
    ControlFlow {
        conditional: ControlConditionalType,
        //NOTE: we say destination and not offset because even for branch co
        destination: ControlDestType,
        register: Option<GPR>,
        //fuck you mips. need this field so we know if we are dealing with delay slots or not
        likely: bool,
        link: bool,
    },

    //these are really just cop0 instructions
    SystemControl {},
}

enum AluOps {
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
enum AluOpSrc {
    Imm,
    Reg,
}

//enum ControlFlowType
enum ControlConditionalType {
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

enum ControlDestType {
    Absolute { dest: usize },
    Relative { offset: i64 },
}
