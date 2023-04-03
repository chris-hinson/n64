pub struct Rdram {
    mem: [u8; 4608],
}

/*impl Rdram {
    pub fn new() -> Self {
        Rdram { mem: [0; 4608] }
    }
}*/

impl Default for Rdram {
    fn default() -> Self {
        Rdram { mem: [0; 4608] }
    }
}
