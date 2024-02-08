#![allow(dead_code)]
#![allow(long_running_const_eval)]
use rand::Rng;
use std::ops::{BitAnd, BitOr, BitXor, Not};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BitBoard {
    pub data: u64,
}

pub type BB = BitBoard;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceT {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
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

/* indexing the 64-squares:
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
    A  B  C  D  E  F  G  H */

/* some u64 bit manipulation support */

impl BitBoard {
    /* ---- bitboard constants ---- */
    pub const ZERO: BB = BB { data: 0u64 };
    pub const ONES: BB = BB { data: u64::MAX };
    pub fn nth(n: usize) -> Self {
        Self { data: 1u64 << n }
    }

    /* ---- bitboard bit manipulation ---- */
    pub const fn lsb_index(self) -> Option<usize> {
        if self.data == 0u64 {
            return None;
        } else {
            return std::option::Option::Some(self.data.trailing_zeros() as usize);
        }
    }
    pub fn set_bit(&mut self, i: usize) {
        self.data = self.data | 1u64 << i;
    }
    pub const fn get_bit(self, i: usize) -> BB {
        BB {
            data: self.data & 1u64 << i,
        }
    }
    pub const fn pop_bit(self, i: usize) -> BB {
        BB {
            data: match self.get_bit(i).data {
                0u64 => 0,
                _ => self.data ^ 1u64 << i,
            },
        }
    }

    /* ---- hard coded attack patterns ---- */
    pub const W_PAWN_ATTACKS: [BB; 64] = BB::init_pawn_attack(Side::White);
    pub const B_PAWN_ATTACKS: [BB; 64] = BB::init_pawn_attack(Side::Black);
    pub const KNIGHT_ATTACKS: [BB; 64] = BB::init_knight_attack();
    pub const BISHOP_ATTACKS_MBB: [[BB; 512]; 64] = BB::init_bishop_attack_mbb();
    pub const ROOK_ATTACKS_MBB: [[BB; 4096]; 64] = BB::init_rook_attack_mbb();

    /* ---- hard coded magic bitboard info ---- */
    pub const ROOK_OCC_BITCOUNT: [usize; 64] = [
        12, 11, 11, 11, 11, 11, 11, 12, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        11, 10, 10, 10, 10, 10, 10, 11, //
        12, 11, 11, 11, 11, 11, 11, 12, //
    ];
    pub const BISHOP_OCC_BITCOUNT: [usize; 64] = [
        6, 5, 5, 5, 5, 5, 5, 6, //
        5, 5, 5, 5, 5, 5, 5, 5, //
        5, 5, 7, 7, 7, 7, 5, 5, //
        5, 5, 7, 9, 9, 7, 5, 5, //
        5, 5, 7, 9, 9, 7, 5, 5, //
        5, 5, 7, 7, 7, 7, 5, 5, //
        5, 5, 5, 5, 5, 5, 5, 5, //
        6, 5, 5, 5, 5, 5, 5, 6, //
    ];
    pub const BISHOP_MBB_MASK: [BB; 64] = BB::init_bishop_mbb_mask();
    pub const ROOK_MBB_MASK: [BB; 64] = BB::init_rook_mbb_mask();
    pub const BISHOP_MAGICS: [u64; 64] = [
        0x800000000400,
        0x1000005200000800,
        0x4000002000009015,
        0x100000800180000,
        0x4020000240100DB6,
        0x4000000000881080,
        0xA800000000400008,
        0x8044000000100008,
        0x3000000000040000,
        0x12000C000400003,
        0x1089000020000010,
        0x1000000014C000,
        0x380600003001020,
        0x900004000800800,
        0x2000400020000000,
        0x2000000402000,
        0x880200000000014,
        0x4140110000A00010,
        0x28020020000,
        0x200018000080020,
        0x40088000000202,
        0xC80000000000080,
        0xE300000482600002,
        0x8300009018001010,
        0x10054,
        0x210000A000000000,
        0x2000000000,
        0x210140004440008,
        0x8200840004802000,
        0x400020002014,
        0x90300000000000,
        0x401150000001,
        0x2004,
        0x1400000040003001,
        0x408100008000008,
        0x102020280880080,
        0x8210040040064,
        0x40000000000020,
        0x800000800000020,
        0x10088000000000,
        0x930C0000008008C,
        0x200400040144004,
        0x8000000011000D,
        0x8200000000001006,
        0x4010800020000000,
        0x1100000100008000,
        0x8002011000010001,
        0x40100000140000,
        0x401800200,
        0x2080000020280204,
        0x1000082000000010,
        0x201000002480,
        0x8004320400000244,
        0x4000024420808,
        0x404040440100090,
        0x8800008012000030,
        0x100006806180,
        0x1400000820000E0,
        0x6002000802000,
        0x1214100820010000,
        0x1000210104000001,
        0x4000080300000000,
        0x4080820000050000,
        0x9202020002000401,
    ];
    pub const ROOK_MAGICS: [u64; 64] = [
        0x80008811204002,
        0x840006001500840,
        0x80281000200080,
        0x1300081002142100,
        0x801E0400800800,
        0x880010C00800200,
        0x6003408008A0003,
        0x200088124010042,
        0x1008004400098A0,
        0x1104000D0002001,
        0x19002004403500,
        0x8800800800801000,
        0x15001800050010,
        0xC06001C02000890,
        0x4009802011014,
        0x101000218804100,
        0x8080002A4000,
        0x1010004010402000,
        0xA08808020041000,
        0x3010848008001001,
        0x321808064002800,
        0x984808002000400,
        0x18040008210A10,
        0x40220000408421,
        0x4007400080002182,
        0x41008100400021,
        0x88104100200100,
        0xC1000900201004,
        0x40200801002D0010,
        0x600120080440080,
        0x210D04C0001081A,
        0x4801408200004401,
        0x800420C001800789,
        0x20005000400020,
        0x2440200191004100,
        0x8900086101001004,
        0x1411050011000800,
        0x8201044008011020,
        0x5081020804000150,
        0x1940A000941,
        0xC0108020488003,
        0x8A1000C020024002,
        0x800406001010030,
        0x1A8081001030020,
        0x4001040008008080,
        0xB2000804A20010,
        0x800852830040002,
        0x280900108041002A,
        0x23800840610300,
        0x14081A2010200,
        0x200040110100,
        0x31002088100100,
        0x904080100100500,
        0x401020080440080,
        0x84048810D2010400,
        0x2200404410890200,
        0x228001021041,
        0x230040003881,
        0x1002010A8020C012,
        0x6009002090000A05,
        0x85004800029005,
        0x4022000804500162,
        0x204108008A100124,
        0x21020C027040182,
    ];

    /* ---- future project (space-optimal hashing function) ---- */
    //pub const BISHOP_ATTACKS: [BB; 64] = BB::init_bishop_attack();
    //pub const ROOK_ATTACKS: [BB; 64] = BB::init_rook_attack();
    //pub const QUEEN_ATTACKS: [BB; 64] = BB::init_queen_attack();
    //pub const KING_ATTACKS: [BB; 64] = BB::init_king_attack();

    /* ---- functions for attack pattern ---- */
    const fn init_pawn_attack(side: Side) -> [BB; 64] {
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
    const fn init_knight_attack() -> [BB; 64] {
        let mut i: usize = 0;
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];
        while i < 64usize {
            let mut data: u64 = 0u64;
            if i < 48 {
                if i % 8 < 7 {
                    //up left is "<< 17"
                    data |= (1u64 << i) << 17
                }
                if i % 8 > 0 {
                    //up right is "<< 15"
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
    const fn init_king_attack() -> [BB; 64] {
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

    /* ---- magic bitboard related functions ---- */
    pub const fn bishop_attack_mbb(i: usize, blocker: BB) -> BB {
        let m = BB::magic_index(BB::BISHOP_MAGICS[i], blocker, BB::BISHOP_OCC_BITCOUNT[i]);
        BB::BISHOP_ATTACKS_MBB[i][m]
    }
    pub const fn rook_attack_mbb(i: usize, blocker: BB) -> BB {
        let m = BB::magic_index(BB::ROOK_MAGICS[i], blocker, BB::ROOK_OCC_BITCOUNT[i]);
        BB::ROOK_ATTACKS_MBB[i][m]
    }

    const SIZE_BISHOP: usize = 1 << 9;
    const SIZE_ROOK: usize = 1 << 12;
    const fn init_bishop_attack_mbb() -> [[BB; BB::SIZE_BISHOP]; 64] {
        let mut attacks = [[BB::ZERO; BB::SIZE_BISHOP]; 64];
        let mut i: usize = 0;
        let max_index: usize = BB::SIZE_BISHOP;
        while i < 64 {
            let mut blocker_bbs = [BB::ZERO; BB::SIZE_BISHOP];
            let mut attacks_bbs = [BB::ZERO; BB::SIZE_BISHOP];
            let mut attack_history = [BB::ZERO; BB::SIZE_BISHOP];
            let mut j: usize = 0;
            let attack_mask: BB = BB::BISHOP_MBB_MASK[i];
            let bitcount: usize = BB::BISHOP_OCC_BITCOUNT[i];
            // initiate relevant blocker bitboards and attack bitboards
            while j < max_index {
                blocker_bbs[j] = BB::compute_occ_bb(j, bitcount, attack_mask);
                attacks_bbs[j] = BB::naive_bishop_attack(i, blocker_bbs[j]);
                j += 1;
            }
            // loop through every magic index
            let mut j: usize = 0;
            while j < max_index {
                let m = BB::magic_index(BB::BISHOP_MAGICS[i], blocker_bbs[j], bitcount);
                if attack_history[m].data == BB::ZERO.data {
                    // if uninitialized, record attack pattern
                    attack_history[m] = attacks_bbs[j];
                } else if attack_history[m].data != attacks_bbs[m].data {
                    // panic if colliding attack patterns do not match
                    panic!("bishop magic number error: invalid collision!");
                }
                j += 1;
            }
            attacks[i] = attack_history;
            i += 1
        }
        return attacks;
    }

    const fn init_rook_attack_mbb() -> [[BB; BB::SIZE_ROOK]; 64] {
        let mut attacks = [[BB::ZERO; BB::SIZE_ROOK]; 64];
        let mut i: usize = 0;
        let max_index: usize = BB::SIZE_ROOK;
        while i < 64 {
            let mut blocker_bbs = [BB::ZERO; BB::SIZE_ROOK];
            let mut attacks_bbs = [BB::ZERO; BB::SIZE_ROOK];
            let mut attack_history = [BB::ZERO; BB::SIZE_ROOK];
            let mut j: usize = 0;
            let attack_mask: BB = BB::ROOK_MBB_MASK[i];
            let bitcount: usize = BB::ROOK_OCC_BITCOUNT[i];
            // initiate relevant blocker bitboards and attack bitboards
            while j < max_index {
                blocker_bbs[j] = BB::compute_occ_bb(j, bitcount, attack_mask);
                attacks_bbs[j] = BB::naive_bishop_attack(i, blocker_bbs[j]);
                j += 1;
            }
            // loop through every magic index
            let mut j = 0;
            while j < max_index {
                let m = BB::magic_index(BB::ROOK_MAGICS[i], blocker_bbs[j], bitcount);
                if attack_history[m].data == BB::ZERO.data {
                    // if uninitialized, record attack pattern
                    attack_history[m] = attacks_bbs[j];
                } else if attack_history[m].data != attacks_bbs[m].data {
                    // panic if colliding attack patterns do not match
                    panic!("rook magic number error: invalid collision!");
                }
                j += 1;
            }
            attacks[i] = attack_history;
            i += 1
        }
        return attacks;
    }
    pub const fn magic_index(magic_num: u64, blocker: BB, bitcount: usize) -> usize {
        ((blocker.data.wrapping_mul(magic_num)) >> (64 - bitcount)) as usize
    }

    // naively implemented functions for debugging mbb
    pub const fn naive_bishop_attack(i: usize, blocker: BB) -> BB {
        let i_rank: isize = (i as isize) / 8isize;
        let i_file: isize = (i as isize) % 8isize;
        let mut j: isize = 0;
        let mut data: u64 = 0u64;
        let mut ul_is_blocked: bool = false;
        let mut dl_is_blocked: bool = false;
        let mut ur_is_blocked: bool = false;
        let mut dr_is_blocked: bool = false;
        while j <= 7 {
            //    up left direction: (+,+)
            if i_rank + j <= 7 && i_file + j <= 7 {
                if !ul_is_blocked {
                    data |= 1u64 << (i_rank + j) * 8 + (i_file + j);
                    if i_rank + j < 7 && i_file + j < 7 {
                        if 1u64 << (i_rank + j) * 8 + (i_file + j) & blocker.data != BB::ZERO.data {
                            ul_is_blocked = true;
                        }
                    }
                }
            }

            //  down left direction: (-,+)
            if 0 <= i_rank - j && i_file + j <= 7 {
                if !dl_is_blocked {
                    data |= 1u64 << (i_rank - j) * 8 + (i_file + j);
                    if 0 < i_rank - j && i_file + j < 7 {
                        if 1u64 << (i_rank - j) * 8 + (i_file + j) & blocker.data != BB::ZERO.data {
                            dl_is_blocked = true;
                        }
                    }
                }
            }

            //    up right direction: (+,-)
            if i_rank + j <= 7 && 0 <= i_file - j {
                if !ur_is_blocked {
                    data |= 1u64 << (i_rank + j) * 8 + (i_file - j);
                    if i_rank + j < 7 && 0 < i_file - j {
                        if 1u64 << (i_rank + j) * 8 + (i_file - j) & blocker.data != BB::ZERO.data {
                            ur_is_blocked = true;
                        }
                    }
                }
            }

            //  down right direction: (-,-)
            if 0 <= i_rank - j && 0 <= i_file - j {
                if !dr_is_blocked {
                    data |= 1u64 << (i_rank - j) * 8 + (i_file - j);
                    if 0 < i_rank - j && 0 < i_file - j {
                        if 1u64 << (i_rank - j) * 8 + (i_file - j) & blocker.data != BB::ZERO.data {
                            dr_is_blocked = true;
                        }
                    }
                }
            }
            j += 1
        }

        BB { data }
    }

    // naively implemented functions for debugging mbb
    pub const fn naive_rook_attack(i: usize, blocker: BB) -> BB {
        let i_rank: isize = (i as isize) / 8isize;
        let i_file: isize = (i as isize) % 8isize;
        let mut j: isize = 0;
        let mut data: u64 = 0u64;
        let mut r_is_blocked: bool = false;
        let mut l_is_blocked: bool = false;
        let mut u_is_blocked: bool = false;
        let mut d_is_blocked: bool = false;
        while j < 8 {
            // right direction (file --)
            if 0 <= i_file - j && !r_is_blocked {
                data |= 1u64 << i_rank * 8 + (i_file - j);
                r_is_blocked = r_is_blocked
                    || (1u64 << i_rank * 8 + (i_file - j) & blocker.data != BB::ZERO.data);
            }
            // left direction (file ++)
            if i_file + j <= 7 && !l_is_blocked {
                data |= 1u64 << i_rank * 8 + (i_file + j);
                l_is_blocked = l_is_blocked
                    || (1u64 << i_rank * 8 + (i_file + j) & blocker.data != BB::ZERO.data);
            }
            //   up direction (rank ++)
            if i_rank + j <= 7 && !u_is_blocked {
                data |= 1u64 << (i_rank + j) * 8 + i_file;
                u_is_blocked = u_is_blocked
                    || (1u64 << (i_rank + j) * 8 + i_file & blocker.data != BB::ZERO.data);
            }
            // down direction (rank --)
            if 0 <= i_rank - j && !d_is_blocked {
                data |= 1u64 << (i_rank - j) * 8 + i_file;
                d_is_blocked = d_is_blocked
                    || (1u64 << (i_rank - j) * 8 + i_file & blocker.data != BB::ZERO.data);
            }

            j += 1
        }
        BB { data }
    }

    // each bitboard flags relevant squares to a bishop in any given location on the chessboard
    const fn init_bishop_mbb_mask() -> [BB; 64] {
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];

        let mut i: usize = 0;
        while i < 64usize {
            let i_rank: isize = (i as isize) / 8isize;
            let i_file: isize = (i as isize) % 8isize;
            let mut j: isize = 1;
            let mut data: u64 = 0u64;
            while j < 7 {
                //    up left direction: (+,+)
                if i_rank + j < 7 && i_file + j < 7 {
                    data |= 1u64 << (i_rank + j) * 8 + (i_file + j);
                }
                //  down left direction: (-,+)
                if 0 < i_rank - j && i_file + j < 7 {
                    data |= 1u64 << (i_rank - j) * 8 + (i_file + j);
                }
                //    up right direction: (+,-)
                if i_rank + j < 7 && 0 < i_file - j {
                    data |= 1u64 << (i_rank + j) * 8 + (i_file - j);
                }
                //    up right direction: (-,-)
                if 0 < i_rank - j && 0 < i_file - j {
                    data |= 1u64 << (i_rank - j) * 8 + (i_file - j);
                }
                j += 1
            }
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }

    // each bitboard flags relevant squares to a rook in any given location on the chessboard
    const fn init_rook_mbb_mask() -> [BB; 64] {
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];

        let mut i: usize = 0;
        while i < 64usize {
            let i_rank: isize = (i as isize) / 8isize;
            let i_file: isize = (i as isize) % 8isize;
            let mut j: isize = 1;
            let mut data: u64 = 0u64;
            while j < 7 {
                // right direction (file --)
                if 0 < i_file - j {
                    data |= 1u64 << i_rank * 8 + (i_file - j);
                }
                // left direction (file ++)
                if i_file + j < 7 {
                    data |= 1u64 << i_rank * 8 + (i_file + j);
                }
                //   up direction (rank ++)
                if i_rank + j < 7 {
                    data |= 1u64 << (i_rank + j) * 8 + i_file;
                }
                // down direction (rank --)
                if 0 < i_rank - j {
                    data |= 1u64 << (i_rank - j) * 8 + i_file;
                }
                j += 1
            }
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }

    // these functions probably would never be used ever again, if magic numbers are hardcoded
    fn init_magic_numbers(piece_type: PieceT) -> [u64; 64] {
        let mut i: usize = 0;
        let mut magic_nums: [u64; 64] = [0u64; 64];
        match piece_type {
            PieceT::Bishop => {
                println!("Finding magic numbers for: Bishop")
            }
            PieceT::Rook => {
                println!("Finding magic numbers for: Rook")
            }
            _ => panic!("error: invalid piece_type parameter"),
        }
        while i < 64 {
            println!("calculating nth magic number: {}", i);
            magic_nums[i] = match piece_type {
                PieceT::Bishop => BB::find_magic_number(i, BB::BISHOP_OCC_BITCOUNT[i], piece_type),
                PieceT::Rook => BB::find_magic_number(i, BB::ROOK_OCC_BITCOUNT[i], piece_type),
                _ => panic!("init_magic_numbers error: invalid PieceT variable"),
            };
            i += 1;
        }
        return magic_nums;
    }

    // note: duplicated code in each bishop and rook branch, not optimal!
    fn find_magic_number(square: usize, mask_bitcount: usize, piece_type: PieceT) -> u64 {
        assert!(square < 64 && (piece_type == PieceT::Bishop || piece_type == PieceT::Rook));
        let mut rng = rand::thread_rng();
        match piece_type {
            PieceT::Bishop => {
                const SIZE: usize = 1 << 9;

                let mut blocker_bbs: [BB; SIZE] = [BB::ZERO; SIZE];
                let mut attacks_bbs: [BB; SIZE] = [BB::ZERO; SIZE];

                let attack_mask: BB = BB::BISHOP_MBB_MASK[square];
                let max_index: usize = 1usize << mask_bitcount;
                let mut i: usize = 0;

                while i < max_index {
                    blocker_bbs[i] = BB::compute_occ_bb(i, mask_bitcount, attack_mask);
                    attacks_bbs[i] = BB::naive_bishop_attack(square, blocker_bbs[i]);
                    i += 1;
                }
                let mut _attempts: usize = 0;

                // brute force loop
                while _attempts < usize::MAX {
                    let magic_num: u64 = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();

                    // skip impossible magic_num candidates
                    if (attack_mask.data.wrapping_mul(magic_num)) & 0xFF00000000000000 < 6 {
                        continue; // note: idk why this is the condition
                    }

                    let mut attacks_history: [BB; SIZE] = [BB::ZERO; SIZE];
                    let mut is_failed: bool = false;
                    let mut i: usize = 0;

                    // verify magic_num works
                    while i < max_index && !is_failed {
                        // get magic index
                        let m: usize = ((blocker_bbs[i].data.wrapping_mul(magic_num))
                            >> (64 - mask_bitcount))
                            as usize;
                        // test magic index
                        if attacks_history[m].data == BB::ZERO.data {
                            // if uninitialized, record attacking pattern
                            attacks_history[m] = attacks_bbs[square];
                        } else if attacks_history[m].data != attacks_bbs[m].data {
                            // check if different attack patterns results from hash collision
                            is_failed = true;
                        }
                        i += 1;
                    }
                    if !is_failed {
                        return magic_num;
                    }
                    _attempts += 1;
                }
                panic!("magic number not found!")
            }

            PieceT::Rook => {
                const SIZE: usize = 1 << 12;

                let mut blocker_bbs: [BB; SIZE] = [BB::ZERO; SIZE];
                let mut attacks_bbs: [BB; SIZE] = [BB::ZERO; SIZE];

                let attack_mask: BB = BB::ROOK_MBB_MASK[square];
                let max_index: usize = 1usize << mask_bitcount;
                let mut i: usize = 0;

                while i < max_index {
                    blocker_bbs[i] = BB::compute_occ_bb(i, mask_bitcount, attack_mask);
                    attacks_bbs[i] = BB::naive_rook_attack(square, blocker_bbs[i]);
                    i += 1
                }
                let mut _attempts: usize = 0;

                // brute force loop
                while _attempts < usize::MAX {
                    let magic_num: u64 = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();

                    // skip impossible magic_num candidates
                    if (attack_mask.data.wrapping_mul(magic_num)) & 0xFF00000000000000 < 6 {
                        continue; // note: idk why this is the condition
                    }

                    let mut attacks_history: [BB; SIZE] = [BB::ZERO; SIZE];
                    let mut is_failed: bool = false;
                    let mut i: usize = 0;

                    // verify magic_num works
                    while i < max_index && !is_failed {
                        // get magic index
                        let m: usize = ((blocker_bbs[i].data.wrapping_mul(magic_num))
                            >> (64 - mask_bitcount))
                            as usize;
                        // test magic index
                        if attacks_history[m].data == BB::ZERO.data {
                            // if uninitialized, record attacking pattern
                            attacks_history[m] = attacks_bbs[square];
                        } else if attacks_history[m].data != attacks_bbs[m].data {
                            // check if different attack patterns results from hash collision
                            is_failed = true;
                        }
                        i += 1;
                    }
                    if !is_failed {
                        return magic_num;
                    }
                    _attempts += 1;
                }
                panic!("magic number not found!")
            }
            _ => panic!("find_magic_number error: invalid PieceT variable"),
        };
    }

    pub const fn compute_occ_bb(index: usize, mask_bitcount: usize, attack_mask: BB) -> BB {
        let mut attack_mask: BB = attack_mask;
        let mut occupancy_bb: BB = BB::ZERO;
        let mut i: usize = 0;
        // while attack_mask is non-zero
        while i < mask_bitcount && attack_mask.data != 0 {
            // square_index is index of least_significant bit
            if let Some(square_index) = attack_mask.lsb_index() {
                attack_mask = attack_mask.pop_bit(square_index);
                // check that square is within range of index
                if index & (1 << i) != 0usize {
                    occupancy_bb.data |= 1u64 << square_index
                }
            }
            i += 1;
        }
        return occupancy_bb;
    }

    // todo: exact hashing function (ideas for space optimal solution)
    //const fn init_bishop_attack() -> [BB; 64] {
    //    !unimplemented!()
    //}
    //const fn init_rook_attack() -> [BB; 64] {
    //    !unimplemented!()
    //}
    //const fn init_queen_attack() -> [BB; 64] {
    //    !unimplemented!()
    //}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
    Neither,
}
