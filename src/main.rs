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

fn main() {
    for i in W_PAWN_ATTACKS {
        //println!("{}", i);
    }

    /*
    println!("Hello, world!");
    let mut chessboard = CB::default();
    let arr = chessboard.generate_moves_array();
    println!("================================================================");
    println!("{}", chessboard);
    println!("{:?}\n", arr);

    chessboard = chessboard.update_state(arr.data()[0].unwrap());
    println!("================================================================");
    println!("{}", chessboard);
    let arr = chessboard.generate_moves_array();
    println!("{:?}\n", arr);

    chessboard = chessboard.update_state(arr.data()[0].unwrap());
    println!("================================================================");
    println!("{}", chessboard);
    let arr = chessboard.generate_moves_array();
    println!("{:?}\n", arr);

    chessboard = chessboard.update_state(arr.data()[0].unwrap());
    println!("================================================================");
    println!("{}", chessboard);
    let arr = chessboard.generate_moves_array();
    println!("{:?}\n", arr);

    chessboard = chessboard.update_state(arr.data()[0].unwrap());
    println!("================================================================");
    println!("{}", chessboard);
    let arr = chessboard.generate_moves_array();
    println!("{:?}\n", arr);
    */
    /* template
    arr = chessboard.generate_moves_array();
    arr.sort();
    chessboard = chessboard.update_state(arr.data()[0].unwrap());
    */
    //let moves_indices: [usize; 12] = [8usize, 0, 6, 8, 18, 1, 1, 4, 0, 0,4, 15]; //series 1
    let moves_indices: [usize; 0] = [];
    //let moves_indices = [6, 0, 6, 0, 8, 10, 18]; //series 2
    let moves_indices = [
        2usize, 0, 8, 0, 4, 0, 16, 0, 0, 0, 6, 0, 11, 7, 0, 1, 9, 14, 0, 18, 15, 21, 9, 4, 0, 10,
        2, 15, 1, 9,
    ];
    let mut chessboard = CB::default();
    let mut moves_arr = MovesArray::new();
    let (x, y) = (23, 58);
    println!("RAYS[x][y]:");
    println!("{}", RAYS[x][y]);
    //let mut chessboard = CB::from_fen(TEST_FEN2);
    moves_arr = chessboard.generate_moves();
    moves_arr.sort();
    for i in moves_indices {
        chessboard = chessboard.update_state(moves_arr.data()[i].unwrap());
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
    println!("{}", chessboard);
    println!("========================");
    let mut i: usize = 1;
    let max_depth = 5;
    while i < max_depth {
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
