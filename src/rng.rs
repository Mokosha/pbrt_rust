pub trait RNG {
}

pub struct PseudoRNG;
impl RNG for PseudoRNG { }
impl PseudoRNG {
    pub fn new(task_idx: i32) -> PseudoRNG { PseudoRNG }
}
