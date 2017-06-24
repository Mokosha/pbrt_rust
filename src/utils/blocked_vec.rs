use std::default::Default;

const LOG_BLOCK_SIZE: usize = 5;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BlockedVec<T: Default + Clone> {
    blocks_memory: Vec<T>,
    log_block_size: usize,
    blocks_wide: usize,
    width: usize,
    height: usize
}

impl<T: Default + Clone> BlockedVec<T> {
    pub fn new(width: usize, height: usize) -> BlockedVec<T> {
        let block_size = 1 << LOG_BLOCK_SIZE;
        let round_up = |x: usize| { (x + block_size - 1) & !(block_size - 1) };

        let num_elements = round_up(width) * round_up(height);
        let mem: Vec<T> = vec![Default::default(); num_elements];

        BlockedVec {
            blocks_memory: mem,
            log_block_size: LOG_BLOCK_SIZE,
            blocks_wide: round_up(width) >> LOG_BLOCK_SIZE,
            width: width,
            height: height
        }
    }

    pub fn new_with(width: usize, height: usize, vals: Vec<T>) -> BlockedVec<T> {
        assert_eq!(width * height, vals.len());
        let mut blocked_vec = BlockedVec::new(width, height);
        for y in 0..height {
            for x in 0..width {
                *(blocked_vec.get_mut(x, y).unwrap()) =
                    vals[y * width + x].clone();
            }
        }

        blocked_vec
    }


    fn block_size(&self) -> usize {
        1 << self.log_block_size
    }
    
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    fn block(&self, x: usize) -> usize { x >> self.log_block_size }
    fn offset(&self, x: usize) -> usize { x & (self.block_size() - 1) }

    fn get_offset(&self, u: usize, v: usize) -> usize {
        let bu = self.block(u);
        let bv = self.block(v);

        let ou = self.offset(u);
        let ov = self.offset(v);

        let pixels_per_block = self.block_size() * self.block_size();

        // Offset to block
        pixels_per_block * (self.blocks_wide * bv + bu) +
            // Offset within block
            self.block_size() * ov + ou
    }

    fn get(&self, u: usize, v: usize) -> Option<&T> {
        if u >= self.width || v >= self.height {
            return None;
        }

        let offset = self.get_offset(u, v);
        Some(&self.blocks_memory[offset])
    }

    fn get_mut(&mut self, u: usize, v: usize) -> Option<&mut T> {
        if u >= self.width || v >= self.height {
            return None;
        }
        let offset = self.get_offset(u, v);
        Some(&mut self.blocks_memory[offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_be_created() {
        let bvec: BlockedVec<f32> = BlockedVec::new(57, 92);

        assert_eq!(bvec.width(), 57);
        assert_eq!(bvec.height(), 92);
    }

    #[test]
    fn it_can_hold_a_two_dimensional_array() {
        let width = 132;
        let height = 256;
        let mut bvec: BlockedVec<u8> = BlockedVec::new(width, height);

        for y in 0..height {
            for x in 0..width {
                *(bvec.get_mut(x, y).unwrap()) = y as u8;
            }
        }
        
        for y in 0..height {
            for x in 0..width {
                assert_eq!(*(bvec.get(x, y).unwrap()), y as u8);
            }
        }
    }

    #[test]
    fn it_can_be_initialized_with_existing_memory() {
        let width = 23;
        let height = 14;
        let mut vec: Vec<u8> = Vec::new();

        for y in 0..height {
            for x in 0..width {
                vec.push((x & y) as u8);
            }
        }

        let bvec = BlockedVec::new_with(width, height, vec);
        for y in 0..height {
            for x in 0..width {
                assert_eq!(*(bvec.get(x, y).unwrap()), (x & y) as u8);
            }
        }
    }

    #[test]
    fn it_recognized_out_of_bounds() {
        let width = 54;
        let height = 29;
        let bvec: BlockedVec<u8> = BlockedVec::new(width, height);

        assert_eq!(bvec.get(55, 28), None);
        assert_eq!(bvec.get(54, 28), None);
        assert_eq!(bvec.get(53, 29), None);
        assert_eq!(bvec.get(200, 300), None);
        assert!(bvec.get(0, 0).is_some());
        assert!(bvec.get(53, 28).is_some());
    }
    
}
