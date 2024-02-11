#![allow(dead_code)]
mod bitboard;
mod chessboard;
use crate::bitboard::*;
use rand::Rng;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_naive_bishop_attack() {
        let i: usize = 27;
        let blocker_data: u64 = 0b0000000000000001000000000001000000000000000000000010000000000000;
        let answer: BB = BB {
            data: 0b0000000000000001000000100001010000000000000101000010001000000001,
        };
        assert_eq!(answer, naive_bishop_attack(i, BB { data: blocker_data }));
    }

    #[test]
    fn test_naive_rook_attack() {
        let i: usize = 27;
        let blocker_data: u64 = 0b0000000000001000000000000000000001000001000010000000000000000000;
        let answer: BB = BB {
            data: 0b0000000000001000000010000000100001110111000010000000000000000000,
        };
        assert_eq!(answer, naive_rook_attack(i, BB { data: blocker_data }));
    }
    #[test]
    fn test_compute_occ_bb_bishop() {
        let mut rng = rand::thread_rng();
        let i: usize = rng.gen_range(0..64);
        let bishop_mask = BISHOP_MBB_MASK[i];
        let bitcount = BISHOP_OCC_BITCOUNT[i];
        let mut j = 0;
        while j < 1usize << bitcount {
            let x = pdep_occ_bb(j, bishop_mask);
            let y = compute_occ_bb(j, bitcount, bishop_mask);
            assert_eq!(x, y);
            j += 1
        }
    }
    #[test]
    fn test_compute_occ_bb_rook() {
        let mut rng = rand::thread_rng();
        let i: usize = rng.gen_range(0..64);
        let rook_mask = ROOK_MBB_MASK[i];
        let bitcount = ROOK_OCC_BITCOUNT[i];
        let mut j = 0;
        while j < 1usize << bitcount {
            let x = pdep_occ_bb(j, rook_mask);
            let y = compute_occ_bb(j, bitcount, rook_mask);
            assert_eq!(x, y);
            j += 1
        }
    }
}
