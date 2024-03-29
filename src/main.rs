#![allow(dead_code)]
#![allow(unused_imports)]
mod bitboard;
mod chessboard;
mod chessmove;
use std::time::Instant;

use crate::bitboard::*;
use crate::chessboard::*;
use crate::chessmove::*;
use rand::Rng;

/* custom position for webperft */
#[rustfmt::skip]
pub const KIWIPETE: [BB; 12] = [
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000100_00000000_00000000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00001000_00000000_00100000_00000000_00000000}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00011000_00000000}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001}, // ♖
    BB { data: 0b00000000_00000000_00000000_00010000_00001000_00000000_11100111_00000000}, // ♙
    BB { data: 0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00000000_00001000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b00000000_00000000_01000100_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00000000_00000010_10000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_10110100_00001010_00000000_01000000_00000001_00000000_00000000}, // ♟
];

#[rustfmt::skip]
pub const POS3: [BB; 12] = [ //no castle
    BB { data: 0b00000000_00000000_00000000_10000000_00000000_00000000_00000000_00000000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_01000000_00000000_00000000_00000000}, // ♖
    BB { data: 0b00000000_00000000_00000000_01000000_00000000_00000000_00001010_00000000}, // ♙
    BB { data: 0b00000000_00000000_00000000_00000000_00000001_00000000_00000000_00000000}, // ♚
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b00000000_00000000_00000000_00000001_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_00100000_00010000_00000000_00000100_00000000_00000000_00000000}, // ♟
];
pub const POS3_CASTLE: [bool; 4] = [false, false, false, false];

#[rustfmt::skip]
pub const POS4: [BB; 12] = [
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♖
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♙
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♟
];

#[rustfmt::skip]
pub const POS5: [BB; 12] = [
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♖
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♙
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♟
];

#[rustfmt::skip]
pub const POS6: [BB; 12] = [
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♖
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♙
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♟
];

fn main() {
    let moves_indices: [usize; 0] = [];
    //2kr3r/p1pNqpb1/1n2pnp1/Pb1P4/1p2P3/2N2Q2/1PPBBP1P/1R2K1bR b K - 0 5
    //let moves_indices = [0usize, 0, 27, 41, 0, 39, 0, 28, 0];
    let moves_indices = [13usize];

    let mut chessboard = CB::default();
    chessboard.piece_bbs = POS3;
    chessboard.mailbox = generate_mailbox(chessboard.piece_bbs);
    chessboard.castle_bools = POS3_CASTLE;
    let mut moves_arr = MovesArray::new();
    //let mut chessboard = CB::from_fen(TEST_FEN2);
    moves_arr = chessboard.generate_moves();
    moves_arr.sort();
    for i in moves_indices {
        chessboard = chessboard.update_state(moves_arr.data[i].unwrap());
        moves_arr = chessboard.generate_moves();
        if i == moves_indices.len() {
            println!("{:?}", moves_arr);
        }
        moves_arr.sort();
    }
    /*
     */
    //println!("mailbox:");
    //println!("{:?}", chessboard.mailbox);
    println!("==== start position ====\n");
    println!("chessboard:\n{}", chessboard);
    println!("========================");
    println!("mailbox:\n{}", print_mailbox(chessboard.mailbox));
    println!("========================");
    println!(
        "castle_bools: ({},{},{},{})",
        chessboard.castle_bools[0],
        chessboard.castle_bools[1],
        chessboard.castle_bools[2],
        chessboard.castle_bools[3]
    );
    println!("========================");
    let mut i: usize = 1;
    let max_depth = 7;
    while i <= max_depth {
        let now = Instant::now();
        let total = chessboard.perft_count(i);
        let elapsed = now.elapsed();
        let mut arr = chessboard.generate_moves();
        arr.sort();
        //arr.sort();
        let mut result_str_vec = Vec::<String>::new();
        let mut j: usize = 0;
        while j < arr.len() && arr.data()[j] != None {
            let chess_move = arr.data()[j].unwrap();
            let mut s = chess_move.print_move();
            let state = chessboard.update_state(chess_move);
            let subtotal = state.perft_count(i - 1);
            s.push_str(format!(" - {}", subtotal).as_str());
            //debug
            //s.push_str(format!("\n  type: {:?}", chess_move.get_move_type()).as_str(),);
            //s.push_str(format!("\n piece: {:?}", chessboard.mailbox[chess_move.source()]).as_str(),);
            result_str_vec.push(s);
            j += 1;
        }
        //result_str_vec.sort();
        println!("depth: {}, time: {}, total: {}", i, elapsed.as_secs(), total);
        for x in result_str_vec {
            println!("{}", x);
        }
        println!("\n");
        i += 1
    }
}
const TEST_FEN: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
const TEST_FEN2: &str = "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1";
/*
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
*/
