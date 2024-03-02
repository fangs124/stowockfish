#![allow(dead_code)]
#![allow(long_running_const_eval)]
#![allow(unused_imports)]
use core::panic;
use std::fmt::Display;
use std::ops::Neg;
use std::ops::Not;

use crate::bitboard::*;
use crate::chessmove::*;

// note: castle_bools[] = [white-king  side castle,
//                         white-queen side castle,
//                         black-king  side castle, 
//                         black-queen side castle,
//                        ]
// pieces: white king, queen, knight, bishop, rook, pawn, 
//         black king...

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    pub piece_bbs: [BB;12],
    pub mailbox: [CPT; 64], // apparently this is called a mailbox 
    pub castle_bools: [bool; 4],
    pub enpassant_bb: BB,
    pub check_bb: BB, //piece locations causing the check
    pub side_to_move: Side,
    pub half_move_clock: usize,
    pub full_move_counter: usize,
    pub moves_arr: MovesArray,
}

pub type CB = ChessBoard;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CheckData {
    pub data:(Side,BitBoard),
}

pub type CD = CheckData;

impl Default for ChessBoard {
    fn default() -> Self {
        Self { 
            piece_bbs: INITIAL_CHESS_POS, 
            mailbox: INITIAL_MAILBOX,
            castle_bools      : [true; 4], 
            enpassant_bb      : BB::ZERO, 
            check_bb          : BB::ZERO,
            side_to_move      : Side::White, 
            half_move_clock   : 0,
            full_move_counter : 0,
            moves_arr         : MovesArray::new(),
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
        for i in 1..=64u64 {
            if (1u64 << 64-i) & empty_squares.data != BB::ZERO.data {
                s.push('.' );
                //println!("empty!");
            }
            else {
                let mut j = 0usize;
                while j < self.piece_bbs.len() {
                    let piece_bb: BB = self.piece_bbs[j];
                    if (1u64 << 64-i) & piece_bb.data != BB::ZERO.data {
                        s.push(UNICODE_SYM[j]);
                        //s.push(ASCII_SYM[j]);
                        //println!("piece!");
                    }
                    j+= 1;
                }
            }
            if i % 8 == 0 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

const ASCII_SYM: [char; 12] = ['K','Q','N','B','R','P','k','q','n','b','r','p'];
const UNICODE_SYM: [char; 12] = ['♚','♛','♞','♝','♜','♟','♔','♕','♘', '♗','♖','♙'];
const W_KING_SIDE_CASTLE_MASK: BB = BB {data: 0b00000110};
const W_QUEEN_SIDE_CASTLE_MASK:  BB = BB {data: 0b01110000};
const B_KING_SIDE_CASTLE_MASK: BB = BB {data: 0b00000110 << (8*7)};
const B_QUEEN_SIDE_CASTLE_MASK:  BB = BB {data: 0b01110000 << (8*7)};

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

type CPT = Option<(Side,PieceType)>;

macro_rules! opt_cpt {
    (K) => {Some((Side::White, PieceType::King  ))};
    (Q) => {Some((Side::White, PieceType::Queen ))};
    (N) => {Some((Side::White, PieceType::Knight))};
    (B) => {Some((Side::White, PieceType::Bishop))};
    (R) => {Some((Side::White, PieceType::Rook  ))};
    (P) => {Some((Side::White, PieceType::Pawn  ))};
    (k) => {Some((Side::Black, PieceType::King  ))};
    (q) => {Some((Side::Black, PieceType::Queen ))};
    (n) => {Some((Side::Black, PieceType::Knight))};
    (b) => {Some((Side::Black, PieceType::Bishop))};
    (r) => {Some((Side::Black, PieceType::Rook  ))};
    (p) => {Some((Side::Black, PieceType::Pawn  ))};
    (_) => {None};
}

macro_rules! cpt {
    (K) => {(Side::White, PieceType::King  )};
    (Q) => {(Side::White, PieceType::Queen )};
    (N) => {(Side::White, PieceType::Knight)};
    (B) => {(Side::White, PieceType::Bishop)};
    (R) => {(Side::White, PieceType::Rook  )};
    (P) => {(Side::White, PieceType::Pawn  )};
    (k) => {(Side::Black, PieceType::King  )};
    (q) => {(Side::Black, PieceType::Queen )};
    (n) => {(Side::Black, PieceType::Knight)};
    (b) => {(Side::Black, PieceType::Bishop)};
    (r) => {(Side::Black, PieceType::Rook  )};
    (p) => {(Side::Black, PieceType::Pawn  )};
}

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

pub const INITIAL_MAILBOX: [Option<ColouredPieceType>; 64] = [
    opt_cpt!(R), opt_cpt!(N), opt_cpt!(B), opt_cpt!(K), opt_cpt!(Q), opt_cpt!(B), opt_cpt!(N), opt_cpt!(R),
    opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P),
    opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
    opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
    opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
    opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
    opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p),
    opt_cpt!(r), opt_cpt!(n), opt_cpt!(b), opt_cpt!(k), opt_cpt!(q), opt_cpt!(b), opt_cpt!(n), opt_cpt!(r),
];

// ['K','Q','N','B','R','P','k','q','n','b','r','p'];
type ColouredPieceType = (Side, PieceType);

pub const fn cpt_index(data: ColouredPieceType) -> usize {
    match data {
        cpt!(K) => 00,
        cpt!(Q) => 01,
        cpt!(N) => 02,
        cpt!(B) => 03,
        cpt!(R) => 04,
        cpt!(P) => 05,
        cpt!(k) => 06,
        cpt!(q) => 07,
        cpt!(n) => 08,
        cpt!(b) => 09,
        cpt!(r) => 10,
        cpt!(p) => 11,
    }
}

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
            mailbox: [None; 64], 
            castle_bools: [true; 4], 
            enpassant_bb:BB::ZERO, 
            check_bb: BB::ZERO,
            side_to_move: Side::White, 
            half_move_clock: 0,
            full_move_counter: 0,
            moves_arr: MovesArray::new(),
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
                        //break
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

        //calculate king_is_in_check information.
        assert!(chessboard.piece_bbs[0].data.count_ones() == 1);
        let i = chessboard.piece_bbs[0].lsb_index().unwrap();
        /*
        if chessboard.king_is_in_check(Side::White) {
            //check for knights
            
            //check for any 
        }
        */
        assert!(chessboard.piece_bbs[6].data.count_ones() == 1);
        let i = chessboard.piece_bbs[6].lsb_index().unwrap();
        return chessboard;
    }

    //#[inline(always)]
    pub const fn blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < 12 {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

     pub const fn white_blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < 6 {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

     pub const fn black_blockers(&self) -> BB {
        let mut i = 6;
        let mut data: u64 = 0;
        while i < self.piece_bbs.len() {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB {data};
    }

    //#[inline(always)]
    pub const fn is_square_attacked(&self, square: usize, attacker_side: Side) -> bool {
        assert!(0 < square && square < 64);
        let blockers = self.blockers();
        //let w_blockers = self.white_blockers();
        //let b_blockers = self.black_blockers();
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
        }
    }

    pub const fn piece_is_pinned(&self, square: usize) -> bool {
        //debug asserts
        assert!(square < 64);
        let white_blockers = self.white_blockers();
        let black_blockers = self.black_blockers();
        let blockers = BB {data: white_blockers.data & black_blockers.data };
        if let Some(piece) = self.mailbox[square] {
            //['K','Q','N','B','R','P','k','q','n','b','r','p'];
            match piece {
                cpt!(K)         => panic!("piece_is_pinned error: invalid piece to check!"),
                cpt!(k)         => panic!("piece_is_pinned error: invalid piece to check!"),
                (Side::White,_) => {
                    if self.king_is_in_check(Side::White) {return true;}
                    if let Some(king_pos) = self.piece_bbs[0].lsb_index() {
                        // check if piece is blocking king
                        if get_queen_attack(king_pos, blockers).data & (1u64 << square) != 0 {
                            // check if king is attacked if piece is removed
                            let mut new_chessboard = self.const_clone();
                            new_chessboard.piece_bbs[cpt_index(piece)].data &= !(1u64 << square);
                            new_chessboard.mailbox[cpt_index(piece)] = None;
                            //todo: more efficient check test?
                            if new_chessboard.king_is_in_check(Side::White) {return true};

                        }
                    }
                    else {
                        panic!("piece_is_pinned error: white king not found!")
                    }
                }

                (Side::Black,_) => {
                    if self.king_is_in_check(Side::Black) {return true;}
                    if let Some(king_pos) = self.piece_bbs[6].lsb_index() {
                        // check if piece is blocking king
                        if get_queen_attack(king_pos, blockers).data & (1u64 << square) != 0 {
                            // check if king is attacked if piece is removed
                            let mut new_chessboard = self.const_clone();
                            new_chessboard.piece_bbs[cpt_index(piece)].data &= !(1u64 << square);
                            new_chessboard.mailbox[cpt_index(piece)] = None;
                            //todo: more efficient check test?
                            if new_chessboard.king_is_in_check(Side::White) {return true};
                        }
                    }
                    else {
                        panic!("piece_is_pinned error: black king not found!")
                    }
                }
            }
        }
        else {
            panic!("piece_is_pinned error: square is empty!")
        }
        return false;
    }

    pub const fn king_is_in_check(&self, king_side: Side) -> bool {
        let mut is_in_check = false;
        match king_side {
            Side::White => {
                assert!(self.piece_bbs[0].data.count_ones() == 1);
                let mut i: usize = 6;
                while i < 12 {
                    if let Some(square) = self.piece_bbs[0].lsb_index() {
                        is_in_check = self.is_square_attacked(square,Side::Black);
                    }
                    else {
                        panic!("king_is_in_check error: bitboard is zero!")
                    }
                    i += 1;
                }
            }

            Side::Black => {
                assert!(self.piece_bbs[6].data.count_ones() == 1);
                let mut i: usize = 0;
                while i < 6 {
                    if let Some(square) = self.piece_bbs[6].lsb_index() {
                        is_in_check = self.is_square_attacked(square,Side::White);
                    }
                    else {
                        panic!("king_is_in_check error: bitboard is zero!")
                    }
                    i += 1;
                }
            }
        }
        return is_in_check;
    }

    // note: castling rules
    // Neither the king nor the rook has previously moved.
    // There are no pieces between the king and the rook.
    // The king is not currently in check.
    // The king does not pass through or finish on a square that is attacked by an enemy piece.

    pub fn generate_moves(&mut self) {
        //let mut attacks = BB::ZERO;
        let blockers = self.blockers();
        let w_blockers = self.white_blockers();
        let b_blockers = self.black_blockers();

        //handle king in check situation
        let mut checking_mask = BB::ZERO;
        if self.check_bb.data != 0 {
            let mut checkers = self.check_bb;
            let j = match self.side_to_move {Side::White => 0, Side::Black => 6};
            let k = match self.piece_bbs[j].lsb_index() {
                Some(i) => i,
                None           => panic!("generate_moves error: king not found!"),
            };

            while checkers.data != 0 {
                if let Some(i) = checkers.lsb_index(){
                    if let Some(piece) = self.mailbox[i] {
                        match piece {
                            cpt!(Q) | cpt!(q) => {
                                checking_mask.data |= 
                                    get_queen_attack(i, blockers).data & get_queen_attack(k, blockers).data;
                            }
                            cpt!(N) | cpt!(n) => {
                                checking_mask.data |= 
                                    KNIGHT_ATTACKS[i].data & KNIGHT_ATTACKS[k].data;
                            }
                            cpt!(B) | cpt!(b) => {
                                checking_mask.data |= 
                                    get_bishop_attack(i, blockers).data & get_bishop_attack(k, blockers).data;
                            }
                            cpt!(R) | cpt!(r) => {
                                checking_mask.data |= 
                                    get_rook_attack(i, blockers).data & get_rook_attack(k, blockers).data;
                            }
                            cpt!(P)           => {
                                checking_mask.data |= 
                                    W_PAWN_ATTACKS[i].data & B_PAWN_ATTACKS[k].data;
                            }
                            cpt!(p)           => {
                                checking_mask.data |= 
                                    B_PAWN_ATTACKS[i].data & W_PAWN_ATTACKS[k].data;
                            }
                            _ => panic!("generate_moves error: king is in check by another king!")
                        }
                    }
                    else {
                        panic!("generate_moves error: mailbox is empty!")
                    }
                    checkers = checkers.pop_bit(i);
                }
                else {
                    panic!("generate_moves error: checkers is zero!");
                }
            }
        }

        match self.side_to_move {
            Side::White => {
                let mut i: usize = 0;
                while i < 6 {
                    let mut bitboard = self.piece_bbs[i];
                    match i {
                        00 => { // white king: K
                            // king can only castle whilst not in check
                            if self.check_bb.data == 0 {
                                // white, king-side castle
                                if self.castle_bools[0] && (blockers.data & W_KING_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = W_KING_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index() {
                                            square_not_attacked = !self.is_square_attacked(source, Side::Black);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::White) {
                                        //white king, castle king side
                                        assert!(self.piece_bbs[4].data & 1<<0 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        self.add_move(3, 1, None, MT::Castle);
                                    }
                                }
                                // white, queen-side castle
                                if self.castle_bools[1] && (blockers.data & W_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = W_QUEEN_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index() {
                                            square_not_attacked = !self.is_square_attacked(source, Side::Black);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::White) {
                                        //white king, castle queen side
                                        assert!(self.piece_bbs[4].data & 1<<7 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        self.add_move(3, 5, None, MT::Castle);
                                    }
                                }
                            }

                            // white king moves
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[i].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::Black){
                                                if b_blockers.data & 1 << target == 0 {
                                                    // king move
                                                    self.add_move(source, target, None, MT::Normal)
                                                }
                                                else {
                                                    // king captures piece at target square
                                                    self.add_move(source, target, None, MT::Normal)
                                                }
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        01 => { // white queens: Q
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_queen_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // queen move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // queen captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        02 => { // white knights: N
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data = KNIGHT_ATTACKS[source].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // knight captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        03 => { // white bishops: B
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_bishop_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // bishop move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // bishop captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        04 => { // white rooks: R
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_rook_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // rook move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // rook captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        05 => { // white pawns: P
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    assert!(source  < 56); //unecessary?
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    // pawn moves
                                    let target = source + 8;
                                    let mut one_move_still_in_check = false;
                                    let mut two_move_still_in_check = false;

                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // if king is in check, only consider blocking/capturing moves
                                        let other_target = target + 8;
                                        if self.check_bb.data != 0 {
                                            if (1u64 << target) & checking_mask.data == 0 {
                                                one_move_still_in_check = true;
                                            }
                                            if (1u64 << other_target) & checking_mask.data == 0 {
                                                two_move_still_in_check = true;
                                            }

                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut chessboard = self.const_clone();
                                            let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            chessboard = chessboard.update_state(chess_move);
                                            if chessboard.king_is_in_check(side_in_check) == true {
                                                one_move_still_in_check = true;
                                            }
                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut other_chessboard = self.const_clone();
                                            let other_chess_move = ChessMove::new(source, other_target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            other_chessboard = other_chessboard.update_state(other_chess_move);
                                            if other_chessboard.king_is_in_check(side_in_check) == true {
                                                two_move_still_in_check = true;
                                            }
                                        }

                                        if !one_move_still_in_check {
                                            // white pawn at 7th rank
                                            if target >=  56 {
                                                // pawn promotion: queen - Q
                                                self.add_move(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn promotion: rook - R
                                                self.add_move(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn promotion: bishop - B
                                                self.add_move(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn promotion: knight - N
                                                self.add_move(source, target, Some(PT::Knight), MT::Promotion)
                                            }
                                            else {
                                                // pawn move 1 square
                                                self.add_move(source, target, None, MT::Normal);
                                            }
                                        }

                                        if !two_move_still_in_check {
                                            // check if pawn is at starting square
                                            if 8 <= source && source <= 15 {
                                                let target = source + 16;
                                                // check if target is vacant
                                                if blockers.data & 1 << target == 0 {
                                                    // pawn move 2 squares
                                                    self.add_move(source, target, None, MT::Normal);
                                                }
                                            }
                                        }
                                    }

                                    // pawn attacks
                                    let data = W_PAWN_ATTACKS[source].data & b_blockers.data;
                                    let mut attacks =  BB {data};
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            // white pawn at 7th rank
                                            if target >= 56 {
                                                // pawn captures promotion: queen - Q
                                                self.add_move(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn captures promotion: rook - R
                                                self.add_move(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn captures promotion: bishop - B
                                                self.add_move(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn captures promotion: knight - N
                                                self.add_move(source, target, Some(PT::Knight), MT::Promotion)
                                            }
                                            else {
                                                // pawn captures 1 ahead
                                                self.add_move(source, target, None, MT::Normal);
                                            }
                                            let new_attacks: BitBoard = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                        //attacks.data = attacks.data & BB::nth(target).not().data
                                    }

                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = W_PAWN_ATTACKS[source].data & self.enpassant_bb.data; 
                                        let mut attacks: BitBoard = BB {data};
                                        while attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // if king is in check, only consider blocking/capturing moves
                                                if self.check_bb.data != 0 {
                                                    // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                    let mut chessboard = self.const_clone();
                                                    let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                    let side_in_check = self.side_to_move;
                                                    chessboard = chessboard.update_state(chess_move);
                                                    if chessboard.king_is_in_check(side_in_check) == true {
                                                        let new_attacks = attacks.pop_bit(target);
                                                        assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                        attacks = new_attacks;
                                                        continue;
                                                    }
                                                }
                                                // pawn enpassant captures
                                                self.add_move(source, target, None, MT::EnPassant); //here
                                                let new_attacks = attacks.pop_bit(target);
                                                assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                attacks = new_attacks;
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                                //
                            }
                        }
                        _ => unreachable!() //piece_bb index out of bounds
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
                            // king can only castle whilst not in check
                            if self.check_bb.data == 0 {
                                // black, king-side castle
                                if self.castle_bools[2] && (blockers.data & B_KING_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = B_KING_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index(){
                                            square_not_attacked = !self.is_square_attacked(source, Side::White);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::Black) { 
                                        //black king, castle king side
                                        assert!(self.piece_bbs[10].data & 1<<56 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        self.add_move(59, 57, None, MT::Castle);
                                    }
                                }
                                // black, queen-side castle
                                if self.castle_bools[3] && (blockers.data & B_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = B_QUEEN_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index(){
                                            square_not_attacked = !self.is_square_attacked(source, Side::White);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::Black) {
                                        //black king, castle queen side
                                        assert!(self.piece_bbs[10].data & 1<<63 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        self.add_move(59, 61, None, MT::Castle);
                                    }
                                }
                            }

                            // black king moves
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[i].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::White){
                                                if w_blockers.data & 1 << target == 0 {
                                                    // king move
                                                    self.add_move(source, target, None, MT::Normal)
                                                }
                                                else {
                                                    // king captures piece at target square
                                                    self.add_move(source, target, None, MT::Normal)
                                                }
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        07 => { // black queens: q
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_queen_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // queen move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // queen captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        08 => { // black knights: n
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data = KNIGHT_ATTACKS[source].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // knight captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        09 => { // black bishops: b
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_bishop_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // bishop move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // bishop captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        10 => { // black rooks: 
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_rook_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // rook move
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // rook captures piece at target square
                                                self.add_move(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        11 => { // black pawns : p
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    assert!(source  > 7);
                                    // pawn moves
                                    let target = source - 8;
                                    let mut one_move_still_in_check = false;
                                    let mut two_move_still_in_check = false;

                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // if king is in check, only consider blocking/capturing moves
                                        let other_target = target + 8;
                                        if self.check_bb.data != 0 {
                                            if (1u64 << target) & checking_mask.data == 0 {
                                                one_move_still_in_check = true;
                                            }
                                            if (1u64 << other_target) & checking_mask.data == 0 {
                                                two_move_still_in_check = true;
                                            }

                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut chessboard = self.const_clone();
                                            let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            chessboard = chessboard.update_state(chess_move);
                                            if chessboard.king_is_in_check(side_in_check) == true {
                                                one_move_still_in_check = true;
                                            }
                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut other_chessboard = self.const_clone();
                                            let other_chess_move = ChessMove::new(source, other_target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            other_chessboard = other_chessboard.update_state(other_chess_move);
                                            if other_chessboard.king_is_in_check(side_in_check) == true {
                                                two_move_still_in_check = true;
                                            }
                                        }

                                        if !one_move_still_in_check {
                                            // black pawn at 2nd rank
                                            if target <=  7 {
                                                // pawn promotion: queen - q
                                                self.add_move(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn promotion: rook - r
                                                self.add_move(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn promotion: bishop - b
                                                self.add_move(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn promotion: knight - n
                                                self.add_move(source, target, Some(PT::Knight), MT::Promotion);
                                            }
                                            else {
                                                // pawn moves 1 square
                                                self.add_move(source, target, None, MT::Normal);
                                            }
                                        }

                                        if !two_move_still_in_check {
                                            // check if pawn is at starting square
                                            if 48 <= source && source <= 55 {
                                                let target = source - 16;
                                                // check if target is vacant
                                                if blockers.data & 1 << target == 0 {
                                                    // pawn move 2 squares
                                                    self.add_move(source, target, None, MT::Normal);
                                                }
                                            }
                                        }
                                    }

                                    // pawn attacks
                                    let data = B_PAWN_ATTACKS[source].data & w_blockers.data;
                                    let mut attacks =  BB {data};
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            // black pawn at 2nd rank
                                            if target <= 7 {
                                                // pawn captures promotion: queen - q
                                                self.add_move(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn captures promotion: rook - r
                                                self.add_move(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn captures promotion: bishop - b
                                                self.add_move(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn captures promotion: knight - n
                                                self.add_move(source, target, Some(PT::Knight), MT::Promotion);

                                            }
                                            else {
                                                // pawn captures 1 ahead
                                                self.add_move(source, target, None, MT::Normal);
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    }

                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = B_PAWN_ATTACKS[source].data & self.enpassant_bb.data; 
                                        let mut attacks = BB {data};
                                        while attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // if king is in check, only consider blocking/capturing moves
                                                if self.check_bb.data != 0 {
                                                    // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                    let mut chessboard = self.const_clone();
                                                    let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                    let side_in_check = self.side_to_move;
                                                    chessboard = chessboard.update_state(chess_move);
                                                    if chessboard.king_is_in_check(side_in_check) == true {
                                                        let new_attacks = attacks.pop_bit(target);
                                                        assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                        attacks = new_attacks;
                                                        continue;
                                                    }
                                                }
                                                // pawn enpassant captures
                                                self.add_move(source, target, None, MT::EnPassant);
                                                let new_attacks = attacks.pop_bit(target);
                                                assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                attacks = new_attacks;
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        _ => unreachable!() //piece_bb index out of bounds
                    }
                    i += 1;
                }
            }
        }
    }

    pub const fn generate_moves_array(&self) -> MovesArray {
        //let mut attacks = BB::ZERO;
        let mut moves_array = MovesArray::new();
        let blockers = self.blockers();
        let w_blockers = self.white_blockers();
        let b_blockers = self.black_blockers();

        //handle king in check situation
        let mut checking_mask = BB::ZERO;
        if self.check_bb.data != 0 {
            let mut checkers = self.check_bb;
            let j = match self.side_to_move {Side::White => 0, Side::Black => 6};
            let k = match self.piece_bbs[j].lsb_index() {
                Some(i) => i,
                None           => panic!("generate_moves error: king not found!"),
            };

            while checkers.data != 0 {
                if let Some(i) = checkers.lsb_index(){
                    if let Some(piece) = self.mailbox[i] {
                        match piece {
                            cpt!(Q) | cpt!(q) => {
                                checking_mask.data |= 
                                    get_queen_attack(i, blockers).data & get_queen_attack(k, blockers).data;
                            }
                            cpt!(N) | cpt!(n) => {
                                checking_mask.data |= 
                                    KNIGHT_ATTACKS[i].data & KNIGHT_ATTACKS[k].data;
                            }
                            cpt!(B) | cpt!(b) => {
                                checking_mask.data |= 
                                    get_bishop_attack(i, blockers).data & get_bishop_attack(k, blockers).data;
                            }
                            cpt!(R) | cpt!(r) => {
                                checking_mask.data |= 
                                    get_rook_attack(i, blockers).data & get_rook_attack(k, blockers).data;
                            }
                            cpt!(P)           => {
                                checking_mask.data |= 
                                    W_PAWN_ATTACKS[i].data & B_PAWN_ATTACKS[k].data;
                            }
                            cpt!(p)           => {
                                checking_mask.data |= 
                                    B_PAWN_ATTACKS[i].data & W_PAWN_ATTACKS[k].data;
                            }
                            _ => panic!("generate_moves error: king is in check by another king!")
                        }
                    }
                    else {
                        panic!("generate_moves error: mailbox is empty!")
                    }
                    checkers = checkers.pop_bit(i);
                }
                else {
                    panic!("generate_moves error: checkers is zero!");
                }
            }
        }

        match self.side_to_move {
            Side::White => {
                let mut i: usize = 0;
                while i < 6 {
                    let mut bitboard = self.piece_bbs[i];
                    match i {
                        00 => { // white king: K
                            // king can only castle whilst not in check
                            if self.check_bb.data == 0 {
                                // white, king-side castle
                                if self.castle_bools[0] && (blockers.data & W_KING_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = W_KING_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index() {
                                            square_not_attacked = !self.is_square_attacked(source, Side::Black);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::White) {
                                        //white king, castle king side
                                        assert!(self.piece_bbs[4].data & 1<<0 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        moves_array = moves_array.new_raw(3, 1, None, MT::Castle);
                                    }
                                }
                                // white, queen-side castle
                                if self.castle_bools[1] && (blockers.data & W_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = W_QUEEN_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index(){
                                            square_not_attacked = !self.is_square_attacked(source, Side::Black);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::White) {
                                        //white king, castle queen side
                                        assert!(self.piece_bbs[4].data & 1<<7 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        moves_array = moves_array.new_raw(3, 5, None, MT::Castle);
                                    }
                                }
                            }

                            // white king moves
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[source].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::Black){
                                                if b_blockers.data & 1 << target == 0 {
                                                    // king move
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                                }
                                                else {
                                                    // king captures piece at target square
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                                }
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        01 => { // white queens: Q
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_queen_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // queen move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // queen captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }

                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        02 => { // white knights: N
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data = KNIGHT_ATTACKS[source].data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // knight captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        03 => { // white bishops: B
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_bishop_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // bishop move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // bishop captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        04 => { // white rooks: R
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_rook_attack(source, blockers).data & !w_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            if b_blockers.data & 1 << target == 0 {
                                                // rook move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // rook captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        05 => { // white pawns: P
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    assert!(source  < 56);
                                    // pawn moves
                                    let target = source + 8;
                                    let mut one_move_still_in_check = false;
                                    let mut two_move_still_in_check = false;

                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // if king is in check, only consider blocking/capturing moves
                                        let other_target = target + 8;
                                        if self.check_bb.data != 0 {
                                            if (1u64 << target) & checking_mask.data == 0 {
                                                one_move_still_in_check = true;
                                            }
                                            if (1u64 << other_target) & checking_mask.data == 0 {
                                                two_move_still_in_check = true;
                                            }

                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut chessboard = self.const_clone();
                                            let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            chessboard = chessboard.update_state(chess_move);
                                            if chessboard.king_is_in_check(side_in_check) == true {
                                                one_move_still_in_check = true;
                                            }
                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut other_chessboard = self.const_clone();
                                            let other_chess_move = ChessMove::new(source, other_target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            other_chessboard = other_chessboard.update_state(other_chess_move);
                                            if other_chessboard.king_is_in_check(side_in_check) == true {
                                                two_move_still_in_check = true;
                                            }
                                        }

                                        if !one_move_still_in_check {
                                            // white pawn at 7th rank
                                            if target >=  56 {
                                                // pawn promotion: queen - Q
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn promotion: rook - R
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn promotion: bishop - B
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn promotion: knight - N
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Knight), MT::Promotion)
                                            }
                                            else {
                                                // pawn move 1 square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                            }
                                        }

                                        if !two_move_still_in_check {
                                            // check if pawn is at starting square
                                            if 8 <= source && source <= 15 {
                                                let target = source + 16;
                                                // check if target is vacant
                                                if blockers.data & 1 << target == 0 {
                                                    // pawn move 2 squares
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                                }
                                            }
                                        }
                                    }

                                    // pawn attacks
                                    let data = W_PAWN_ATTACKS[source].data & b_blockers.data;
                                    let mut attacks =  BB {data};
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // if king is in check, only consider blocking/capturing moves
                                            if self.check_bb.data != 0 {
                                                if (1u64 << target) & checking_mask.data == 0 {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                                // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                let mut chessboard = self.const_clone();
                                                let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                let side_in_check = self.side_to_move;
                                                chessboard = chessboard.update_state(chess_move);
                                                if chessboard.king_is_in_check(side_in_check) == true {
                                                    let new_attacks = attacks.pop_bit(target);
                                                    assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                    attacks = new_attacks;
                                                    continue;
                                                }
                                            }
                                            // white pawn at 7th rank
                                            if target >= 56 {
                                                // pawn captures promotion: queen - Q
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn captures promotion: rook - R
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn captures promotion: bishop - B
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn captures promotion: knight - N
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Knight), MT::Promotion)
                                            }
                                            else {
                                                // pawn captures 1 ahead
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    }

                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = W_PAWN_ATTACKS[source].data & self.enpassant_bb.data; 
                                        let mut attacks = BB {data};
                                        while attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // if king is in check, only consider blocking/capturing moves
                                                if self.check_bb.data != 0 {
                                                    // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                    let mut chessboard = self.const_clone();
                                                    let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                    let side_in_check = self.side_to_move;
                                                    chessboard = chessboard.update_state(chess_move);
                                                    if chessboard.king_is_in_check(side_in_check) == true {
                                                        let new_attacks = attacks.pop_bit(target);
                                                        assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                        attacks = new_attacks;
                                                        continue;
                                                    }
                                                }
                                                // pawn enpassant captures
                                                moves_array = moves_array.new_raw(source, target, None, MT::EnPassant);
                                                let new_attacks = attacks.pop_bit(target);
                                                assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                attacks = new_attacks;
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                    i += 1;
                }
            }
            Side::Black => {
                let mut i: usize = 6;
                while i < 12 {
                    let mut bitboard = self.piece_bbs[i];
                    match i {
                        06 => { // black king : k
                            // king can only castle whilst not in check
                            if self.check_bb.data == 0 {
                                // black, king-side castle
                                if self.castle_bools[2] && (blockers.data & B_KING_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = B_KING_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index(){
                                            square_not_attacked = !self.is_square_attacked(source, Side::White);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::Black) { 
                                        //black king, castle king side
                                        assert!(self.piece_bbs[10].data & 1<<56 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        moves_array = moves_array.new_raw(59, 57, None, MT::Castle);
                                    }
                                }
                                // black, queen-side castle
                                if self.castle_bools[3] && (blockers.data & B_QUEEN_SIDE_CASTLE_MASK.data == 0) {
                                    //check if squares are under attack
                                    let mut squares = B_QUEEN_SIDE_CASTLE_MASK;
                                    let mut square_not_attacked = true;
                                    while squares.data != 0 {
                                        if let Some(source) = squares.lsb_index() {
                                            square_not_attacked = !self.is_square_attacked(source, Side::White);
                                            squares = squares.pop_bit(source);
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    if square_not_attacked && !self.king_is_in_check(Side::Black) {
                                        //black king, castle queen side
                                        assert!(self.piece_bbs[10].data & 1<<63 != 0); //might be unecessary
                                        //hard coded castling, bad for chess960 support!
                                        moves_array = moves_array.new_raw(59, 61, None, MT::Castle);
                                    }
                                }
                            }

                            // black king moves
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    let data: u64 = KING_ATTACKS[source].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // king cannot move to an attacked/defended square
                                            if !self.is_square_attacked(target, Side::White){
                                                if w_blockers.data & 1 << target == 0 {
                                                    // king move
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                                }
                                                else {
                                                    // king captures piece at target square
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                                }
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        07 => { // black queens: q
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_queen_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // queen move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // queen captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        08 => { // black knights: n
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data = KNIGHT_ATTACKS[source].data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1u64 << target == 0 {
                                                // knight move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // knight captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    } 
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        09 => { // black bishops: b
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_bishop_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // bishop move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // bishop captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        10 => { // black rooks: 
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    let data: u64 = get_rook_attack(source, blockers).data & !b_blockers.data;
                                    let mut attacks = BB {data};
                                    while attacks.data != 0 {
                                        if let Some(target) = attacks.lsb_index() {
                                            if w_blockers.data & 1 << target == 0 {
                                                // rook move
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            else {
                                                // rook captures piece at target square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal)
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: bitboard is zero!");
                                        }
                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                            }
                        }
                        11 => { // black pawns : p
                            while bitboard.data != 0 {
                                if let Some(source) = bitboard.lsb_index() {
                                    if self.piece_is_pinned(source) {bitboard = bitboard.pop_bit(source); continue};
                                    assert!(source  > 7);
                                    // pawn moves
                                    let target = source - 8;
                                    let mut one_move_still_in_check = false;
                                    let mut two_move_still_in_check = false;

                                    // one square ahead is not blocked
                                    if self.blockers().get_bit(target).data == 0 {
                                        // if king is in check, only consider blocking/capturing moves
                                        let other_target = target + 8;
                                        if self.check_bb.data != 0 {
                                            if (1u64 << target) & checking_mask.data == 0 {
                                                one_move_still_in_check = true;
                                            }
                                            if (1u64 << other_target) & checking_mask.data == 0 {
                                                two_move_still_in_check = true;
                                            }

                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut chessboard = self.const_clone();
                                            let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            chessboard = chessboard.update_state(chess_move);
                                            if chessboard.king_is_in_check(side_in_check) == true {
                                                one_move_still_in_check = true;
                                            }
                                            // make sure king is no longer in check if move happens //note:probably expensive!!!
                                            let mut other_chessboard = self.const_clone();
                                            let other_chess_move = ChessMove::new(source, other_target,None,MT::Normal);
                                            let side_in_check = self.side_to_move;
                                            other_chessboard = other_chessboard.update_state(other_chess_move);
                                            if other_chessboard.king_is_in_check(side_in_check) == true {
                                                two_move_still_in_check = true;
                                            }
                                        }

                                        if !one_move_still_in_check {
                                            // black pawn at 2nd rank
                                            if target <=  7 {
                                                // pawn promotion: queen - q
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn promotion: rook - r
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn promotion: bishop - b
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn promotion: knight - n
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Knight), MT::Promotion);
                                            }
                                            else {
                                                // pawn moves 1 square
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                            }
                                        }

                                        if !two_move_still_in_check {
                                            // check if pawn is at starting square
                                            if 48 <= source && source <= 55 {
                                                let target = source - 16;
                                                // if king is in check, only consider blocking/capturing moves
                                                if self.check_bb.data != 0 {
                                                    if (1u64 << target) & checking_mask.data == 0 {
                                                        continue;
                                                    }
                                                    // make sure king is no longer in check if move happens //note:probably expensive!!!
                                                    let mut chessboard = self.const_clone();
                                                    let chess_move = ChessMove::new(source,target,None,MT::Normal);
                                                    let side_in_check = self.side_to_move;
                                                    chessboard = chessboard.update_state(chess_move);
                                                    if chessboard.king_is_in_check(side_in_check) == true {
                                                        continue;
                                                    }
                                                }
                                                // check if target is vacant
                                                if blockers.data & 1 << target == 0 {
                                                    // pawn move 2 squares
                                                    moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                                }
                                            }
                                        }
                                    }

                                    // pawn attacks
                                    let data = B_PAWN_ATTACKS[source].data & w_blockers.data;
                                    let mut attacks =  BB { data };
                                    while attacks.data != 0u64 {
                                        if let Some(target) = attacks.lsb_index() {
                                            // black pawn at 2nd rank
                                            if target <= 7 {
                                                // pawn captures promotion: queen - q
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Queen), MT::Promotion);
                                                // pawn captures promotion: rook - r
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Rook), MT::Promotion);
                                                // pawn captures promotion: bishop - b
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Bishop), MT::Promotion);
                                                // pawn captures promotion: knight - n
                                                moves_array = moves_array.new_raw(source, target, Some(PT::Knight), MT::Promotion);

                                            }
                                            else {
                                                // pawn captures 1 ahead
                                                moves_array = moves_array.new_raw(source, target, None, MT::Normal);
                                            }
                                            let new_attacks = attacks.pop_bit(target);
                                            assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                            attacks = new_attacks;
                                        }
                                        else {
                                            panic!("generate_moves error: attacks is zero!");
                                        }
                                    }

                                    // pawn enpassant
                                    if self.enpassant_bb.data != 0u64 {
                                        let data = B_PAWN_ATTACKS[source].data & self.enpassant_bb.data; 
                                        let mut attacks = BB {data};
                                        while attacks.data != 0u64 {
                                            if let Some(target) = attacks.lsb_index() {
                                                // pawn enpassant captures
                                                moves_array = moves_array.new_raw(source, target, None, MT::EnPassant);
                                                let new_attacks = attacks.pop_bit(target);
                                                assert!(attacks.data.count_ones() - new_attacks.data.count_ones() == 1);
                                                attacks = new_attacks;
                                            }
                                            else {
                                                panic!("generate_moves error: attacks is zero!");
                                            }
                                        }

                                    }
                                    bitboard = bitboard.pop_bit(source);
                                }
                                else {
                                    panic!("generate_moves error: bitboard is zero!");
                                }
                                //
                            }
                        }
                        _ => panic!()
                    }
                    i += 1;
                }
                //
            }
        }
        return moves_array;
    }


    pub fn add_move(&mut self, source: usize, target: usize, piece_data: Option<PieceType>, move_type: MoveType) {
        self.moves_arr.push(ChessMove::new(source,target, piece_data, move_type));
    }

    pub const fn is_capture_move(&self, chess_move : ChessMove) -> bool {
        match self.side_to_move {
            Side::White => {
                return (self.black_blockers().data & 1 << chess_move.get_target_index()) != 0;
            }
            Side::Black => {
                return (self.white_blockers().data & 1 << chess_move.get_target_index()) != 0;
            }
        }
    }

    pub const fn const_clone(&self) -> ChessBoard {
        ChessBoard {
            piece_bbs        : self.piece_bbs,
            mailbox          : self.mailbox,
            castle_bools     : self.castle_bools,
            enpassant_bb     : self.enpassant_bb,
            side_to_move     : self.side_to_move,
            half_move_clock  : self.half_move_clock,
            full_move_counter: self.full_move_counter,
            moves_arr        : self.moves_arr.const_clone(),
            check_bb    : self.check_bb,
        }
    }
    //pub const fn Mathemagician(mut self, chess_move : ChessMove) -> Self {self.check_bb = BB{data:u64::MAX}; self}
    pub const fn update_state(&self, chess_move : ChessMove) -> ChessBoard {
        let mut chess_board = self.const_clone();
        let mut enpassant_bb:BitBoard = BB::ZERO;
        let source: usize = chess_move.source();
        let target: usize = chess_move.target();       
        // handle special cases
        match chess_board.mailbox[source] {
            opt_cpt!(_) => panic!("update_state error: source mailbox is None!"),
            
            /* special case: castling rights */ 
            opt_cpt!(K) => {
                chess_board.castle_bools[0] = false;
                chess_board.castle_bools[1] = false;
            }
            opt_cpt!(R) => {
                if source == 0 {
                    chess_board.castle_bools[0] = false;
                }
                else if source == 7 {
                    chess_board.castle_bools[1] = false
                }
            }
            opt_cpt!(k) => {
                chess_board.castle_bools[2] = false;
                chess_board.castle_bools[3] = false;
            }
            opt_cpt!(r) => {
                if source == 56 {
                    chess_board.castle_bools[2] = false;
                }
                else if source == 63 {
                    chess_board.castle_bools[3] = false
                }
            }
            /* special case: pawn 2-squares move, en passant rules */ 
            opt_cpt!(P) => {
                // check if move is 2-square
                if source + 16 == target {
                    if target + 1 < 64 {
                        // check pawn lands next to enemy pawn
                        match chess_board.mailbox[target + 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chess_board.piece_is_pinned(target + 1) {
                                    enpassant_bb.data &= 1 << target - 8
                                }
                            }
                            _______ => {}
                        }
                    }

                    if 0 + 1 <= target { // unsigned hack: 0 <= target - 1
                        // check pawn lands next to enemy pawn
                        match chess_board.mailbox[target - 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chess_board.piece_is_pinned(target - 1) {
                                    enpassant_bb.data &= 1 << target - 8
                                }
                            }
                            _______ => {}
                        }
                    }
                }
            }
            opt_cpt!(p) => {
                if source == target + 16 { // unsinged hack: source - 16 == target
                    if target + 1 < 64 {
                        // check pawn lands next to enemy pawn
                        match chess_board.mailbox[target + 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chess_board.piece_is_pinned(target + 1) {
                                    enpassant_bb.data &= 1 << target + 8
                                }
                            }
                            _______ => {}
                        }
                    }
                    
                    if 0 + 1 <= target { // unsigned hack: 0 <= target - 1
                        // check pawn lands next to enemy pawn
                        match chess_board.mailbox[target - 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chess_board.piece_is_pinned(target - 1) {
                                    enpassant_bb.data &= 1 << target + 8
                                }
                            }
                            _______ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        // update piece_bbs and mailbox
        match chess_move.get_move_type() {
            MoveType::Normal => {
                if let Some(source_data) = chess_board.mailbox[source] {
                    //if source is a pawn and move is two-squares, encode enpassant data
                    match chess_board.mailbox[source] {
                        Some(cpt!(P)) => {
                            if source + 16 == target {
                                enpassant_bb.data |= 1 << (target - 8);
                            }
                        },
                        Some(cpt!(p)) => {
                            if source == target + 16 { //source - 16 == target
                                enpassant_bb.data |= 1 << (target + 8);
                            }
                        },
                        _____________ => {},
                    }

                    // set bit of source to zero
                    chess_board.piece_bbs[cpt_index(source_data)].data &= !(1 << source);
                    // set bit of target to one
                    chess_board.piece_bbs[cpt_index(source_data)].data |=  (1 << target);

                    // if target is occupied, deal with piece capture
                    if let Some(target_data) = chess_board.mailbox[target] {
                        chess_board.piece_bbs[cpt_index(target_data)].data &= !(1 << target);
                        match target_data {
                            // how to handle checks/king captures??
                            /*cpt!(K) => {
                                self.castle_bools[0] = false;
                                self.castle_bools[1] = false;
                            }*/
                            cpt!(R) => {
                                if source == 0 {
                                    chess_board.castle_bools[0] = false;
                                }
                                else if source == 7 {
                                    chess_board.castle_bools[1] = false
                                }
                            }
                          
                            cpt!(r) => {
                                if source == 56 {
                                    chess_board.castle_bools[2] = false;
                                }
                                else if source == 63 {
                                    chess_board.castle_bools[3] = false
                                }
                            }
                            _ => {}
                        }
                    }


                    // set source mailbox to None
                    chess_board.mailbox[source] = None;
                    // set target mailbox to new ColouredPieceType
                    chess_board.mailbox[target] = Some(source_data);
                }
                else {
                    panic!("update_state error: source mailbox is None!")
                }
            }

            MoveType::Castle => {
                if let Some(data_source) = chess_board.mailbox[source] {
                    // set bit of source to zero
                    chess_board.piece_bbs[cpt_index(data_source)].data &= !(1 << source);
                    // set bit of target to one
                    chess_board.piece_bbs[cpt_index(data_source)].data |= (1 << target);

                    // set source mailbox to None
                    chess_board.mailbox[source] = None;
                    // set target mailbox to new ColouredPieceType
                    chess_board.mailbox[target] = Some(data_source);

                    // deal with rook movement
                    match self.side_to_move {
                        Side::White => {
                            // king-side castle
                            if target == 1 {
                                // check if rook is present
                                assert!(self.piece_bbs[04].data & 1 << 00 != 0);
                                // ?? check if rook target square is empty
                                // assert!(...)
                                
                                chess_board.piece_bbs[04].data &= !1 << 00;
                                chess_board.piece_bbs[04].data |=  1 << 02;
                                chess_board.mailbox[00] = None;
                                chess_board.mailbox[02] = opt_cpt!(R)
                            }
                            // queen-side castle
                            else if target == 5 {
                                // check if rook is present
                                assert!(self.piece_bbs[04].data & 1 << 07 != 0);
                                // ?? check if rook target square is empty
                                // assert!(...)

                                chess_board.piece_bbs[04].data &= !1 << 07;
                                chess_board.piece_bbs[04].data |=  1 << 04;
                                chess_board.mailbox[04] = None;
                                chess_board.mailbox[02] = opt_cpt!(R)
                            }
                        },
                        Side::Black => {
                            // king-side castle
                            if target == 57 {
                                // check if rook is present
                                assert!(self.piece_bbs[10].data & 1 << 56 != 0);
                                // ?? check if rook target square is empty
                                // assert!(...)

                                chess_board.piece_bbs[10].data &= !1 << 56;
                                chess_board.piece_bbs[10].data |=  1 << 58;
                                chess_board.mailbox[56] = None;
                                chess_board.mailbox[58] = opt_cpt!(r)
                            }
                            // queen-side castle
                            else if target == 61 {
                                // check if rook is present
                                assert!(self.piece_bbs[10].data & 1 << 63 != 0);
                                // ?? check if rook target square is empty
                                // assert!(...)

                                chess_board.piece_bbs[10].data &= !1 << 63;
                                chess_board.piece_bbs[10].data |=  1 << 60;
                                chess_board.mailbox[56] = None;
                                chess_board.mailbox[58] = opt_cpt!(r)
                            }
                        },
                    }
                }
                else {
                    panic!("update_state error: source mailbox is None!")
                }
            }

            MoveType::EnPassant => {
                // note: target is where the capturing pawn will end up
                //       square is where the pawn to be captured is 
                if let Some(data_source) = chess_board.mailbox[source] {
                    // set bit of source to zero
                    chess_board.piece_bbs[cpt_index(data_source)].data &= !(1 << source);
                    // set bit of target to one
                    chess_board.piece_bbs[cpt_index(data_source)].data |= 1 << target;
                    let i = match self.side_to_move {
                        Side::White => 11,
                        Side::Black => 05,
                    };

                    let square = match self.side_to_move {
                        Side::White => target - 8,
                        Side::Black => target + 8,
                    };
                    
                    //check presence of pawn to be captured
                    assert!(chess_board.piece_bbs[i].data & (1 << square) != 0);

                   
                    // assert!(chess_board.mailbox[square] == Some(relevant_piece));
                    if let Some(piece) = chess_board.mailbox[square] { //note: assert hack
                        match self.side_to_move {
                            Side::White => { match piece {
                                cpt!(p) => {}, 
                                _ => panic!("update_state error: square mailbox is not pawn, en_passant case!")
                            }}
                            Side::Black => { match piece {
                                cpt!(P) => {},
                                _ => panic!("update_state error: square mailbox is not pawn, en_passant case!")
                            }}
                        }
                    } else {panic!("update_state error: square mailbox is None when move is an en_passant!")}

                    // deal with piece capture
                    if let Some(data_square) = chess_board.mailbox[square] {
                        let j = cpt_index(data_square);
                        chess_board.piece_bbs[j] =chess_board.piece_bbs[j].pop_bit(square);
                    }
                    else {
                        panic!("update_state error: target mailbox is None when move is an en_passant!");
                    }

                    // set source mailbox to None
                    chess_board.mailbox[source] = None;
                    // set target mailbox to new ColouredPieceType
                    chess_board.mailbox[target] = Some(data_source);
                    // set en_passant square mailbox to None
                    chess_board.mailbox[square] = None;
                }
                else {
                    panic!("update_state error: source mailbox is None!")
                }
            }

            MoveType::Promotion => {
                if let Some(data_source) = chess_board.mailbox[source] {
                    // set bit of source to zero
                    chess_board.piece_bbs[cpt_index(data_source)].data &= !(1 << source);
                    
                    if let Some(piece_type) = chess_move.get_piece_data() {
                        let new_cpt = (data_source.0, piece_type);
                        // set bit of target to one
                        chess_board.piece_bbs[cpt_index(new_cpt)].data &= (1 << target);
                        
                        // if target is occupied, deal with piece capture
                        if let Some(data_target) = chess_board.mailbox[target] {
                            chess_board.piece_bbs[cpt_index(data_target)].data &= !(1 << target);
                        }
                        
                        // set source mailbox to None
                        chess_board.mailbox[source] = None;
                        // set target mailbox to new ColouredPieceType
                        chess_board.mailbox[target] = Some(new_cpt);

                    } else {panic!("update_state error: chess_move is a promotion with None piece data!");}
                }
                else {panic!("update_state error: source mailbox is None!")}
            }
        }

        chess_board.enpassant_bb = enpassant_bb;
        
        //note: potential bug here
        // check if king is in check in new state, return old position otherwise //old idea, its dead
        // if chess_board.king_is_in_check(chess_board.side_to_move) {return self.const_clone()}

        
        match chess_board.side_to_move {
            Side::Black => chess_board.full_move_counter += 1,
            _____ => {},
        }
        chess_board.side_to_move = chess_board.side_to_move.update();
        chess_board.half_move_clock += 1;
        chess_board.moves_arr = MovesArray::new();
        
        // ['K','Q','N','B','R','P','k','q','n','b','r','p'];
        //check if move results in opponent's king to be in check
        match chess_board.king_is_in_check(chess_board.side_to_move) {
            true => {match chess_board.side_to_move{
                Side::White => {
                    let blockers = chess_board.blockers();
                    if let Some(king_pos) = chess_board.piece_bbs[0].lsb_index(){
                        let mut check_bitboard = BB::ZERO;
                        //q
                        check_bitboard.data |= chess_board.piece_bbs[07].data & get_queen_attack(king_pos, blockers).data;
                        //n
                        check_bitboard.data |= chess_board.piece_bbs[08].data & KNIGHT_ATTACKS[king_pos].data;
                        //b
                        check_bitboard.data |= chess_board.piece_bbs[09].data & get_bishop_attack(king_pos, blockers).data;
                        //r
                        check_bitboard.data |= chess_board.piece_bbs[10].data & get_rook_attack(king_pos, blockers).data;
                        //p
                        check_bitboard.data |= chess_board.piece_bbs[11].data & W_PAWN_ATTACKS[king_pos].data;
                        chess_board.check_bb = check_bitboard;
                    }
                    else {
                        panic!("update_state error: white king bitboard is empty!");
                    }
                }

                Side::Black => {
                    let blockers = chess_board.blockers();
                    if let Some(king_pos) = chess_board.piece_bbs[0].lsb_index(){
                        let mut check_bitboard = BB::ZERO;
                        //Q
                        check_bitboard.data |= chess_board.piece_bbs[01].data & get_queen_attack(king_pos, blockers).data;
                        //N
                        check_bitboard.data |= chess_board.piece_bbs[02].data & KNIGHT_ATTACKS[king_pos].data;
                        //B
                        check_bitboard.data |= chess_board.piece_bbs[03].data & get_bishop_attack(king_pos, blockers).data;
                        //R
                        check_bitboard.data |= chess_board.piece_bbs[04].data & get_rook_attack(king_pos, blockers).data;
                        //P
                        check_bitboard.data |= chess_board.piece_bbs[05].data & B_PAWN_ATTACKS[king_pos].data;
                        chess_board.check_bb = check_bitboard;
                    }
                    else {
                        panic!("update_state error: white king bitboard is empty!");
                    }
                }
            }}
            false => {
                chess_board.check_bb = BB::ZERO;
            }
        }
        return chess_board;
    }

        
    pub fn perft_count(&self, depth: usize) -> u64 {
        //debug
        //println!("{}",self);

        if depth == 0 {
            return 1;
        }
        
        let moves_array = self.generate_moves_array();//todo!!!!
        let mut i: usize = 0;
        let mut total: u64 = 0;
        while i < moves_array.len() {
            if let Some(chess_move) = moves_array.data()[i] {
                let chess_board = self.update_state(chess_move);
                total += chess_board.perft_count(depth - 1);
            }
            else {
                panic!("perft_count error: chess_move is None!");
            }
            i += 1;
        }

        return total;
}
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

