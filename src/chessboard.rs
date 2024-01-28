#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::ops::{BitAnd, BitOr, BitXor, Not};

pub struct BitBoard {
    pub data: u64,
}

pub type BB = BitBoard;

impl BitAnd for BitBoard {
    type Output = BitBoard;
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard {
            data: self.data & rhs.data,
        }
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;
    fn bitor(self, rhs: BitBoard) -> Self::Output {
        BitBoard {
            data: self.data | rhs.data,
        }
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;
    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        BitBoard {
            data: self.data ^ rhs.data,
        }
    }
}

impl Not for BitBoard {
    type Output = BitBoard;
    fn not(self) -> Self::Output {
        BitBoard { data: !self.data }
    }
}

/*
indexing the 64-squares:
   -----------------------
8 |63 62 61 60 59 58 57 56|
7 |55 54 53 52 51 50 49 48|
6 |47 46 45 44 43 42 41 40|
5 |39 38 37 36 35 34 33 32|
4 |31 30 29 28 27 26 25 24|
3 |23 22 21 20 19 18 17 16|
2 |15 14 13 12 11 10  9  8|
1 | 7  6  5  4  3  2  1  0|
   -----------------------
    A  B  C  D  E  F  G  H

 */
impl BitBoard {
    pub const ZERO: BB = BB { data: 0u64 };
    pub const ONES: BB = BB { data: u64::MAX };
    pub fn nth(n: usize) -> Self {
        Self { data: 1u64 << n }
    }

    pub const W_PAWN_ATTACKS: [BB; 64] = BB::init_pawn_attacks(Side::White);
    pub const B_PAWN_ATTACKS: [BB; 64] = BB::init_pawn_attacks(Side::Black);
    pub const KNIGHT_ATTACKS: [BB; 64] = BB::init_knight_attacks();
    pub const BISHOP_ATTACKS: [BB; 64] = BB::init_bishop_attacks();
    pub const ROOK_ATTACKS: [BB; 64] = BB::init_rook_attacks();
    pub const QUEEN_ATTACKS: [BB; 64] = BB::init_queen_attacks();
    pub const KING_ATTACKS: [BB; 64] = BB::init_king_attacks();

    const fn init_pawn_attacks(side: Side) -> [BB; 64] {
        let mut i: usize = 0;
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];
        while i < 64usize {
            let mut data: u64 = 0u64;
            match side {
                Side::White => {
                    if i < 56 {
                        if i % 8 > 0 {
                            data |= (1u64 << i) << 7
                        }
                        if i % 8 < 7 {
                            data |= (1u64 << i) << 9
                        }
                    }
                }
                Side::Black => {
                    if i > 7 {
                        if i % 8 > 0 {
                            data |= (1u64 << i) >> 9
                        }
                        if i % 8 < 7 {
                            data |= (1u64 << i) >> 7
                        }
                    }
                }
                Side::Neither => unreachable!(),
            }
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }

    const fn init_knight_attacks() -> [BB; 64] {
        let mut i: usize = 0;
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];
        while i < 64usize {
            let mut data: u64 = 0u64;
            if i < 48 {
                if i % 8 < 7 {
                    //up left << 17
                    data |= (1u64 << i) << 17
                }
                if i % 8 > 0 {
                    //up right << 15
                    data |= (1u64 << i) << 15
                }
            }
            if i < 56 {
                if i % 8 < 6 {
                    //left up << 10
                    data |= (1u64 << i) << 10
                }
                if i % 8 > 1 {
                    //right up <<  6
                    data |= (1u64 << i) << 6
                }
            }
            if i > 7 {
                if i % 8 < 6 {
                    //left down >> 6
                    data |= (1u64 << i) >> 6
                }
                if i % 8 > 1 {
                    //right down >> 10
                    data |= (1u64 << i) >> 10
                }
            }
            if i > 15 {
                if i % 8 < 7 {
                    //down left >> 15
                    data |= (1u64 << i) >> 15
                }
                if i % 8 > 0 {
                    //down right >> 17
                    data |= (1u64 << i) >> 17
                }
            }
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }

    // use standard techniques for now
    const fn init_bishop_attacks_mbb() -> [BB; 64] {
        !unimplemented!()
    }
    const fn init_rook_attacks_mbb() -> [BB; 64] {
        !unimplemented!()
    }
    const fn init_queen_attacks_mbb() -> [BB; 64] {
        !unimplemented!()
    }

    // todo: exact hashing function
    const fn init_bishop_attacks() -> [BB; 64] {
        !unimplemented!()
    }
    const fn init_rook_attacks() -> [BB; 64] {
        !unimplemented!()
    }
    const fn init_queen_attacks() -> [BB; 64] {
        !unimplemented!()
    }

    const fn init_king_attacks() -> [BB; 64] {
        let mut i: usize = 0;
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];
        while i < 64usize {
            let mut data: u64 = 0u64;
            if i < 56 {
                //up
                data |= (1u64 << i) << 8
            }
            if i > 7 {
                //down
                data |= (1u64 << i) >> 8
            }
            if i % 8 < 7 {
                //left
                data |= (1u64 << i) << 1
            }
            if i % 8 > 0 {
                //right
                data |= (1u64 << i) >> 1
            }
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
    Neither,
}
