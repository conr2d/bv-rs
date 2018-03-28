use super::storage::{BlockType, Address};
use num_traits::{Zero, One, ToPrimitive};

/// Read-only bit vector operations.
///
/// Minimal complete definition is `get_bit` or `get_block`, since each
/// is defined in terms of the other. Note that `get_block` in terms of
/// `get_bit` is inefficient, and thus you should implement `get_block`
/// directly if possible.
pub trait BitVec {
    /// The underlying block type used to store the bits of the vector.
    type Block: BlockType;

    /// The length of the slice in bits.
    fn bit_len(&self) -> u64;

    /// The number of bits into the first block that the bit vector starts.
    /// Must be less than `Block::nbits()`.
    fn bit_offset(&self) -> u8;

    /// The length of the slice in blocks.
    fn block_len(&self) -> usize {
        u64::ceil_div_nbits(self.bit_len() + self.bit_offset() as u64)
    }

    /// Gets the bit at `position`
    ///
    /// The default implementation calls `get_block` and masks out the
    /// correct bit.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    fn get_bit(&self, position: u64) -> bool {
        assert!(position < self.bit_len(), "BitVec::get_bit: out of bounds");

        let address = Address::new::<Self::Block>(position + self.bit_offset() as u64);
        let block = self.get_block(address.block_index);
        block.get_bit(address.bit_offset)
    }

    /// Gets the block at `position`
    ///
    /// The bits are laid out `Block::nbits()` per block, with the notional
    /// zeroth bit in the least significant position. If `self.bit_len()` is
    /// not a multiple of `Block::nbits()` then the last block will
    /// contain extra bits that are not part of the bit vector.
    ///
    /// The default implementation assembles a block by reading each of its
    /// bits. Consider it a slow reference implementation, and override it.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    fn get_block(&self, position: usize) -> Self::Block {
        assert!(position < self.block_len(),
                "BitVec::get_block: out of bounds");

        let bit_position = position as u64 * Self::Block::nbits() as u64;

        let mut result = Self::Block::zero();
        let mut mask = Self::Block::one();

        for i in 0 .. Self::Block::nbits() as u64 {
            if bit_position + i >= self.bit_offset() as u64
                && bit_position + i - (self.bit_offset() as u64) < self.bit_len()
                && self.get_bit(bit_position + i - self.bit_offset() as u64) {
                result = result | mask;
            }
            mask = mask << 1;
        }

        result
    }

    /// Gets `count` bits starting at bit index `start`, interpreted as a
    /// little-endian integer.
    ///
    /// # Panics
    ///
    /// Panics if the bit span goes out of bounds.
    fn get_bits(&self, start: u64, count: usize) -> Self::Block {
        let limit = start + count as u64;
        assert!(limit <= self.bit_len(), "BitVec::get_bits: out of bounds");

        let address = Address::new::<Self::Block>(start + self.bit_offset() as u64);
        let margin = Self::Block::nbits() - address.bit_offset;

        if margin >= count {
            let block = self.get_block(address.block_index);
            return block.get_bits(address.bit_offset, count)
        }

        let extra = count - margin;

        let block1 = self.get_block(address.block_index);
        let block2 = self.get_block(address.block_index + 1);

        let low_bits = block1.get_bits(address.bit_offset, margin);
        let high_bits = block2.get_bits(0, extra);

        (high_bits << margin) | low_bits
    }
}

/// Mutable bit vector operations that don’t affect the length.
///
/// Minimal complete definition is `set_bit` or `set_block`, since each
/// is defined in terms of the other. Note that `set_block` in terms of
/// `set_bit` is inefficient, and thus you should implement `set_block`
/// directly if possible.
pub trait BitVecMut: BitVec {
    /// Sets the bit at `position` to `value`.
    ///
    /// The default implementation uses `get_block` and `set_block`.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    fn set_bit(&mut self, position: u64, value: bool) {
        assert!(position < self.bit_len(), "BitVecMut::set_bit: out of bounds");

        let address = Address::new::<Self::Block>(position + self.bit_offset() as u64);
        let old_block = self.get_block(address.block_index);
        let new_block = old_block.with_bit(address.bit_offset, value);
        self.set_block(address.block_index, new_block);
    }

    /// Sets the block at `position` to `value`.
    ///
    /// The bits are laid out `Block::nbits()` per block, with the notional
    /// zeroth bit in the least significant position. If `self.bit_len()` is
    /// not a multiple of `Block::nbits()` then the last block will
    /// contain extra bits that are not part of the bit vector. Implementations
    /// of `set_block` should not change those trailing bits.
    ///
    /// The default implementation sets a block by setting each of its bits
    /// in turn. Consider it a slow reference implementation, and override it.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    fn set_block(&mut self, position: usize, mut value: Self::Block) {
        let start = if position == 0 && self.bit_offset() > 0 {
            value = value >> self.bit_offset() as usize;
            self.bit_offset() as u64
        } else {
            0
        };

        let limit = if position + 1 == self.block_len() {
            Self::Block::last_block_bits(self.bit_len() + self.bit_offset() as u64)
        } else {
            Self::Block::nbits()
        };

        let offset = Self::Block::mul_nbits(position);
        let bit_offset = self.bit_offset() as u64;

        for i in start .. limit as u64 {
            let bit = value & Self::Block::one() != Self::Block::zero();
            self.set_bit(offset + i - bit_offset, bit);
            value = value >> 1;
        }
    }

    /// Sets `count` bits starting at bit index `start`, interpreted as a
    /// little-endian integer.
    ///
    /// # Panics
    ///
    /// Panics if the bit span goes out of bounds.
    fn set_bits(&mut self, start: u64, count: usize, value: Self::Block) {
        let limit = start + count as u64;
        assert!(limit <= self.bit_len(), "BitVecMut::set_bits: out of bounds");

        let address = Address::new::<Self::Block>(start + self.bit_offset() as u64);
        let margin = Self::Block::nbits() - address.bit_offset;

        if margin >= count {
            let old_block = self.get_block(address.block_index);
            let new_block = old_block.with_bits(address.bit_offset, count, value);
            self.set_block(address.block_index, new_block);
            return;
        }

        let extra = count - margin;

        let old_block1 = self.get_block(address.block_index);
        let old_block2 = self.get_block(address.block_index + 1);

        let high_bits = value >> margin;

        let new_block1 = old_block1.with_bits(address.bit_offset,
                                              margin, value);
        let new_block2 = old_block2.with_bits(0, extra, high_bits);
        self.set_block(address.block_index, new_block1);
        self.set_block(address.block_index + 1, new_block2);
    }
}

/// Bit vector operations that change the length.
pub trait BitVecPush: BitVecMut {
    /// Adds the given bit to the end of the bit vector.
    fn push_bit(&mut self, value: bool);

    /// Removes and returns the last bit, if any.
    fn pop_bit(&mut self) -> Option<bool>;

    /// Pushes `value` 0 or more times until the size of the bit
    /// vector is block-aligned.
    fn align_block(&mut self, value: bool) {
        while Self::Block::mod_nbits(self.bit_len() + self.bit_offset() as u64) != 0 {
            self.push_bit(value);
        }
    }

    /// Pushes the given block onto the end of the bit vector.
    ///
    /// If the end of the bit vector is not currently block-aligned,
    /// it pads with 0s up to the next block before pushing.
    ///
    /// The default implementation pushes the block one bit at a time;
    /// override it with something more efficient.
    fn push_block(&mut self, mut value: Self::Block) {
        self.align_block(false);

        for _ in 0 .. Self::Block::nbits() {
            self.push_bit(value & Self::Block::one() != Self::Block::zero());
            value = value >> 1;
        }
    }
}

/// Types that support (re-)slicing by ranges.
pub trait BitSliceable<Range> {
    /// The type of the slice.
    type Slice;

    /// (Re-)slices the given object.
    fn slice(self, range: Range) -> Self::Slice;
}

impl<Block: BlockType> BitVec for [Block] {
    type Block = Block;

    #[inline]
    fn bit_len(&self) -> u64 {
        self.len() as u64 * Block::nbits() as u64
    }

    #[inline]
    fn bit_offset(&self) -> u8 {
        0
    }

    #[inline]
    fn block_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get_block(&self, position: usize) -> Block {
        self[position]
    }
}

impl<Block: BlockType> BitVecMut for [Block] {
    #[inline]
    fn set_block(&mut self, position: usize, value: Block) {
        self[position] = value;
    }
}

impl BitVec for Vec<bool> {
    type Block = u8; // This is bogus

    #[inline]
    fn bit_len(&self) -> u64 {
        self.len() as u64
    }

    #[inline]
    fn bit_offset(&self) -> u8 {
        0
    }

    fn get_bit(&self, position: u64) -> bool {
        self[position.to_usize().expect("Vec<bool>::get_bit: overflow")]
    }
}

impl BitVecMut for Vec<bool> {
    fn set_bit(&mut self, position: u64, value: bool) {
        let position = position.to_usize()
            .expect("Vec<bool>::set_bit: overflow");
        self[position] = value;
    }
}

impl BitVecPush for Vec<bool> {
    fn push_bit(&mut self, value: bool) {
        self.push(value);
    }

    fn pop_bit(&mut self) -> Option<bool> {
        self.pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn vec_u8_is_bit_vec() {
        let v = vec![0b01001000u8, 0b11100011u8];
        assert!( !v.get_bit(0) );
        assert!( !v.get_bit(1) );
        assert!( !v.get_bit(2) );
        assert!(  v.get_bit(3) );
        assert!( !v.get_bit(4) );
        assert!( !v.get_bit(5) );
        assert!(  v.get_bit(6) );
        assert!( !v.get_bit(7) );
        assert!(  v.get_bit(8) );
        assert!(  v.get_bit(9) );
        assert!( !v.get_bit(10) );
        assert!( !v.get_bit(11) );
        assert!( !v.get_bit(12) );
        assert!(  v.get_bit(13) );
        assert!(  v.get_bit(14) );
        assert!(  v.get_bit(15) );

        assert_eq!( v.get_bits(4, 8), 0b00110100u8 );
    }

    #[test]
    fn vec_u8_is_bit_vec_mut() {
        let mut v = vec![0b01001000u8, 0b11100011u8];
        assert!( !v.get_bit(0) );
        v.set_bit(0, true);
        assert!(  v.get_bit(0) );
        assert!( !v.get_bit(1) );
        v.set_bit(1, true);
        assert!(  v.get_bit(1) );
        assert!( !v.get_bit(10) );
        v.set_bit(10, true);
        assert!(  v.get_bit(10) );

        v.set_bits(4, 8, 0b11110000);

        assert!( !v.get_bit(4) );
        assert!( !v.get_bit(5) );
        assert!( !v.get_bit(6) );
        assert!( !v.get_bit(7) );
        assert!(  v.get_bit(8) );
        assert!(  v.get_bit(9) );
        assert!(  v.get_bit(10) );
        assert!(  v.get_bit(11) );
    }
}
