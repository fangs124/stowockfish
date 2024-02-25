#![allow(dead_code)]
#![allow(long_running_const_eval)]

use std::{fmt::{self, Display}, ops::Index};

use crate::bitboard::*;
use crate::chessmove::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
    Neither,
    Both,
}

// note: castle_bools[] = [white-king  side castle,
//                         white-queen side castle,
//                         black-king  side castle, 
//                         black-queen side castle,
//                        ]
// pieces: white king, queen, knight, bishop, rook, pawn, 
//         black king...
#[derive(Debug, PartialEq, Eq)]
pub struct ChessBoard {
    pub piece_bbs: [BB;12],
    pub castle_bools: [bool; 4],
    pub enpassant_bb: BB,
    pub side_to_move: Side,
    pub half_move_clock: usize,
    pub full_move_counter: usize,
}
pub type CB = ChessBoard;

impl Default for ChessBoard {
    fn default() -> Self {
        Self { piece_bbs: INITIAL_CHESS_POS, 
            castle_bools: [true; 4], 
            enpassant_bb:BB::ZERO, 
            side_to_move: Side::White, 
            half_move_clock: 0,
            full_move_counter : 0,
        }
    }
}

impl Display for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        
        // get empty_squares
        let mut empty_squares = BB::ZERO;
        for piece_bb in self.piece_bbs {
            empty_squares.data = piece_bb.data | empty_squares.data;
        }
        empty_squares.data = !empty_squares.data;
        //println!("empty squares:");
        //println!("{}", empty_squares);
        
        // append characters according to piece
        for i in 0..64u64 {
            if (1u64 << i) & empty_squares.data != BB::ZERO.data {
                s.push('.' );
                //println!("empty!");
            }
            else {
                let mut j = 0usize;
                while j < self.piece_bbs.len() {
                    let piece_bb: BB = self.piece_bbs[j];
                    if (1u64 << i) & piece_bb.data != BB::ZERO.data {
                        s.push(UNICODE_SYM[j]);
                        //s.push(ASCII_SYM[j]);
                        //println!("piece!");
                    }
                    j+= 1;
                }
            }
            if (i % 8 == 7) {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

const SQUARE_SYM_REV: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", //
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", //
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", //
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", //
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", //
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", //
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", //
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", //
];

const SQUARE_SYM: [&str; 64] = [
    "h1", "g1", "f1", "e1", "d1", "c1", "b1", "a1", //
    "h2", "g2", "f2", "e2", "d2", "c2", "b2", "a2", //
    "h3", "g3", "f3", "e3", "d3", "c3", "b3", "a3", //
    "h4", "g4", "f4", "e4", "d4", "c4", "b4", "a4", //
    "h5", "g5", "f5", "e5", "d5", "c5", "b5", "a5", //
    "h6", "g6", "f6", "e6", "d6", "c6", "b6", "a6", //
    "h7", "g7", "f7", "e7", "d7", "c7", "b7", "a7", //
    "h8", "g8", "f8", "e8", "d8", "c8", "b8", "a8", //
];

/* indexing the 64-squares:
  |-----------------------| BLACK KING SIDE
8 |63 62 61 60 59 58 57 56|
7 |55 54 53 52 51 50 49 48|
6 |47 46 45 44 43 42 41 40|
5 |39 38 37 36 35 34 33 32|
4 |31 30 29 28 27 26 25 24|
3 |23 22 21 20 19 18 17 16|
2 |15 14 13 12 11 10  9  8|
1 | 7  6  5  4  3  2  1  0|
  |-----------------------| WHITE KING SIDE
    A  B  C  D  E  F  G  H */

const ASCII_SYM: [char; 12] = ['K','Q','N','B','R','P','k','q','n','b','r','p'];
const UNICODE_SYM: [char; 12] = ['♔','♕','♘', '♗','♖','♙','♚','♛','♞','♝','♜','♟'];
const W_QUEEN_SIDE_CASTLE_MASK: BB = BB {data: 0b00000110};
const W_KING_SIDE_CASTLE_MASK:  BB = BB {data: 0b01110000};
const B_QUEEN_SIDE_CASTLE_MASK: BB = BB {data: 0b00000110 << (8*7)};
const B_KING_SIDE_CASTLE_MASK:  BB = BB {data: 0b01110000 << (8*7)};

pub const INITIAL_CHESS_POS: [BB; 12] = [
    BB { data: 0b00000000_00001000}, // ♔
    BB { data: 0b00000000_00010000}, // ♕
    BB { data: 0b00000000_01000010}, // ♘
    BB { data: 0b00000000_00100100}, // ♗
    BB { data: 0b00000000_10000001}, // ♖
    BB { data: 0b11111111_00000000}, // ♙
    BB { data: 0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000}, // ♟
];

pub const fn sym_index(c: char) -> usize {
    match c {
        'K' =>  0,
        'Q' =>  1,
        'N' =>  2,
        'B' =>  3,
        'R' =>  4,
        'P' =>  5,
        'k' =>  6,
        'q' =>  7,
        'n' =>  8,
        'b' =>  9,
        'r' => 10,
        'p' => 11,
        _   => panic!("sym_index error: invalid char!"),
    }
}

pub const fn square_index(square_name: &str) -> usize {
    let mut i: usize = 0;
    while i < 64 {
        let mut j: usize = 0;
        let mut is_match = SQUARE_SYM[i].as_bytes().len() == square_name.as_bytes().len();
        while j < SQUARE_SYM[i].as_bytes().len() {
            if  SQUARE_SYM[i].as_bytes()[j] != square_name.as_bytes()[j]{
                is_match = false;
            }
            j += 1;
        }
        if is_match {
            return i;
        }
        i += 1
    }
    panic!("square_index error: invalid square!");
}

impl ChessBoard {
    pub fn from_fen(input: &str) -> ChessBoard {
        let mut chessboard = ChessBoard {
            piece_bbs: [BB::ZERO; 12], 
            castle_bools: [true; 4], 
            enpassant_bb:BB::ZERO, 
            side_to_move: Side::White, 
            half_move_clock: 0,
            full_move_counter: 0,
        };
        assert!(input.is_ascii());
        let input_vec: Vec<&str> = input.split_ascii_whitespace().collect();
        assert!(input_vec.len() == 6);

        // parse piece placement data
        for i in 0usize..8 {
            let mut j = 0usize;
            while j < 8 {
                let square: usize = 8*i + j;
                for s in input_vec[0].chars() {
                    if s.is_ascii_alphabetic() {
                        chessboard.piece_bbs[sym_index(s)].data |= 1u64 <<square
                    }
                    else if s.is_ascii_digit() {
                        j +=  (s.to_digit(10).unwrap() as usize) - 1;
                        break
                    }
                    else {
                        panic!("from_fen error: invalid char in piece placement portion!")
                    }
                }
                j += 1;
            }
        }
        // parse active colour
        chessboard.side_to_move = match input_vec[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => panic!("from_fen error: invalid active side!"),
        };

        // parse castling information
        for s in input_vec[2].chars() {
            match s {
                '-' => continue,
                'K' => chessboard.castle_bools[0] = true,
                'Q' => chessboard.castle_bools[1] = true,
                'k' => chessboard.castle_bools[2] = true,
                'q' => chessboard.castle_bools[3] = true,
                _ => panic!("from_fen error: invalid castling information!"),
                
            }
        }

        // parse en passant information
        if input_vec[3] != "-" {
            chessboard.enpassant_bb.data |= 1<<square_index(input_vec[3]);
        }
        //parse halfmove clock
        //assert!(input_vec[4].is_ascii_digit()); doesnt work for &str
        chessboard.half_move_clock = input_vec[4].parse::<usize>().unwrap();
        //parse fullmove number
        //assert!(input_vec[5].is_ascii_digit()); doesnt work for &str
        chessboard.full_move_counter = input_vec[5].parse::<usize>().unwrap();
        return chessboard;
    }

    //#[inline(always)]
    pub const fn blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < self.piece_bbs.len() {
            data = data & self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

     pub const fn white_blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < 6{
            data = data & self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

     pub const fn black_blockers(&self) -> BB {
        let mut i = 6;
        let mut data: u64 = 0;
        while i < self.piece_bbs.len() {
            data = data & self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

    //#[inline(always)]
    pub const fn is_square_attacked(&self, square: usize, attacker_side: Side) -> bool {
        assert!(0 < square && square < 64);
        let blockers = self.blockers();
        let w_blockers = self.white_blockers();
        let b_blockers = self.black_blockers();
        match attacker_side {
            Side::White => {
                return (W_PAWN_ATTACKS[square].data & self.piece_bbs[5].data) != 0u64 ||
                    (get_rook_attack(square, blockers).data & self.piece_bbs[4].data) != 0u64 ||
                    (get_bishop_attack(square, blockers).data & self.piece_bbs[3].data) != 0u64 ||
                    (KNIGHT_ATTACKS[square].data & self.piece_bbs[2].data) != 0u64 ||
                    (get_queen_attack(square, blockers).data & self.piece_bbs[1].data) != 0u64 ||
                    (KING_ATTACKS[square].data &  self.piece_bbs[0].data) != 0u64;
            }
            Side::Black => {
                 return (B_PAWN_ATTACKS[square].data & self.piece_bbs[11].data) != 0u64 ||
                    (get_rook_attack(square, blockers).data & self.piece_bbs[10].data) != 0u64 ||
                    (get_bishop_attack(square, blockers).data & self.piece_bbs[9].data) != 0u64 ||
                    (KNIGHT_ATTACKS[square].data & self.piece_bbs[8].data) != 0u64 ||
                    (get_queen_attack(square, blockers).data & self.piece_bbs[7].data) != 0u64 ||
                    (KING_ATTACKS[square].data &  self.piece_bbs[6].data) != 0u64;
            }
            _ => panic!("is_square_attacked error: invalid side!"),
        }
    }
    pub const fn king_is_in_check(&self, king_side: Side) -> bool {
        let mut is_in_check = false;
        match king_side {
            Side::White => {
                assert!(self.piece_bbs[0].data.count_ones() == 1);
                let i: usize = 6;
                while i < 12 {
                    if let Some(square) = self.piece_bbs[0].lsb_index() {
                        is_in_check = self.is_square_attacked(square,Side::Black);
                    }
                    else {
                        panic!("king_is_in_check error: bitboard is zero!")
                    }
                }
            }

            Side::Black => {
                assert!(self.piece_bbs[6].data.count_ones() == 1);
                let i: usize = 0;
                while i < 6 {
                    if let Some(square) = self.piece_bbs[6].lsb_index() {
                        is_in_check = self.is_square_attacked(square,Side::White);
                    }
                    else {
                        panic!("king_is_in_check error: bitboard is zero!")
                    }
                }
            }

            _ => panic!("king_is_in_check error: invalid side!")
        }
        return is_in_check;
    }
    // note: castling rules
    // Neither the king nor the rook has previously moved.
    // There are no pieces between the king and the rook.
    // The king is not currently in check.
    // The king does not pass through or finish on a square that is attacked by an enemy piece.
    pub const fn generate_moves(&self) {
        let mut attacks = BB::ZERO;
        
        let blockers = self.blockers();
        let w_blockers = self.white_blockers();
        let b_blockers = self.black_blockers();
        match self.side_to_move {
            Side::White => {
                let mut i: usize = 0;
                while i < 6 {
                    let mut bitboard = self.piece_bbs[i];
                    match i {
                        00 => { // white king: K
                            // white, king-side castle
                            if self.castle_bools[0] && (blockers.data & W_KING_SIDE_CASTLE_MASK.data == 0) {
                                //check if squares are under attack
                                let mut squares = W_KING_SIDE_CASTLE_MASK;
                                let mut square_not_attacked = true;
                                while squares.data != 0 {
                                    if let Some(index) = squares.lsb_index(){
                                        square_not_attacked = !self.is_square_attacked(index, Side::Black);
                                        squares = squares.pop_bit(index);
                                    }
                                    else {
                                        panic!("generate_moves error: bitboard is zero!");
                                    }
                                }
                                if square_not_attacked && !self.king_is_in_check(Side::White) {
                                    //white king, castle king side
                                }
                            }
                            // white, queen-side castle
                            if self.castle_bools[1] && (blockers.data & W_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                //check if squares are under attack
                                let mut squares = W_QUEEN_SIDE_CASTLE_MASK;
                                let mut square_not_attacked = true;
                                while squares.data != 0 {
                                    if let Some(index) = squares.lsb_index(){
                                        square_not_attacked = !self.is_square_attacked(index, Side::Black);
                                        squares = squares.pop_bit(index);
                                    }
                                    else {
                                        panic!("generate_moves error: bitboard is zero!");
                                    }
                                }
                                if square_not_attacked && !self.king_is_in_check(Side::White) {
                                    //white king, castle queen side
                                }
                            }

                            // white king moves
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[i].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::Black){
                                                if b_blockers.data & 1 << target == 0 {
                                                    // king move
                                                }
                                                else {
                                                    // king captures piece at target square
                                                }
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        01 => { // white queens: Q
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_queen_attack(index, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if b_blockers.data & 1 << target == 0 {
                                                // queen move
                                            }
                                            else {
                                                // queen captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        02 => { // white knights: N
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data = KNIGHT_ATTACKS[index].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if b_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                            }
                                            else {
                                                // knight captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        03 => { // white bishops: B
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_bishop_attack(index, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if b_blockers.data & 1 << target == 0 {
                                                // bishop move
                                            }
                                            else {
                                                // bishop captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        04 => { // white rooks: R
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_rook_attack(index, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if b_blockers.data & 1 << target == 0 {
                                                // rook move
                                            }
                                            else {
                                                // rook captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        05 => { // white pawns: P
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    assert!(index  < 56);
                                    // pawn moves
                                    let target = index + 8;
                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // white pawn at 7th rank
                                        if target >=  56 {
                                            // pawn promotion: queen - Q
                                            // pawn promotion: rook - R
                                            // pawn promotion: bishop - B
                                            // pawn promotion: knight - N
                                        }
                                        else {
                                            // pawn move 1 square
                                        }
                                        // check if pawn is at starting square
                                        if 8 <= index && index <= 15 {
                                            let target = index + 16;
                                            // pawn move 2 squares
                                        }
                                    }
                                    // pawn attacks
                                    let data = W_PAWN_ATTACKS[index].data & self.white_blockers().data;
                                    let mut attacks =  BB {data};
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // white pawn at 7th rank
                                            if target >= 56 {
                                                // pawn captures promotion: queen - Q
                                                // pawn captures promotion: rook - R
                                                // pawn captures promotion: bishop - B
                                                // pawn captures promotion: knight - N
                                            }
                                            else {
                                                // pawn captures 1 ahead
                                            }
                                            
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                        attacks = attacks.pop_bit(target);
                                    }
                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = W_PAWN_ATTACKS[index].data & self.enpassant_bb.data; 
                                        let attacks = BB {data};
                                        if attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // pawn enpassant captures
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                                //
                            }
                        }
                        _ => unreachable!()
                    }
                    i += 1;
                }
                //
            }
            Side::Black => {
                let mut i: usize = 6;
                while i < 12 {
                    let mut bitboard = self.piece_bbs[i];
                    match i {
                        06 => { // black king : k
                            // black, king-side castle
                            if self.castle_bools[2] && (blockers.data & B_KING_SIDE_CASTLE_MASK.data == 0) {
                                //check if squares are under attack
                                let mut squares = B_KING_SIDE_CASTLE_MASK;
                                let mut square_not_attacked = true;
                                while squares.data != 0 {
                                    if let Some(index) = squares.lsb_index(){
                                        square_not_attacked = !self.is_square_attacked(index, Side::White);
                                        squares = squares.pop_bit(index);
                                    }
                                    else {
                                        panic!("generate_moves error: bitboard is zero!");
                                    }
                                }
                                if square_not_attacked && !self.king_is_in_check(Side::Black) { 
                                    //black king, castle king side
                                }
                            }
                            // black, queen-side castle
                            if self.castle_bools[3] && (blockers.data & B_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                //check if squares are under attack
                                let mut squares = B_QUEEN_SIDE_CASTLE_MASK;
                                let mut square_not_attacked = true;
                                while squares.data != 0 {
                                    if let Some(index) = squares.lsb_index(){
                                        square_not_attacked = !self.is_square_attacked(index, Side::White);
                                        squares = squares.pop_bit(index);
                                    }
                                    else {
                                        panic!("generate_moves error: bitboard is zero!");
                                    }
                                }
                                if square_not_attacked && !self.king_is_in_check(Side::Black) {
                                    //black king, castle queen side
                                }
                            }

                            // black king moves
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[i].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::White){
                                                if w_blockers.data & 1 << target == 0 {
                                                    // king move
                                                }
                                                else {
                                                    // king captures piece at target square
                                                }
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        07 => { // black queens: q
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_queen_attack(index, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // queen move
                                            }
                                            else {
                                                // queen captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        08 => { // black knights: n
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data = KNIGHT_ATTACKS[index].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                            }
                                            else {
                                                // knight captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        09 => { // black bishops: b
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_bishop_attack(index, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // bishop move
                                            }
                                            else {
                                                // bishop captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        10 => { // black rooks: 
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    let data: u64 = get_rook_attack(index, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // rook move
                                            }
                                            else {
                                                // rook captures piece at target square
                                            }
                                            attacks = attacks.pop_bit(target);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        11 => { // black pawns : p
                            while bitboard.data != 0 {
                                if let Some(index) = bitboard.lsb_index() {
                                    assert!(index  > 7);
                                    // pawn moves
                                    let target = index - 8;
                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // black pawn at 2nd rank
                                        if target <=  7 {
                                            // pawn promotion: queen - q
                                            // pawn promotion: rook - r
                                            // pawn promotion: bishop - b
                                            // pawn promotion: knight - n
                                        }
                                        else {
                                            // pawn moves 1 square
                                        }
                                        // check if pawn is at starting square
                                        if 48 <= index && index <= 55 {
                                            let target = index - 16;
                                            // pawn move 2 squares
                                        }
                                    }
                                    // pawn attacks
                                    let data = B_PAWN_ATTACKS[index].data & self.white_blockers().data;
                                    let mut attacks =  BB { data };
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // black pawn at 2nd rank
                                            if target <= 7 {
                                                // pawn captures promotion: queen - q
                                                // pawn captures promotion: rook - r
                                                // pawn captures promotion: bishop - b
                                                // pawn captures promotion: knight - n
                                            }
                                            else {
                                                // pawn captures 1 ahead
                                            }
                                            
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                        attacks = attacks.pop_bit(target);
                                    }
                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = B_PAWN_ATTACKS[index].data & self.enpassant_bb.data; 
                                        let attacks = BB {data};
                                        if attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // pawn enpassant captures
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }

                                    bitboard = bitboard.pop_bit(index);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                                //
                            }
                        }
                        _ => unreachable!()
                    }
                    i += 1;
                }
                //
            }
            _ => panic!("generate_moves error: invalid side_to_move!")
        }
    }
}


#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ParseFenError {
    InvalidFen,
    //InvalidBoard,
    //InvalidPocket,
    //InvalidTurn,
    //InvalidCastling,
    //InvalidEpSquare,
    //InvalidRemainingChecks,
    //InvalidHalfmoveClock,
    //InvalidFullmoves,
    //THESE ARE STUFF TODO! FIX FEN PARSING ERROR HANDLING
}

