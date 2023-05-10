use crate::cart::Cart;
use crate::rdram::Rdram;
use proc_bitfield::bitfield;
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

//struct for the main VR4300 cpu
#[derive(Default)]
pub struct Cpu {
    pub rf: Rf,
    //needs cop0(cpu controll coprocessor) - NOTE: coprocessors are on the same die!!! they are not separate hardware!!
    pub cop0: cop0,
    //needs cop1 (fp coprocessor)

    //////////////////////////////////
    //Rcs to other pieces of the system that we can touch
    mem: Rc<RefCell<Rdram>>,
    cart: Rc<RefCell<Cart>>,
}

impl Cpu {
    pub fn new(mem: Rc<RefCell<Rdram>>, cart: Rc<RefCell<Cart>>) -> Self {
        Self {
            mem,
            cart,
            ..Default::default()
        }
    }
}

//main register file of the cpu
#[allow(non_snake_case)]
#[derive(Default)]
pub struct Rf {
    //gprs - 32, 64 bit regs (always reads as 64-bit)
    pub gprs: [u64; 32],
    //fprs - these are 64 bit IEEE754 compliant. should we use u64s or actual doubles
    //NOTE: youre gonna have to put some more thought into this i think
    //p. 208 in the manual. look at how you can access differently. might be okay to just use u32s and cast?
    pub fprs: [u64; 32],
    //PC
    pub PC: u64,
    //HI
    pub HI: u64,
    //LO
    pub LO: u64,
    //LLBit
    pub LLBit: bool,

    //NOTE: these are floating point control registers
    pub FCR0: u32,
    //FCR31 -- constrol/status reg //bitflags //chris, go figure out how to use kelpsys crate
    pub FCR31: FP_control_reg,
}

//lets let us index into rf's gprs either explicitly or via a gpr number (5 bit bitfield)
/*
zero => 0
at   => 1
v0   => 2
v1   => 3
a0   => 4
a1   => 5
a2   => 6
a3   => 7
t0   => 8
t1   => 9
t2 => 10
t3 => 11
t4 => 12
t5 => 13
t6 => 14
t7 => 15
s0 => 16
s1 => 17
s2 => 18
s3 => 19
s4 => 20
s5 => 21
s6 => 22
s7 => 23
t8 => 24
t9 => 25
k0 => 26
k1 => 27
gp => 28
sp => 29
fp => 30
ra => 31
*/

impl Index<GPR> for Rf {
    type Output = u64;
    fn index(&self, reg: GPR) -> &<Self as Index<GPR>>::Output {
        if reg == GPR::zero {
            return &0;
        } else {
            return &self.gprs[(reg as u8) as usize];
        }
    }
}
impl IndexMut<GPR> for Rf {
    //type Output = u64;
    fn index_mut(&mut self, reg: GPR) -> &mut <Self as Index<GPR>>::Output {
        // if reg == GPR::zero {
        //     return &mut 0;
        //} else {
        //TODO: is there some good way to not let us mutably access $zero?
        return &mut self.gprs[(reg as u8) as usize];
        // }
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
pub enum GPR {
    zero,
    at,
    v0,
    v1,
    a0,
    a1,
    a2,
    a3,
    t0,
    t1,
    t2,
    t3,
    t4,
    t5,
    t6,
    t7,
    s0,
    s1,
    s2,
    s3,
    s4,
    s5,
    s6,
    s7,
    t8,
    t9,
    k0,
    k1,
    gp,
    sp,
    fp,
    ra,
}
impl From<GPR> for u8 {
    fn from(val: GPR) -> Self {
        return match val {
            GPR::zero => 0,
            GPR::at => 1,
            GPR::v0 => 2,
            GPR::v1 => 3,
            GPR::a0 => 4,
            GPR::a1 => 5,
            GPR::a2 => 6,
            GPR::a3 => 7,
            GPR::t0 => 8,
            GPR::t1 => 9,
            GPR::t2 => 10,
            GPR::t3 => 11,
            GPR::t4 => 12,
            GPR::t5 => 13,
            GPR::t6 => 14,
            GPR::t7 => 15,
            GPR::s0 => 16,
            GPR::s1 => 17,
            GPR::s2 => 18,
            GPR::s3 => 19,
            GPR::s4 => 20,
            GPR::s5 => 21,
            GPR::s6 => 22,
            GPR::s7 => 23,
            GPR::t8 => 24,
            GPR::t9 => 25,
            GPR::k0 => 26,
            GPR::k1 => 27,
            GPR::gp => 28,
            GPR::sp => 29,
            GPR::fp => 30,
            GPR::ra => 31,
            _ => panic!("bad value"),
        };
    }
}
impl From<u8> for GPR {
    fn from(val: u8) -> Self {
        return match val {
            0 => GPR::zero,
            1 => GPR::at,
            2 => GPR::v0,
            3 => GPR::v1,
            4 => GPR::a0,
            5 => GPR::a1,
            6 => GPR::a2,
            7 => GPR::a3,
            8 => GPR::t0,
            9 => GPR::t1,
            10 => GPR::t2,
            11 => GPR::t3,
            12 => GPR::t4,
            13 => GPR::t5,
            14 => GPR::t6,
            15 => GPR::t7,
            16 => GPR::s0,
            17 => GPR::s1,
            18 => GPR::s2,
            19 => GPR::s3,
            20 => GPR::s4,
            21 => GPR::s5,
            22 => GPR::s6,
            23 => GPR::s7,
            24 => GPR::t8,
            25 => GPR::t9,
            26 => GPR::k0,
            27 => GPR::k1,
            28 => GPR::gp,
            29 => GPR::sp,
            30 => GPR::fp,
            31 => GPR::ra,
            _ => panic!("bad value"),
        };
    }
}

//in depth defs on page 146
/*
0 Index Programmable pointer into TLB array
1 Random Pseudorandom pointer into TLB array (read only)
2 EntryLo0 Low half of TLB entry for even virtual address (VPN)
3 EntryLo1 Low half of TLB entry for odd virtual address (VPN)
4 Context Pointer to kernel virtual page table entry (PTE) in 32-bit mode
5 PageMask Page size specification
6 Wired Number of wired TLB entries
7 — Reserved for future use
8 BadVAddr Display of virtual address that occurred an error last
9 Count Timer Count
10 EntryHi High half of TLB entry (including ASID)
11 Compare Timer Compare Value
12 Status Operation status setting
13 Cause Display of cause of last exception
14 EPC Exception Program Counter
15 PRId Processor Revision Identifier
16 Config Memory system mode setting
17 LLAddr Load Linked instruction address display
18 WatchLo Memory reference trap address low bits
19 WatchHi Memory reference trap address high bits
20 XContext Pointer to Kernel virtual PTE table in 64-bit mode
21–25 — Reserved for future use
26 Parity Error* Cache parity bits
27 Cache Error* Cache Error and Status register
28 TagLo Cache Tag register low
29 TagHi Cache Tag register high
30 ErrorEPC Error Exception Program Counter
31 — Reserved for future use
*/

enum cop0reg {
    Index,    //32 bit
    Random,   //32 bit
    EntryLo0, //64 bit (32 bit access sign extends)
    EntryLo1, //64 bit (32 bit access sign extends)
    Context,  //64 bit (32 bit sign access sign extends?)
    PageMask, //64 bit (32 bit access sign extends)
    Wired,    //32 bit
    //7 — Reserved for future use
    BadVAddr, //64 (32 ?)
    Count,    //32 bit
    EntryHi,  //64 bit (32 bit access sign extends)
    Compare,  //32 bit
    Status,   //32 bit NOTE: this is actually a bitfield
    Cause,    //32 bit NOTE: this is actually a bitfield
    EPC,      //64 (32?)
    PRId,     //32 NOTE: bitfield
    Config,   //32 NOTE: bitfield
    LLAddr,   //32
    WatchLo,  //32
    WatchHi,  //32
    XContext, //64 NOTE: bitfield
    //21–25 — Reserved for future use
    Parity, //32: bitfield
    Cache,  //32
    TagLo,  //32 bitfield
    TagHi,  //32 bitfield
    ErrorEPC, //64 (32?)
            //31 — Reserved for future use
}

//make this indexable by an enum of all the registers it contains. impl Index and IndexMut traits
#[derive(Default)]
pub struct cop0 {
    pub Index: u32,    //32 bit
    pub Random: u32,   //32 bit
    pub EntryLo0: u64, //64 bit (32 bit access sign extends)
    pub EntryLo1: u64, //64 bit (32 bit access sign extends)
    pub Context: u64,  //64 bit (32 bit sign access sign extends?)
    pub PageMask: u64, //64 bit (32 bit access sign extends)
    pub Wired: u32,    //32 bit
    //7 — Reserved for future use
    pub BadVAddr: u64,          //64 (32 ?)
    pub Count: u32,             //32 bit
    pub EntryHi: u64,           //64 bit (32 bit access sign extends)
    pub Compare: u32,           //32 bit
    pub Status: status_reg,     //32 bit NOTE: this is actually a bitfield
    pub Cause: cause_reg,       //32 bit NOTE: this is actually a bitfield
    pub EPC: u64,               //64 (32?)
    pub PRId: PRId_reg,         //32 NOTE: bitfield
    pub Config: config_reg,     //32 NOTE: bitfield
    pub LLAddr: u32,            //32
    pub WatchLo: u32,           //32
    pub WatchHi: u32,           //32
    pub XContext: XContext_reg, //64 NOTE: bitfield
    //21–25 — Reserved for future use
    //this reg is only here for VR4200 compat and we never use it. so no nice bitfield for it
    pub Parity: u32,      //32: bitfield
    pub Cache: u32,       //32
    pub TagLo: TagLo_reg, //32 bitfield
    //this is just always 0??
    pub TagHi: u32,    //32 bitfield
    pub ErrorEPC: u64, //64 (32?)*/
}

//status reg
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct status_reg(pub u32): Debug, FromRaw, IntoRaw, DerefRaw{
        pub CU: u8 @ 28..=31,
        pub RP: bool @ 27,
        pub FR: bool @ 26,
        pub RE: bool @ 25,
        //pub DS: DS @ 16..=24,
        //these fields are part of the sub bitfield SD (self-diagnostic)
        pub ITS: bool @ 24,
        //hardwired 0 23
        pub BEV: bool @22,
        pub TS: bool @ 21,
        pub SR: bool @ 20,
        //hardwired 0 19
        pub CH: bool @ 18,
        pub CE: bool @ 17,
        pub DE: bool @ 16,
        //end of sub-bitfield
        pub IM: u8 @ 8..=15,
        pub KX: bool @ 7,
        pub SX: bool @ 6,
        pub UX: bool @ 5,
        pub KSU: u8 @ 3..=4,
        pub ERL: bool @ 2,
        pub EXL: bool @ 1,
        pub IE: bool @ 0,
    }
}

//cause register
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct cause_reg(pub u32): Debug, FromRaw, IntoRaw, DerefRaw{
        pub BD: bool @ 31,
        //bit 30 is 0
        pub CE: u8 @ 28 ..= 29,
        //bits 26 ..= 27 are 0
        pub IP: u8 @ 8 ..= 15,
        //bit 7 is 0,
        pub ExcCode: u8 @ 2 ..= 6,
        //bit 0 and 1 are 0
    }
}

//PRId register
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct PRId_reg(pub u32): Debug, FromRaw, IntoRaw, DerefRaw{
        //upper 16 are zeroed
        pub Imp: u8 @ 8 ..= 15,
        pub Rev: u8 @ 0 ..= 7,

    }
}
//Config register
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct config_reg(pub u32): Debug, FromRaw, IntoRaw, DerefRaw{
        //bit 31 is 0
        pub EC: u8 @ 28 ..= 30,
        pub EP: u8 @ 24 ..= 27,
        //16..= 23 are set explicitly to the pattern "00000110"
        pub BE: bool @ 15,
        //4..= 14 are set eplicitly to the pattern "11001000110"
        pub CU: bool @ 3,
        pub K0: u8 @ 0..=2
    }
}

//this reg might be important later.
//XContext register
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct XContext_reg(pub u64): Debug, FromRaw, IntoRaw, DerefRaw{
        pub PTEBase: u32 @ 33 ..= 63,
        pub R: u8 @ 31 ..= 32,
        pub BadVPN2: u32 @ 4..= 30,
        //0..=3 are 0
    }
}

//TagLo register
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct TagLo_reg(pub u64): Debug, FromRaw, IntoRaw, DerefRaw{
        //28 ..= 31 are 0
        pub PTagLo: u32 @ 8 ..= 27,
        pub PState: u8 @ 6..= 7,
        //0..=5 are 0

    }
}

//TagHi register
//all 0s all the time??

//this controls our fp modes and assoc shit
bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    #[allow(non_snake_case)]
    pub struct FP_control_reg(pub u64): Debug, FromRaw, IntoRaw, DerefRaw{
        //25 ..= 31
        pub FS: bool @ 24,
        pub C: bool @ 23,
        //18..=22 are 0
        pub Cause: u8 @ 12 ..= 17,
        pub Enables: u8 @ 7 ..= 11,
        pub Flags: u8 @ 2 ..= 6,
        pub RM: u8 @ 0 ..= 1
    }
}
