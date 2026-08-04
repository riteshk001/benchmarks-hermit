// fastrand.rs — PRNG 
// Usage : un XorShiftRng per core, created once at the booting.
//   let mut rng = XorShiftRng::new_for_core(core_id);
//   let x = rng.next_u32();          // nb pseudo random 
//   let x = rng.gen_range(total);    // nb in [0, total) 

pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    // Générator auto-seed, per core.

    pub fn new_for_core(core_id: usize) -> Self {
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        let mut seed = tsc ^ ((core_id as u64) << 32) ^ (core_id as u64);
        if seed == 0 {
            seed = 0xdeadbeef; // xorshift cannot start at 0
        }
        Self { state: seed }
    }

    
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 32) as u32
    }

   
    /// to replace `rand::random::<u32>() % total`.
    #[inline]
    pub fn gen_range(&mut self, bound: u32) -> u32 {
        if bound == 0 { return 0; }
        self.next_u32() % bound
    }
}