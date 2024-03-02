#![allow(dead_code)]
#![allow(long_running_const_eval)]

use bitintr::Pdep;
use rand::Rng;
use std::fmt::Display;
use std::ops::{BitAnd, BitOr, BitXor, Not};

/* general bitboard functions and definitions */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard {
    pub data: u64,
}

pub type BB = BitBoard;

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for i in 0..8u64 {
            s.push_str(&format!(
                "{:08b}",
                (self.data & (0xFFu64 << 8 * (7 - i))) >> 8 * (7 - i)
            ));
            s.push('\n');
        }
        write!(f, "{}", s)
    }
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

pub const SQUARE_SYM_REV: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", //
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", //
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", //
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", //
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", //
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", //
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", //
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", //
];

pub const SQUARE_SYM: [&str; 64] = [
    "h1", "g1", "f1", "e1", "d1", "c1", "b1", "a1", //
    "h2", "g2", "f2", "e2", "d2", "c2", "b2", "a2", //
    "h3", "g3", "f3", "e3", "d3", "c3", "b3", "a3", //
    "h4", "g4", "f4", "e4", "d4", "c4", "b4", "a4", //
    "h5", "g5", "f5", "e5", "d5", "c5", "b5", "a5", //
    "h6", "g6", "f6", "e6", "d6", "c6", "b6", "a6", //
    "h7", "g7", "f7", "e7", "d7", "c7", "b7", "a7", //
    "h8", "g8", "f8", "e8", "d8", "c8", "b8", "a8", //
];

/* some u64 bit manipulation support */
impl BitBoard {
    pub const ZERO: BB = BB { data: 0u64 };
    pub const ONES: BB = BB { data: u64::MAX };
    pub fn nth(n: usize) -> Self {
        Self { data: 1u64 << n }
    }

    pub const fn lsb_index(&self) -> Option<usize> {
        if self.data == 0u64 {
            return None;
        } else {
            return Some(self.data.trailing_zeros() as usize);
        }
    }

    pub fn set_bit(&mut self, i: usize) {
        self.data = self.data | 1u64 << i;
    }

    pub const fn get_bit(&self, i: usize) -> BB {
        BB {
            data: self.data & (1u64 << i),
        }
    }

    pub const fn pop_bit(&self, i: usize) -> BB {
        BB {
            data: match self.get_bit(i).data {
                0u64 => 0,
                //_ => self.data ^ (1u64 << i), //old line, idk how it works
                _ => self.data & !(1u64 << i),
            },
        }
    }
}

/* chessboard specific bitboard functions and definitions*/
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
pub type PT = PieceType;

impl PieceType {
    pub fn to_char(&self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }
}

pub const W_PAWN_ATTACKS: [BB; 64] = init_pawn_attack(Side::White);
pub const B_PAWN_ATTACKS: [BB; 64] = init_pawn_attack(Side::Black);
pub const KNIGHT_ATTACKS: [BB; 64] = init_knight_attack();
pub const KING_ATTACKS: [BB; 64] = init_king_attack();
//pub const OMNI_ATTACKS: [BB; 64] = init_omni_attack();

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub const fn update(&self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

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
                //left up is "<< 10"
                data |= (1u64 << i) << 10
            }
            if i % 8 > 1 {
                //right up is "<<  6"
                data |= (1u64 << i) << 6
            }
        }
        if i > 7 {
            if i % 8 < 6 {
                //left down is ">> 6"
                data |= (1u64 << i) >> 6
            }
            if i % 8 > 1 {
                //right down is ">> 10"
                data |= (1u64 << i) >> 10
            }
        }
        if i > 15 {
            if i % 8 < 7 {
                //down left is ">> 15"
                data |= (1u64 << i) >> 15
            }
            if i % 8 > 0 {
                //down right is ">> 17"
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
            data |= (1u64 << i) << 8;
        }
        if i > 7 {
            //down
            data |= (1u64 << i) >> 8;
        }
        if i % 8 > 0 {
            //right
            data |= (1u64 << i) >> 1;
        }
        if i % 8 < 7 {
            //left
            data |= (1u64 << i) << 1;
        }
        if i < 56 && i % 8 > 0 {
            //up right
            data |= ((1u64 << i) << 8) >> 1;
        }
        if i < 56 && 1 % 8 < 7 {
            //up left
            data |= ((1u64 << i) << 8) << 1;
        }
        if i > 7 && i % 8 > 0 {
            //down right
            data |= ((1u64 << i) >> 8) >> 1;
        }
        if i > 7 && i % 8 < 7 {
            //down left
            data |= ((1u64 << i) >> 8) << 1;
        }
        attack_array[i] = BB { data };
        i += 1;
    }
    return attack_array;
}

const fn init_omni_attack() -> [BB; 64] {
    let mut i: usize = 0;
    let mut attack_array: [BB; 64] = [BB::ZERO; 64];
    while i < 64usize {
        attack_array[i].data = get_queen_attack(i, BB::ZERO).data | KNIGHT_ATTACKS[i].data;
        i += 1;
    }
    return attack_array;
}

/* magic bitboard related functions and definitions */
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

pub const BISHOP_MBB_MASK: [BB; 64] = init_bishop_mbb_mask();
pub const ROOK_MBB_MASK: [BB; 64] = init_rook_mbb_mask();

pub const BISHOP_MAGICS: [u64; 64] = [
    0x140C80810488022,
    0x20021C01142000,
    0x308C2080200102,
    0x4040880000A09,
    0x824042080000001,
    0xC1010840807080,
    0x810C010403200000,
    0x49CE404044202081,
    0x4405048410020200,
    0x42104440080,
    0x801C12112008003,
    0x100080A43014001,
    0x20210010000,
    0x110020110080990,
    0x800004804042000,
    0x1000002434020800,
    0xC108E014890204,
    0x4040210440100,
    0x4808001000801012,
    0x8004620801080,
    0x481000290400A01,
    0x1000180A00921,
    0x1204010900A80492,
    0xA88400024041C00,
    0x1002100088501014,
    0x5045040818008C,
    0x2080081004408,
    0x208280005820002,
    0x509010040104008,
    0x8010004000241000,
    0x8908108440540400,
    0x142060800404240,
    0x231101010402410,
    0x2011140241020,
    0x100A002A00101180,
    0x2001010800110041,
    0x8118022401224100,
    0x4420092A40020800,
    0x22D000C880031400,
    0x102108002A420,
    0x4008044404102020,
    0x8000842402002000,
    0x200242400080E,
    0x30004202208802,
    0x11214000601,
    0x10C0008099011081,
    0x10080104608A0C00,
    0x2285D00202700,
    0x9A182414050000,
    0x20100A210223022,
    0x2C02080102,
    0x20884010,
    0x280029002022040,
    0x8250102490342010,
    0x40020464048080,
    0x4120040102042200,
    0x280A010401018800,
    0x8010008084104200,
    0x9009002484501A,
    0x1A08830080420208,
    0x2000064022604100,
    0x12400420044101,
    0x40042818810C00,
    0x1024211464008200,
];
pub const ROOK_MAGICS: [u64; 64] = [
    0x818001C000802018,
    0xA240100020004000,
    0x100081041002000,
    0x1080048010000800,
    0x8600020020040810,
    0x580018002004400,
    0x1080020000800100,
    0x20000204A088401,
    0x4000800080204000,
    0x40804000200080,
    0x801000200080,
    0x222000C10204200,
    0x42000600081020,
    0xA2001004080200,
    0x1000800100800200,
    0x82000092010044,
    0x800848000400420,
    0x30044040002001,
    0x8000110041002004,
    0x4200200A0010,
    0x810808004000800,
    0xC028808002000400,
    0x280040090080201,
    0x804020000508104,
    0x80400480088024,
    0x400200440100241,
    0x401001100200040,
    0x100080800800,
    0x8010100041008,
    0x8000020080800400,
    0x1000012400024830,
    0x4008200210054,
    0x8084A0082002100,
    0x4080201000404000,
    0xC000102001004100,
    0x4082101001002,
    0x9820800800400,
    0x900C800400800200,
    0x9040080204008150,
    0x80B0140446000493,
    0x6040244000828000,
    0x210002000504000,
    0x15002002110040,
    0x41001000210008,
    0x1004800050010,
    0x2000804010100,
    0x5008081002040081,
    0x220040A1020004,
    0x101400120800180,
    0x2040002000C08180,
    0x1120001000480040,
    0x18001020400A0200,
    0x4050010080100,
    0x1023020080040080,
    0x1080102100400,
    0x1000282004300,
    0x190401100800021,
    0x805854001021021,
    0x600010400C200101,
    0x10210009100005,
    0x1001001002080005,
    0x9801000C00080A29,
    0x2006080A45029014,
    0x8804581022C02,
];

pub const BISHOP_ATTACKS_MBB: [[BB; 1 << 9]; 64] = init_bishop_attack_mbb();
pub const ROOK_ATTACKS_MBB: [[BB; 1 << 12]; 64] = init_rook_attack_mbb();

// naive function to calculate bishop attack patterns
pub const fn naive_bishop_attack(i: usize, blockers: BB) -> BB {
    let i_rank: isize = (i as isize) / 8isize;
    let i_file: isize = (i as isize) % 8isize;
    let mut j: isize = 1;
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
                    if 1u64 << (i_rank + j) * 8 + (i_file + j) & blockers.data != BB::ZERO.data {
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
                    if 1u64 << (i_rank - j) * 8 + (i_file + j) & blockers.data != BB::ZERO.data {
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
                    if 1u64 << (i_rank + j) * 8 + (i_file - j) & blockers.data != BB::ZERO.data {
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
                    if 1u64 << (i_rank - j) * 8 + (i_file - j) & blockers.data != BB::ZERO.data {
                        dr_is_blocked = true;
                    }
                }
            }
        }
        j += 1
    }

    BB { data }
}

pub const fn naive_rook_attack(i: usize, blockers: BB) -> BB {
    let i_rank: isize = (i as isize) / 8isize; // row
    let i_file: isize = (i as isize) % 8isize; // collumn
    let mut data: u64 = 0u64;

    let mut j: isize = 1;
    let mut r_is_blocked: bool = false;
    let mut l_is_blocked: bool = false;
    let mut u_is_blocked: bool = false;
    let mut d_is_blocked: bool = false;

    while j <= 7 {
        // right direction: (file - j, rank)
        if 0 <= i_file - j {
            if !r_is_blocked {
                data |= 1u64 << (i_rank * 8) + (i_file - j);
                if 0 < i_file - j {
                    if 1u64 << (i_rank * 8) + (i_file - j) & blockers.data != BB::ZERO.data {
                        r_is_blocked = true;
                    }
                }
            }
        }
        // left direction: (file + j, rank)
        if i_file + j <= 7 {
            if !l_is_blocked {
                data |= 1u64 << (i_rank * 8) + (i_file + j);
                if i_file + j < 7 {
                    if 1u64 << (i_rank * 8) + (i_file + j) & blockers.data != BB::ZERO.data {
                        l_is_blocked = true;
                    }
                }
            }
        }
        //   up direction: (file, rank + j)
        if i_rank + j <= 7 {
            if !u_is_blocked {
                data |= 1u64 << ((i_rank + j) * 8) + i_file;
                if i_rank + j < 7 {
                    if 1u64 << ((i_rank + j) * 8) + i_file & blockers.data != BB::ZERO.data {
                        u_is_blocked = true;
                    }
                }
            }
        }
        // down direction: (file, rank - j)
        if 0 <= i_rank - j {
            if !d_is_blocked {
                data |= 1u64 << ((i_rank - j) * 8) + i_file;
                if 0 < i_rank - j {
                    if 1u64 << ((i_rank - j) * 8) + i_file & blockers.data != BB::ZERO.data {
                        d_is_blocked = true;
                    }
                }
            }
        }
        j += 1
    }
    BB { data }
}

pub const fn magic_index(magic_num: u64, blockers: BB, bitcount: usize) -> usize {
    ((blockers.data.wrapping_mul(magic_num)) >> (64 - bitcount)) as usize
}

fn find_magic_number(square: usize, mask_bitcount: usize, piece_type: PieceType) -> u64 {
    let max_index: usize = 1 << mask_bitcount;
    let mut rng = rand::thread_rng();
    let mut blockers: Vec<BB> = vec![BB::ZERO; max_index];
    let mut attacks: Vec<BB> = vec![BB::ZERO; max_index];
    //let mut attack_history: Vec<BB> = vec![BB::ZERO; max_index];
    let mask = match piece_type {
        PieceType::Bishop => BISHOP_MBB_MASK[square],
        PieceType::Rook => ROOK_MBB_MASK[square],
        _ => panic!("find_magic_number error: invalid piece type!"),
    };

    let mut i: usize = 0;
    // precalculate table
    while i < max_index {
        blockers[i] = compute_occ_bb(i, mask_bitcount, mask);
        attacks[i] = match piece_type {
            PieceType::Bishop => naive_bishop_attack(square, blockers[i]),
            PieceType::Rook => naive_rook_attack(square, blockers[i]),
            _ => panic!("find_magic_number error: invalid piece type!"),
        };
        i += 1;
    }

    let mut _attempts: usize = 0;
    // bruteforce magic number
    while _attempts < usize::MAX {
        let magic_num: u64 = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();

        // skip bad magic_num
        if (mask.data.wrapping_mul(magic_num) & 0xFF00000000000000u64).count_ones() < 6 {
            continue;
        }

        let mut attack_history = vec![BB::ZERO; max_index];
        let mut i: usize = 0;
        let mut is_failed = false;

        while !is_failed && i < max_index {
            let m = magic_index(magic_num, blockers[i], mask_bitcount);

            if attack_history[m] == BB::ZERO {
                attack_history[m] = attacks[i];
            } else {
                is_failed = attack_history[m] != attacks[i];
            }
            i += 1
        }
        if !is_failed {
            return magic_num;
        }
        _attempts += 1;
    }
    panic!("find_magic_number error: failed to find magic!");
}

// note: since this function is not const, we only use it to verify compute_occ_bb is correct
pub fn pdep_occ_bb(index: usize, attack_mask: BB) -> BB {
    let data = (index as u64).pdep(attack_mask.data);
    BB { data }
}

pub const fn compute_occ_bb(index: usize, mask_bitcount: usize, attack_mask: BB) -> BB {
    /* use pdep? */
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

pub fn init_magics(piece_type: PieceType) -> [u64; 64] {
    let mut i: usize = 0;
    let mut magic_nums: [u64; 64] = [0u64; 64];
    match piece_type {
        PieceType::Bishop => {
            println!("Finding magic numbers for: Bishop")
        }
        PieceType::Rook => {
            println!("Finding magic numbers for: Rook")
        }
        _ => panic!("error: invalid piece_type parameter"),
    }

    while i < 64 {
        println!("calculating nth magic number: {}", i);
        let mut magic_found = false;
        let mut magic: u64 = 0;
        while !magic_found {
            magic = match piece_type {
                PieceType::Bishop => find_magic_number(i, BISHOP_OCC_BITCOUNT[i], piece_type),
                PieceType::Rook => find_magic_number(i, ROOK_OCC_BITCOUNT[i], piece_type),
                _ => panic!("init_magic_numbers error: invalid PieceType variable"),
            };
            magic_found = true;
            for x in magic_nums {
                if x == magic {
                    magic_found = false;
                    break;
                };
            }
        }
        magic_nums[i] = magic;
        i += 1;
    }
    return magic_nums;
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
        let i_rank: isize = (i as isize) / 8isize; // row
        let i_file: isize = (i as isize) % 8isize; // collumn
        let mut j: isize = 1;
        let mut data: u64 = 0u64;
        while j < 7 {
            // right direction: (file - j, rank)
            if 0 < i_file - j {
                data |= 1u64 << (i_rank * 8) + (i_file - j);
            }
            // left direction: (file + j, rank)
            if i_file + j < 7 {
                data |= 1u64 << (i_rank * 8) + (i_file + j);
            }
            //   up direction: (file, rank + j)
            if i_rank + j < 7 {
                data |= 1u64 << ((i_rank + j) * 8) + i_file;
            }
            // down direction: (file, rank - j)
            if 0 < i_rank - j {
                data |= 1u64 << ((i_rank - j) * 8) + i_file;
            }
            j += 1
        }
        attack_array[i] = BB { data };
        i += 1;
    }
    return attack_array;
}

const SIZE_BISHOP: usize = 1 << 9;
const SIZE_ROOK: usize = 1 << 12;

const fn init_bishop_attack_mbb() -> [[BB; 1 << 9]; 64] {
    let mut i: usize = 0;
    let mut attacks: [[BB; 1 << 9]; 64] = [[BB::ZERO; 1 << 9]; 64];
    while i < 64 {
        let mask = BISHOP_MBB_MASK[i];
        let bitcount = BISHOP_OCC_BITCOUNT[i];
        let max_index: usize = 1 << bitcount;

        let mut j: usize = 0;
        while j < max_index {
            let blockers = compute_occ_bb(j, bitcount, mask);
            let m = magic_index(BISHOP_MAGICS[i], blockers, bitcount);

            if attacks[i][m].data == BB::ZERO.data {
                attacks[i][m] = naive_bishop_attack(i, blockers);
            } else if attacks[i][m].data != naive_bishop_attack(i, blockers).data {
                panic!("init_bishop_attack_mbb error: invalid colision!");
            }
            j += 1;
        }
        i += 1;
    }
    return attacks;
}

const fn init_rook_attack_mbb() -> [[BB; 1 << 12]; 64] {
    let mut i: usize = 0;
    let mut attacks: [[BB; 1 << 12]; 64] = [[BB::ZERO; 1 << 12]; 64];
    while i < 64 {
        let mask = ROOK_MBB_MASK[i];
        let bitcount = ROOK_OCC_BITCOUNT[i];
        let max_index: usize = 1 << bitcount;

        let mut j: usize = 0;
        while j < max_index {
            let blockers = compute_occ_bb(j, bitcount, mask);
            let m = magic_index(ROOK_MAGICS[i], blockers, bitcount);

            if attacks[i][m].data == BB::ZERO.data {
                attacks[i][m] = naive_rook_attack(i, blockers);
            } else if attacks[i][m].data != naive_rook_attack(i, blockers).data {
                panic!("init_rook_attack_mbb error: invalid colision!");
            }
            j += 1;
        }
        i += 1;
    }
    return attacks;
}

//#[inline(always)]
pub const fn get_bishop_attack(square: usize, blockers: BB) -> BB {
    let mask = BISHOP_MBB_MASK[square];
    let data = blockers.data & mask.data;
    let blockers = BB { data };
    let m = magic_index(BISHOP_MAGICS[square], blockers, BISHOP_OCC_BITCOUNT[square]);
    return BISHOP_ATTACKS_MBB[square][m];
}

//#[inline(always)]
pub const fn get_rook_attack(square: usize, blockers: BB) -> BB {
    let mask = ROOK_MBB_MASK[square];
    let data = blockers.data & mask.data;
    let blockers = BB { data };
    let m = magic_index(ROOK_MAGICS[square], blockers, ROOK_OCC_BITCOUNT[square]);
    return ROOK_ATTACKS_MBB[square][m];
}

//#[inline(always)]
pub const fn get_queen_attack(square: usize, blockers: BB) -> BB {
    BB {
        data: get_bishop_attack(square, blockers).data | get_rook_attack(square, blockers).data,
    }
}
