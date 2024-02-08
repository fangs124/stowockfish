/*
use super::*;

#[allow(long_running_const_eval)]

// implementation for things relevant to magic bitboards
impl BitBoard {
    // naively implemented functions for debugging mbb
    pub(super) const fn naive_bishop_attack(i: usize, blocker: BB) -> BB {
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
            if i_rank + j <= 7 && i_file + j <= 7 && !ul_is_blocked {
                data |= 1u64 << (i_rank + j) * 8 + (i_file + j);
            }
            //  down left direction: (-,+)
            if 0 <= i_rank - j && i_file + j <= 7 && !dl_is_blocked {
                data |= 1u64 << (i_rank - j) * 8 + (i_file + j);
            }
            //    up right direction: (+,-)
            if i_rank + j <= 7 && 0 <= i_file - j && !ur_is_blocked {
                data |= 1u64 << (i_rank + j) * 8 + (i_file - j);
            }
            //  down right direction: (-,-)
            if 0 <= i_rank - j && 0 <= i_file - j && !dr_is_blocked {
                data |= 1u64 << (i_rank - j) * 8 + (i_file - j);
            }
            ul_is_blocked = ul_is_blocked
                || (1u64 << (i_rank + j) * 8 + (i_file + j) & blocker.data != BB::ZERO.data);
            dl_is_blocked = dl_is_blocked
                || (1u64 << (i_rank - j) * 8 + (i_file + j) & blocker.data != BB::ZERO.data);
            ur_is_blocked = ur_is_blocked
                || (1u64 << (i_rank + j) * 8 + (i_file - j) & blocker.data != BB::ZERO.data);
            dr_is_blocked = dr_is_blocked
                || (1u64 << (i_rank - j) * 8 + (i_file - j) & blocker.data != BB::ZERO.data);
            j += 1
        }

        BB { data }
    }

    pub(super) const fn naive_rook_attack(i: usize, blocker: BB) -> BB {
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

    pub(super) const fn naive_queen_attack(i: usize, blocker: BB) -> BB {
        let bp = BB::naive_bishop_attack(i, blocker);
        let rk = BB::naive_rook_attack(i, blocker);
        return BB {
            data: bp.data | rk.data,
        };
    }

    // magic bitboard blocker masks
    pub(super) const fn init_bishop_attack_mbb_mask() -> [BB; 64] {
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

    pub(super) const fn init_rook_attack_mbb_mask() -> [BB; 64] {
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

    pub(super) const fn init_queen_attack_mbb_mask() -> [BB; 64] {
        let bp_arr: [BB; 64] = BB::init_bishop_attack_mbb_mask();
        let rk_arr: [BB; 64] = BB::init_rook_attack_mbb_mask();
        let mut attack_array: [BB; 64] = [BB::ZERO; 64];
        let mut i: usize = 0;
        while i < 64 {
            let data = bp_arr[i].data | rk_arr[i].data;
            attack_array[i] = BB { data };
            i += 1;
        }
        return attack_array;
    }

    // magic bitboard support functions
    pub const fn compute_occupancy_bb(index: usize, mask_bitcount: usize, attack_mask: BB) -> BB {
        let mut attack_mask = attack_mask; // ???: is this necessary here?
        let mut occupancy_bb: BB = BB::ZERO;
        let mut i: usize = 0;
        while i < mask_bitcount && attack_mask.data != 0 {
            if let Some(square_index) = attack_mask.lsb_index() {
                attack_mask = attack_mask.pop_bit(square_index);
                if index & (1 << i) != 0usize {
                    occupancy_bb.data |= 1u64 << square_index;
                }
            } else {
                panic!("attack_mask was zero!");
            }
            i += 1;
        }
        return occupancy_bb;
    }

    pub const ROOK_OCCUPANCY_BITCOUNT: [usize; 64] = BB::init_relv_occupancy_bitcount_rook();
    pub const BISHOP_OCCUPANCY_BITCOUNT: [usize; 64] = BB::init_relv_occupancy_bitcount_bishop();

    pub(super) const fn init_relv_occupancy_bitcount_rook() -> [usize; 64] {
        let mut bit_counts: [usize; 64] = [0usize; 64];
        let mut i: usize = 0;
        while i < 64 {
            bit_counts[i] = Self::ROOK_ATTACKS_MBB_MASK[i].data.count_ones() as usize;
            i += 1
        }
        return bit_counts;
    }

    pub(super) const fn init_relv_occupancy_bitcount_bishop() -> [usize; 64] {
        let mut bit_counts: [usize; 64] = [0usize; 64];
        let mut i: usize = 0;
        while i < 64 {
            bit_counts[i] = Self::BISHOP_ATTACKS_MBB_MASK[i].data.count_ones() as usize;
            i += 1
        }
        return bit_counts;
    }

    const fn find_magic_number(
        random_seed: u64,
        square: usize,
        mask_bitcount: usize,
        piece_type: PieceT,
    ) -> u64 {
        assert!(square < 64);
        const SIZE: usize = 4096;
        let mut blocker_bb: [BB; SIZE] = [BB::ZERO; SIZE];
        let mut attacks_bb: [BB; SIZE] = [BB::ZERO; SIZE];
        let mut used_attacks: [BB; SIZE] = [BB::ZERO; SIZE];
        let attack_mask: BB = match piece_type {
            PieceT::Rook => BB::ROOK_ATTACKS_MBB_MASK[square],
            PieceT::Bishop => BB::BISHOP_ATTACKS_MBB_MASK[square],
            _ => unreachable!(),
        };

        let max_index: usize = 1usize << mask_bitcount;
        let mut i: usize = 0;
        while i < max_index {
            blocker_bb[i] = BB::compute_occupancy_bb(i, mask_bitcount, attack_mask);
            attacks_bb[i] = match piece_type {
                PieceT::Bishop => BB::naive_bishop_attack(square, blocker_bb[i]),
                PieceT::Rook => BB::naive_rook_attack(square, blocker_bb[i]),
                _ => unreachable!(),
            };
            i += 1;
        }

        let mut _attempts: usize = 0;
        let mut random_seed: u64 = random_seed;
        // bruteforce loop
        while _attempts < usize::MAX {
            let n1 = mbb::get_rand_u64(random_seed);
            let n2 = mbb::get_rand_u64(n1);
            let n3 = mbb::get_rand_u64(n2);
            let magic_num: u64 = n1 & n2 & n3;
            random_seed = n3;
            // skip impossible magic_nums
            if (attack_mask.data.wrapping_mul(magic_num)) & 0xFF00000000000000 < 6 {
                continue;
            }
            used_attacks = [BB::ZERO; SIZE];
            let mut is_failed: bool = false;
            let mut i: usize = 0;
            while i < max_index && !is_failed {
                // get magical index
                let m: usize =
                    ((blocker_bb[i].data.wrapping_mul(magic_num)) >> (64 - mask_bitcount)) as usize;
                // test magic index
                if used_attacks[m].data == BB::ZERO.data {
                    // initialize used_attacks
                    used_attacks[m] = attacks_bb[square];
                } else if used_attacks[m].data != attacks_bb[m].data {
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

    // magic bitboard implementation

    pub const ROOK_MAGIC_NUMS: [u64; 64] = mbb::init_rook_magic_numbers();
    pub const BISHOP_MAGIC_NUMS: [u64; 64] = mbb::init_rook_magic_numbers();

    pub(super) const fn get_magic_ind(blocker: BB, magic_number: u64, bitcount: usize) -> usize {
        (blocker.data.wrapping_mul(magic_number) >> bitcount) as usize
    }
    pub(super) const fn init_rook_attack_mbb() -> [[BB; 4096]; 64] {
        let mut square: usize = 0;
        let mut attacks: [[BB; 4096]; 64] = [[BB::ZERO; 4096]; 64];
        // loop over 64 squares
        while square < 64 {
            let attack_mask = BB::ROOK_ATTACKS_MBB_MASK[square];
            let bitcount = BB::ROOK_OCCUPANCY_BITCOUNT[square];
            let max_index = 1usize << bitcount;
            let mut i: usize = 0;
            while i < max_index {
                let blocker = BB::compute_occupancy_bb(i, bitcount, attack_mask);
                let m = BB::get_magic_ind(blocker, BB::ROOK_MAGIC_NUMS[square], bitcount);
                attacks[i][m] = BB::naive_rook_attack(square, blocker);
                i += 1
            }
            square += 1;
        }
        attacks
    }

    pub(super) const fn init_bishop_attack_mbb() -> [[BB; 512]; 64] {
        let mut square: usize = 0;
        let mut attacks: [[BB; 512]; 64] = [[BB::ZERO; 512]; 64];
        // loop over 64 squares
        while square < 64 {
            let attack_mask = BB::BISHOP_ATTACKS_MBB_MASK[square];
            let bitcount = BB::BISHOP_OCCUPANCY_BITCOUNT[square];
            let max_index = 1usize << bitcount;
            let mut i: usize = 0;
            while i < max_index {
                let blocker = BB::compute_occupancy_bb(i, bitcount, attack_mask);
                let m = BB::get_magic_ind(blocker, BB::BISHOP_MAGIC_NUMS[square], bitcount);
                attacks[i][m] = BB::naive_bishop_attack(square, blocker);
                i += 1
            }
            square += 1;
        }
        attacks
    }
    //pub(super) const fn init_queen_attack_mbb() -> [[BB; 512]; 64] {
    //    unimplemented!()
    //}

    pub const fn rook_attack_mbb(i: usize, blocker: BB) -> BB {
        let blocker_data = BB::BISHOP_ATTACKS_MBB_MASK[i].data & blocker.data;
        let m = BB::get_magic_ind(
            BB { data: blocker_data },
            BB::ROOK_MAGIC_NUMS[i],
            BB::ROOK_OCCUPANCY_BITCOUNT[i],
        );
        return BB::ROOK_ATTACKS_MBB[i][m];
    }
}

mod mbb {
    /*
    // init magic numbers
    void init_magic_numbers() {
        // loop over 64 board squares
        for (int square = 0; square < 64; square++)
            // init rook magic numbers
            rook_magic_numbers[square] = find_magic_number(square, rook_relv_bits[square], rook);

        // loop over 64 board squares
        for (int square = 0; square < 64; square++)
            // init bishop magic numbers
            bishop_magic_numbers[square] = find_magic_number(square, bishop_relv_bits[square], bishop);
    }
    */

    use super::PieceT;

    pub const fn init_rook_magic_numbers() -> [u64; 64] {
        let r: u64 = 3206097770;
        let mut i: usize = 0;
        let mut rook_magic_nums: [u64; 64] = [0u64; 64];
        while i < 64 {
            rook_magic_nums[i] = super::BB::find_magic_number(
                r,
                i,
                super::BB::ROOK_OCCUPANCY_BITCOUNT[i],
                PieceT::Rook,
            );
            i += 1;
        }
        return rook_magic_nums;
    }
    pub const fn init_bishop_magic_numbers() -> [u64; 64] {
        let r: u64 = 3206097770;
        let mut i: usize = 0;
        let mut bishop_magic_nums: [u64; 64] = [0u64; 64];
        while i < 64 {
            bishop_magic_nums[i] = super::BB::find_magic_number(
                r,
                i,
                super::BB::BISHOP_OCCUPANCY_BITCOUNT[i],
                PieceT::Bishop,
            );
            i += 1;
        }
        return bishop_magic_nums;
    }
    /*
    pub(crate) const fn get_rand_u32() -> u32 {
        let mut num: u32 = unsafe { RANDOM_STATE_U32 };
        // bit hackery
        num ^= num << 13;
        num ^= num >> 17;
        num ^= num << 5;
        unsafe { RANDOM_STATE_U32 = num }
        return num;
    }
    */
    pub(crate) const fn get_rand_u64(random_seed: u64) -> u64 {
        let mut num: u64 = random_seed;
        // bit hackery
        num ^= num << 13;
        num ^= num >> 7;
        num ^= num << 17;
        return num;
    }
}
*/
