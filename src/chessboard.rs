#![allow(dead_code)]
#![allow(long_running_const_eval)]

use core::panic;
use std::fmt::Display;

use crate::bitboard::*;
use crate::chessboard;
use crate::chessmove::*;

// note: castle_bools[] = [white-king  side castle,
//                         white-queen side castle,
//                         black-king  side castle,
//                         black-queen side castle]
//
// pieces: white king, white queen, white knight, white bishop, white rook, white pawn,
//         black king, black queen, black knight, black bishop, black rook, black pawn,

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    pub piece_bbs: [BB; 12],
    pub mailbox: [CPT; 64],
    pub castle_bools: [bool; 4],
    pub enpassant_bb: BB,
    pub check_bb: BB, //piece locations causing the check
    pub side_to_move: Side,
    pub half_move_clock: usize,
    pub full_move_counter: usize,
}

pub type CB = ChessBoard;

impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            piece_bbs: INITIAL_CHESS_POS,
            mailbox: INITIAL_MAILBOX,
            castle_bools: [true; 4],
            enpassant_bb: BB::ZERO,
            check_bb: BB::ZERO,
            side_to_move: Side::White,
            half_move_clock: 0,
            full_move_counter: 0,
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

        // append characters according to piece
        for i in 1..=64u64 {
            if (1u64 << 64 - i) & empty_squares.data != BB::ZERO.data {
                s.push('.');
            } else {
                let mut j = 0usize;
                while j < self.piece_bbs.len() {
                    let piece_bb: BB = self.piece_bbs[j];
                    if (1u64 << 64 - i) & piece_bb.data != BB::ZERO.data {
                        s.push(UNICODE_SYM[j]);
                        //s.push(ASCII_SYM[j]);
                    }
                    j += 1;
                }
            }
            if i % 8 == 0 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

const ASCII_SYM: [char; 12] = ['K', 'Q', 'N', 'B', 'R', 'P', 'k', 'q', 'n', 'b', 'r', 'p'];
const UNICODE_SYM: [char; 12] = ['♚', '♛', '♞', '♝', '♜', '♟', '♔', '♕', '♘', '♗', '♖', '♙'];
const W_KING_SIDE_CASTLE_MASK: BB = BB { data: 0b00000110 };
const W_QUEEN_SIDE_CASTLE_MASK: BB = BB { data: 0b01110000 };
const B_KING_SIDE_CASTLE_MASK: BB =
    BB { data: 0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000 };
const B_QUEEN_SIDE_CASTLE_MASK: BB =
    BB { data: 0b01110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000 };


#[rustfmt::skip]
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

type CPT = Option<(Side, PieceType)>;

#[rustfmt::skip]
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

#[rustfmt::skip]
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

#[rustfmt::skip]
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

pub const fn generate_mailbox(bbs: [BitBoard; 12]) -> [Option<ColouredPieceType>; 64] {
    let mut mailbox: [Option<ColouredPieceType>; 64] = [None; 64];
    let mut i: usize = 0;
    while i < 12 {
        let mut j: usize = 0;
        while j < 64 {
            if bbs[i].data & (1u64 << j) != 0 {
                mailbox[j] = match i {
                    00 => opt_cpt!(K),
                    01 => opt_cpt!(Q),
                    02 => opt_cpt!(N),
                    03 => opt_cpt!(B),
                    04 => opt_cpt!(R),
                    05 => opt_cpt!(P),
                    06 => opt_cpt!(k),
                    07 => opt_cpt!(q),
                    08 => opt_cpt!(n),
                    09 => opt_cpt!(b),
                    10 => opt_cpt!(r),
                    11 => opt_cpt!(p),
                    __ => unreachable!(),
                };
            }
            j += 1;
        }
        i += 1;
    }
    mailbox
}

// ['K','Q','N','B','R','P','k','q','n','b','r','p'];
type ColouredPieceType = (Side, PieceType);

#[rustfmt::skip]
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

#[rustfmt::skip]
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
            if SQUARE_SYM[i].as_bytes()[j] != square_name.as_bytes()[j] {
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
    pub const fn blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < 12 {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB { data };
    }

    pub const fn white_blockers(&self) -> BB {
        let mut i = 0;
        let mut data: u64 = 0;
        while i < 6 {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB { data };
    }

    pub const fn black_blockers(&self) -> BB {
        let mut i = 6;
        let mut data: u64 = 0;
        while i < self.piece_bbs.len() {
            data = data | self.piece_bbs[i].data;
            i += 1;
        }
        return BB { data };
    }

    pub const fn is_square_attacked(&self, square: usize, attacker_side: Side) -> bool {
        assert!(square < 64);
        let blockers = self.blockers();
        match attacker_side {
            Side::White => {
                return (B_PAWN_ATTACKS[square].data & self.piece_bbs[5].data) != 0u64
                    || (get_rook_attack(square, blockers).data & self.piece_bbs[4].data) != 0u64
                    || (get_bishop_attack(square, blockers).data & self.piece_bbs[3].data) != 0u64
                    || (KNIGHT_ATTACKS[square].data & self.piece_bbs[2].data) != 0u64
                    || (get_queen_attack(square, blockers).data & self.piece_bbs[1].data) != 0u64
                    || (KING_ATTACKS[square].data & self.piece_bbs[0].data) != 0u64;
            }
            Side::Black => {
                return (W_PAWN_ATTACKS[square].data & self.piece_bbs[11].data) != 0u64
                    || (get_rook_attack(square, blockers).data & self.piece_bbs[10].data) != 0u64
                    || (get_bishop_attack(square, blockers).data & self.piece_bbs[9].data) != 0u64
                    || (KNIGHT_ATTACKS[square].data & self.piece_bbs[8].data) != 0u64
                    || (get_queen_attack(square, blockers).data & self.piece_bbs[7].data) != 0u64
                    || (KING_ATTACKS[square].data & self.piece_bbs[6].data) != 0u64;
            }
        }
    }

    //['K','Q','N','B','R','P','k','q','n','b','r','p'];
    // note: might be slow
    pub const fn piece_is_pinned(&self, square: usize) -> bool {
        assert!(square < 64);
        let mut chessboard = self.const_clone();
        let piece = match self.mailbox[square] {
            Some(p) => p,
            None => {
                //debug
                //println!("========================");
                //println!("square:\n{}", BB { data: (1u64 << square) });
                //println!("========================");
                //println!("chessboard:\n{}", chessboard);
                //println!("========================");
                //println!("mailbox:\n{}", print_mailbox(chessboard.mailbox));
                //println!("========================");
                panic!("piece_is_pinned error: square is empty!");
            }
        };
        chessboard.piece_bbs[cpt_index(piece)].data &= !(1u64 << square);
        chessboard.mailbox[cpt_index(piece)] = None;
        let side = self.side_to_move;

        // assertion hack
        match piece {
            cpt!(K) | cpt!(k) => panic!("piece_is_pinned error: invalid piece to check!"),
            _ => {}
        }
        // if king is not in check, test if removing piece causes king to be in check
        if !self.king_is_in_check(side) {
            return chessboard.king_is_in_check(side);
        } else {
            //note: THIS BIT IS SLOW!!!
            let (q_index, b_index, r_index) = match side {
                Side::White => (07, 09, 10),
                Side::Black => (01, 03, 04),
            };

            let d_data = self.piece_bbs[q_index].data | self.piece_bbs[b_index].data;
            let l_data = self.piece_bbs[q_index].data | self.piece_bbs[r_index].data;
            let diagonals = BB { data: d_data };
            let laterals = BB { data: l_data };

            let enemies = match side {
                Side::White => self.black_blockers(),
                Side::Black => self.white_blockers(),
            };

            assert!(
                self.piece_bbs[0].data.count_ones() == 1
                    && self.piece_bbs[6].data.count_ones() == 1
            );
            let king_pos: usize = match side {
                Side::White => match self.piece_bbs[0].lsb_index() {
                    Some(x) => x,
                    None => unreachable!(),
                },
                Side::Black => match self.piece_bbs[6].lsb_index() {
                    Some(x) => x,
                    None => unreachable!(),
                },
            };
            let removed_blockers = BB { data: self.blockers().data & !(1u64 << square) };
            let data = enemies.data
                & ((get_bishop_attack(king_pos, removed_blockers).data & diagonals.data)
                    | (get_rook_attack(king_pos, removed_blockers).data & laterals.data));
            let mut potential_pinners: BitBoard = BB { data };

            while potential_pinners.data != 0 {
                let potential_pinner = match potential_pinners.lsb_index() {
                    Some(x) => x,
                    None => unreachable!(),
                };
                // check if piece is between king and potential_pinner
                if RAYS[king_pos][potential_pinner].data & (1u64 << square) != 0 {
                    return true;
                }
                potential_pinners = potential_pinners.pop_bit(potential_pinner);
            }
        }
        return false;
    }

    pub const fn king_is_in_check(&self, king_side: Side) -> bool {
        let i = match king_side {
            Side::White => 0,
            Side::Black => 6,
        };
        let square = match self.piece_bbs[i].lsb_index() {
            Some(x) => x,
            None => panic!("king_is_in_check error: king not found!"),
        };
        self.is_square_attacked(square, self.side_to_move.update())
    }

    pub const fn const_clone(&self) -> ChessBoard {
        ChessBoard {
            piece_bbs: self.piece_bbs,
            mailbox: self.mailbox,
            castle_bools: self.castle_bools,
            enpassant_bb: self.enpassant_bb,
            side_to_move: self.side_to_move,
            half_move_clock: self.half_move_clock,
            full_move_counter: self.full_move_counter,
            check_bb: self.check_bb,
        }
    }

    pub fn perft_count(&self, depth: usize) -> u64 {
        if depth == 0 {
            // this is used when printing the individual moves in a given position
            return 1;
        }

        let arr = self.generate_moves();
        if depth == 1 {
            return arr.len() as u64;
        }
        let mut i: usize = 0;
        let mut total: u64 = 0;
        while i < arr.len() {
            if let Some(chess_move) = arr.data()[i] {
                let chessboard = self.update_state(chess_move);
                total += chessboard.perft_count(depth - 1);
            } else {
                panic!("perft_count error: chess_move is None!");
            }
            i += 1;
        }

        return total;
    }

    //pub const fn generate_moves(&self) -> MovesArray {
    pub const fn generate_moves(&self) -> MovesArray {
        assert!(
            self.piece_bbs[0].data.count_ones() == 1 && self.piece_bbs[6].data.count_ones() == 1
        );
        let mut arr = MovesArray::new();
        let blockers = self.blockers();
        let w_blockers = self.white_blockers();
        let b_blockers = self.black_blockers();
        let side = self.side_to_move;
        let king_pos = match side {
            Side::White => match self.piece_bbs[0].lsb_index() {
                Some(x) => x,
                None => unreachable!(),
            },
            Side::Black => match self.piece_bbs[6].lsb_index() {
                Some(x) => x,
                None => unreachable!(),
            },
        };

        let enemies = match side {
            Side::White => b_blockers,
            Side::Black => w_blockers,
        };

        let friends = match side {
            Side::White => w_blockers,
            Side::Black => b_blockers,
        };

        // consider if king is in check
        let mut check_mask: BitBoard = self.check_bb;
        let checkers_count = self.check_bb.data.count_ones();
        if self.check_bb.data != 0 {
            let mut checkers = self.check_bb;
            let index: usize = match side {
                Side::White => 0,
                Side::Black => 6,
            };

            let k: usize = match self.piece_bbs[index].lsb_index() {
                Some(x) => x,
                None => panic!("generate_moves error: king not found!"),
            };

            //debug
            //println!("checkers:");
            //println!("{}", checkers);
            //println!("king_pos:");
            //println!("{}", BB{data:(1u64 << king_pos)});
            while checkers.data != 0 {
                let i: usize = match checkers.lsb_index() {
                    Some(x) => x,
                    None => unreachable!(),
                };

                if let Some(piece) = self.mailbox[i] {
                    match piece {
                        cpt!(K) | cpt!(k) => {
                            panic!("generate_moves error: king is in check by another king!")
                        }
                        cpt!(N) | cpt!(n) => {
                            check_mask.data |= KNIGHT_ATTACKS[i].data & KNIGHT_ATTACKS[k].data;
                        }
                        _ => {
                            check_mask.data |= RAYS[i][k].data;
                        } /*
                          cpt!(Q) | cpt!(q) => {
                              check_mask.data |=  RAYS[i][k].data;
                          }
                          cpt!(B) | cpt!(b) => {
                              check_mask.data |= RAYS[i][k].data;
                          }
                          cpt!(R) | cpt!(r) => {
                              check_mask.data |= RAYS[i][k].data;
                          }
                          cpt!(P) | cpt!(p) => {
                              check_mask.data |= RAYS[i][k].data;
                          }
                          */
                    }
                }
                checkers = checkers.pop_bit(i)
            }
            //debug
            //println!("check_mask:");
            //println!("{}", check_mask);
        }

        let mut i: usize = match side {
            Side::White => 0,
            Side::Black => 6,
        };

        let limit = i + 6;
        while i < limit {
            let mut sources = self.piece_bbs[i];
            while sources.data != 0 {
                let source: usize = match sources.lsb_index() {
                    Some(x) => x,
                    None => unreachable!(),
                };

                // pin information
                let mut pinners = BB::ZERO;
                let mut pin_mask = BB::ZERO;
                let is_pinned = match i {
                    0 | 6 => false,
                    _____ => self.piece_is_pinned(source),
                };
                if is_pinned {
                    let (q_index, b_index, r_index) = match side {
                        Side::White => (07, 09, 10),
                        Side::Black => (01, 03, 04),
                    };
                    let d_data = (self.piece_bbs[q_index].data | self.piece_bbs[b_index].data)
                        & !(1u64 << source);
                    let l_data = (self.piece_bbs[q_index].data | self.piece_bbs[r_index].data)
                        & !(1u64 << source);
                    let diagonals = BB { data: d_data };
                    let laterals = BB { data: l_data };
                    let data = enemies.data
                        & ((get_bishop_attack(king_pos, diagonals).data & diagonals.data)
                            | (get_rook_attack(king_pos, laterals).data & laterals.data));
                    let mut potential_pinners: BitBoard = BB { data };
                    while potential_pinners.data != 0 {
                        let potential_pinner = match potential_pinners.lsb_index() {
                            Some(x) => x,
                            None => unreachable!(),
                        };
                        // check if piece is between king and potential_pinner
                        if RAYS[king_pos][potential_pinner].data & (1u64 << source) != 0 {
                            pinners.data |= 1u64 << potential_pinner;
                            pin_mask.data |=
                                RAYS[king_pos][potential_pinner].data | (1u64 << potential_pinner);
                        }
                        potential_pinners = potential_pinners.pop_bit(potential_pinner);
                    }
                }

                match i {
                    /* king */
                    00 | 06 => {
                        /* castling */
                        if self.check_bb.data == 0 {
                            // can not castle whilst in check
                            let (k_mask, k_index) = match side {
                                Side::White => (W_KING_SIDE_CASTLE_MASK, 0),
                                Side::Black => (B_KING_SIDE_CASTLE_MASK, 2),
                            };
                            // king-side
                            if self.castle_bools[k_index] && (blockers.data & k_mask.data == 0) {
                                //check if squares are under attack
                                let mut squares = k_mask;
                                let mut can_castle = true;
                                while squares.data != 0 {
                                    let square = match squares.lsb_index() {
                                        Some(x) => x,
                                        None => unreachable!(),
                                    };

                                    if self.is_square_attacked(square, side.update()) {
                                        can_castle = false;
                                    }
                                    squares = squares.pop_bit(square);
                                }
                                if can_castle {
                                    arr = match side {
                                        Side::White => arr.new_raw(03, 01, None, MT::Castle),
                                        Side::Black => arr.new_raw(59, 57, None, MT::Castle),
                                    }
                                }
                            }

                            let (q_mask, q_index) = match side {
                                Side::White => (W_QUEEN_SIDE_CASTLE_MASK, 1),
                                Side::Black => (B_QUEEN_SIDE_CASTLE_MASK, 3),
                            };
                            // queen side
                            if self.castle_bools[q_index] && (blockers.data & q_mask.data == 0) {
                                //check if squares are under attack
                                let data = match side {
                                    Side::White => q_mask.data & !(1u64 << 06),
                                    Side::Black => q_mask.data & !(1u64 << 62),
                                };

                                let mut squares = BB { data };
                                let mut can_castle = true;
                                while squares.data != 0 {
                                    let square = match squares.lsb_index() {
                                        Some(x) => x,
                                        None => unreachable!(),
                                    };

                                    if self.is_square_attacked(square, side.update()) {
                                        can_castle = false;
                                    }
                                    squares = squares.pop_bit(square);
                                }
                                if can_castle {
                                    arr = match side {
                                        Side::White => arr.new_raw(03, 05, None, MT::Castle),
                                        Side::Black => arr.new_raw(59, 61, None, MT::Castle),
                                    }
                                }
                            }
                        }

                        /* moves and attacks */
                        let data: u64 = KING_ATTACKS[source].data & !friends.data;
                        let mut attacks = BB { data };
                        while attacks.data != 0 {
                            let target = match attacks.lsb_index() {
                                Some(x) => x,
                                None => unreachable!(),
                            };
                            // king cannot move to a square under attack
                            let mut removed_king_cb = self.const_clone();
                            let king_index = match side {
                                Side::White => 0,
                                Side::Black => 6,
                            };
                            removed_king_cb.piece_bbs[king_index] = BB::ZERO;
                            removed_king_cb.mailbox[king_index] = None;
                            if !removed_king_cb.is_square_attacked(target, side.update()) {
                                arr = arr.new_raw(source, target, None, MT::Normal);
                            };
                            attacks = attacks.pop_bit(target);
                        }
                    }

                    /* queen */
                    01 | 07 => {
                        let data = get_queen_attack(source, blockers).data & !friends.data;
                        let mut attacks = BB { data };
                        while attacks.data != 0 {
                            let target = match attacks.lsb_index() {
                                Some(x) => x,
                                None => unreachable!(),
                            };

                            // only consider moves along pinning ray if pinned
                            if (pin_mask.data != 0) && (pin_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // only consider moves along checking ray if in check
                            if (check_mask.data != 0) && (check_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // when double checked king has to move
                            if checkers_count > 1 {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            arr = arr.new_raw(source, target, None, MT::Normal);
                            attacks = attacks.pop_bit(target);
                        }
                    }

                    /* knights */
                    02 | 08 => {
                        let data = KNIGHT_ATTACKS[source].data & !friends.data;
                        let mut attacks = BB { data };
                        // pinned knights can not move
                        if pin_mask.data != 0 {
                            sources = sources.pop_bit(source);
                            continue;
                        }

                        while attacks.data != 0 {
                            let target = match attacks.lsb_index() {
                                Some(x) => x,
                                None => unreachable!(),
                            };

                            // only consider moves along checking ray if in check
                            if (check_mask.data != 0) && (check_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // when double checked king has to move
                            if checkers_count > 1 {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            arr = arr.new_raw(source, target, None, MT::Normal);
                            attacks = attacks.pop_bit(target);
                        }
                    }

                    /* bishops */
                    03 | 09 => {
                        let data = get_bishop_attack(source, blockers).data & !friends.data;
                        let mut attacks = BB { data };
                        while attacks.data != 0 {
                            let target = match attacks.lsb_index() {
                                Some(x) => x,
                                None => unreachable!(),
                            };

                            // only consider moves along pinning ray if pinned
                            if (pin_mask.data != 0) && (pin_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // only consider moves along checking ray if in check
                            if (check_mask.data != 0) && (check_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // when double checked king has to move
                            if checkers_count > 1 {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            arr = arr.new_raw(source, target, None, MT::Normal);
                            attacks = attacks.pop_bit(target);
                        }
                    }

                    /* rooks */
                    04 | 10 => {
                        let data = get_rook_attack(source, blockers).data & !friends.data;
                        let mut attacks = BB { data };
                        while attacks.data != 0 {
                            let target = match attacks.lsb_index() {
                                Some(x) => x,
                                None => unreachable!(),
                            };

                            // only consider moves along pinning ray if pinned
                            if (pin_mask.data != 0) && (pin_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // only consider moves along checking ray if in check
                            if (check_mask.data != 0) && (check_mask.data & (1u64 << target) == 0) {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            // when double checked king has to move
                            if checkers_count > 1 {
                                attacks = attacks.pop_bit(target);
                                continue;
                            }

                            arr = arr.new_raw(source, target, None, MT::Normal);
                            attacks = attacks.pop_bit(target);
                        }
                    }

                    /* pawns */
                    05 | 11 => {
                        let mut is_diagonal_pinned = false;
                        let mut is_vertical_pinned = false;
                        let mut is_horizontal_pinned = false;
                        //debug
                        //let data = blockers.data & !(1u64 << source);
                        //let other_blockers = BB { data };
                        //let data = enemies.data & get_queen_attack(king_pos, other_blockers).data;
                        //let side_to_move_is_black = match side {
                        //    Side::White => false,
                        //    Side::Black => true,
                        //};
                        //if king_pos == 22 {
                        //    println!("source:");
                        //    println!("{}", BB{data: (1u64<<source)});
                        //    println!("is_pinned:{}", is_pinned);
                        //    println!("pinners:");
                        //    println!("{}", pinners);
                        //    println!("pin_mask:");
                        //    println!("{}", pin_mask);
                        //}

                        if pin_mask.data != 0 {
                            // TODO: FIX HERE!!!
                            let mut squares = pinners;
                            while squares.data != 0 {
                                let square = match squares.lsb_index() {
                                    Some(x) => x,
                                    None => unreachable!(),
                                };
                                assert!(source != square);
                                if RAYS[king_pos][square].data & (1u64 << source) != 0 {
                                    if DDIAG[source] == DDIAG[square]
                                        || ADIAG[source] == ADIAG[square]
                                    {
                                        is_diagonal_pinned = true;
                                    } else if COLS[source] == COLS[square] {
                                        is_vertical_pinned = true;
                                    } else if ROWS[source] == ROWS[square] {
                                        is_horizontal_pinned = true;
                                    }
                                }
                                squares = squares.pop_bit(square);
                            }
                        }

                        /* pawn moves */
                        if !is_diagonal_pinned && !is_horizontal_pinned {
                            /* one square */
                            //if source < 8 {
                            //    println!("chessboard:");
                            //    println!("{}", self);
                            //}
                            let target = match side {
                                Side::White => source + 8,
                                Side::Black => source - 8,
                            };
                            // can only move one square if next square is empty
                            if (1u64 << target) & blockers.data == 0 {
                                // can only move one square if not in check or blocks check
                                if check_mask.data == 0
                                    || (check_mask.data & (1u64 << target) != 0
                                        && checkers_count == 1)
                                {
                                    let next_square_promotion = match side {
                                        Side::White => source >= 48,
                                        Side::Black => source <= 15,
                                    };

                                    if next_square_promotion {
                                        // promotions
                                        arr = arr.new_promotions(source, target);
                                    } else {
                                        // pawn move 1 square
                                        arr = arr.new_raw(source, target, None, MT::Normal);
                                    }
                                }
                            }

                            /* two square */
                            let next = match side {
                                Side::White => source + 8,
                                Side::Black => source - 8,
                            };

                            let is_initial_sq = match side {
                                Side::White => ROWS[source] == 1,
                                Side::Black => ROWS[source] == 6,
                            };
                            if is_initial_sq {
                                let target = match side {
                                    Side::White => source + 16,
                                    Side::Black => source - 16,
                                };
                                // can only move two squares if pawn is at starting position, and next two squares are empty
                                if ((1u64 << next) | (1 << target)) & blockers.data == 0 {
                                    // can only move one square if not in check or blocks check
                                    if check_mask.data == 0
                                        || (check_mask.data & (1u64 << target) != 0
                                            && checkers_count == 1)
                                    {
                                        arr = arr.new_raw(source, target, None, MT::Normal);
                                    }
                                }
                            }
                        }

                        /* pawn attacks */
                        if !is_horizontal_pinned && !is_vertical_pinned {
                            let data = match side {
                                Side::White => W_PAWN_ATTACKS[source].data & b_blockers.data,
                                Side::Black => B_PAWN_ATTACKS[source].data & w_blockers.data,
                            };
                            let mut attacks = BB { data };
                            while attacks.data != 0 {
                                let target = match attacks.lsb_index() {
                                    Some(x) => x,
                                    None => unreachable!(),
                                };

                                // can only attack a square if not in check or attack blocks check
                                if check_mask.data == 0
                                    || (check_mask.data & (1u64 << target) != 0
                                        && checkers_count == 1)
                                {
                                    //can only attack a square if not pinned or attack is along pin ray
                                    if pin_mask.data == 0 || pin_mask.data & (1u64 << target) != 0 {
                                        let next_square_promotion = match side {
                                            Side::White => source >= 48,
                                            Side::Black => source <= 15,
                                        };

                                        if next_square_promotion {
                                            // promotions
                                            arr = arr.new_promotions(source, target);
                                        } else {
                                            // pawn capture
                                            arr = arr.new_raw(source, target, None, MT::Normal);
                                        }
                                    }
                                }
                                attacks = attacks.pop_bit(target);
                            }
                        }

                        /* en passant */
                        if self.enpassant_bb.data != 0 && !is_pinned {
                            let data = self.enpassant_bb.data
                                & match side {
                                    Side::White => W_PAWN_ATTACKS[source].data,
                                    Side::Black => B_PAWN_ATTACKS[source].data,
                                };
                            let mut targets = BB { data };
                            while targets.data != 0 {
                                let target = match targets.lsb_index() {
                                    Some(x) => x,
                                    None => unreachable!(),
                                };

                                // if there are no checks
                                if self.check_bb.data == 0 {
                                    arr = arr.new_raw(source, target, None, MT::EnPassant);
                                    targets = targets.pop_bit(target);
                                    continue;
                                }

                                // if in check, can only en passant to remove checking pawn
                                if checkers_count == 1 {
                                    let checker = match self.check_bb.lsb_index() {
                                        Some(x) => x,
                                        None => unreachable!(),
                                    };

                                    let enemy_pawn_pos = match side {
                                        Side::White => target - 8,
                                        Side::Black => target + 8,
                                    };

                                    if checker == enemy_pawn_pos {
                                        arr = arr.new_raw(source, target, None, MT::EnPassant);
                                    }
                                }
                                targets = targets.pop_bit(target);
                            }
                        }
                    }

                    __ => unreachable!(),
                }
                sources = sources.pop_bit(source);
            }
            i += 1;
        }
        arr
    }

    pub const fn update_state(&self, chess_move: ChessMove) -> ChessBoard {
        let mut chessboard = self.const_clone();
        let mut enpassant_bb: BitBoard = BB::ZERO;
        let source: usize = chess_move.source();
        let target: usize = chess_move.target();
        let source_data = match chessboard.mailbox[source] {
            Some(x) => x,
            None => panic!("update_state error: source mailbox is None!"),
        };
        let source_index = cpt_index(source_data);

        // handle special cases
        match chessboard.mailbox[source] {
            opt_cpt!(_) => panic!("update_state error: source mailbox is None!"),

            /* special case: castling rights */
            opt_cpt!(K) => {
                chessboard.castle_bools[0] = false;
                chessboard.castle_bools[1] = false;
            }
            opt_cpt!(R) => {
                if source == 0 {
                    chessboard.castle_bools[0] = false;
                } else if source == 7 {
                    chessboard.castle_bools[1] = false
                }
            }
            opt_cpt!(k) => {
                chessboard.castle_bools[2] = false;
                chessboard.castle_bools[3] = false;
            }
            opt_cpt!(r) => {
                if source == 56 {
                    chessboard.castle_bools[2] = false;
                } else if source == 63 {
                    chessboard.castle_bools[3] = false
                }
            }
            /* special case: pawn 2-squares move, en passant rules */
            opt_cpt!(P) => {
                // check if move is 2-square
                if source + 16 == target {
                    if target + 1 < 64 {
                        // check pawn lands next to enemy pawn
                        match chessboard.mailbox[target + 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chessboard.piece_is_pinned(target + 1) {
                                    enpassant_bb.data &= 1 << target - 8
                                }
                            }
                            _______ => {}
                        }
                    }

                    if 0 + 1 <= target {
                        // unsigned hack: 0 <= target - 1
                        // check pawn lands next to enemy pawn
                        match chessboard.mailbox[target - 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chessboard.piece_is_pinned(target - 1) {
                                    enpassant_bb.data &= 1 << target - 8
                                }
                            }
                            _______ => {}
                        }
                    }
                }
            }
            opt_cpt!(p) => {
                if source == target + 16 {
                    // unsinged hack: source - 16 == target
                    if target + 1 < 64 {
                        // check pawn lands next to enemy pawn
                        match chessboard.mailbox[target + 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chessboard.piece_is_pinned(target + 1) {
                                    enpassant_bb.data &= 1 << target + 8
                                }
                            }
                            _______ => {}
                        }
                    }

                    if 0 + 1 <= target {
                        // unsigned hack: 0 <= target - 1
                        // check pawn lands next to enemy pawn
                        match chessboard.mailbox[target - 1] {
                            opt_cpt!(p) => {
                                //check if pawn is not pinned
                                if !chessboard.piece_is_pinned(target - 1) {
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
                // if source is a pawn and move is two-squares, encode enpassant data
                match source_data {
                    cpt!(P) => {
                        if source + 16 == target {
                            enpassant_bb.data |= 1 << (target - 8);
                        }
                    }

                    cpt!(p) => {
                        if source == target + 16 {
                            //source - 16 == target
                            enpassant_bb.data |= 1 << (target + 8);
                        }
                    }

                    _ => {}
                }

                // update source bitboard
                chessboard.piece_bbs[source_index].data &= !(1 << source);
                chessboard.piece_bbs[source_index].data |= 1 << target;

                // if target is occupied, deal with piece capture
                if let Some(target_data) = chessboard.mailbox[target] {
                    chessboard.piece_bbs[cpt_index(target_data)].data &= !(1 << target);
                    match target_data {
                        cpt!(R) => {
                            if target == 0 {
                                chessboard.castle_bools[0] = false;
                            } else if target == 7 {
                                chessboard.castle_bools[1] = false
                            }
                        }

                        cpt!(r) => {
                            if target == 56 {
                                chessboard.castle_bools[2] = false;
                            } else if target == 63 {
                                chessboard.castle_bools[3] = false
                            }
                        }
                        _ => {}
                    }
                }

                // update mailbox
                chessboard.mailbox[source] = None;
                chessboard.mailbox[target] = Some(source_data);
            }

            MoveType::Castle => {
                // update source bitboard
                chessboard.piece_bbs[source_index].data &= !(1 << source);
                chessboard.piece_bbs[source_index].data |= 1 << target;

                // update mailbox
                chessboard.mailbox[source] = None;
                chessboard.mailbox[target] = Some(source_data);

                // deal with rook movement
                match (self.side_to_move, target) {
                    // white king-side castle
                    (Side::White, 1) => {
                        // check if rook is present
                        assert!(self.piece_bbs[04].data & 1u64 << 00 != 0);
                        chessboard.piece_bbs[04].data &= !(1u64 << 00);
                        chessboard.piece_bbs[04].data |= 1u64 << 02;
                        chessboard.mailbox[00] = None;
                        chessboard.mailbox[02] = opt_cpt!(R);
                    }

                    // white queen-side castle
                    (Side::White, 5) => {
                        // check if rook is present
                        assert!(self.piece_bbs[04].data & 1u64 << 07 != 0);
                        chessboard.piece_bbs[04].data &= !(1u64 << 07);
                        chessboard.piece_bbs[04].data |= 1u64 << 04;
                        chessboard.mailbox[07] = None;
                        chessboard.mailbox[04] = opt_cpt!(R)
                    }

                    // black king-side castle
                    (Side::Black, 57) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].data & 1u64 << 56 != 0);
                        chessboard.piece_bbs[10].data &= !(1u64 << 56);
                        chessboard.piece_bbs[10].data |= 1u64 << 58;
                        chessboard.mailbox[56] = None;
                        chessboard.mailbox[58] = opt_cpt!(r)
                    }

                    (Side::Black, 61) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].data & 1u64 << 63 != 0);
                        chessboard.piece_bbs[10].data &= !(1u64 << 63);
                        chessboard.piece_bbs[10].data |= 1u64 << 60;
                        chessboard.mailbox[63] = None;
                        chessboard.mailbox[60] = opt_cpt!(r)
                    }

                    _ => panic!("update_state error: invalid castling target!"),
                }
            }

            MoveType::EnPassant => {
                // note: target is where the capturing pawn will end up,
                //       square is where the pawn to be captured is

                // update source bitboard
                chessboard.piece_bbs[cpt_index(source_data)].data &= !(1 << source);
                chessboard.piece_bbs[cpt_index(source_data)].data |= 1 << target;

                let index = match self.side_to_move {
                    Side::White => 11usize,
                    Side::Black => 05usize,
                };

                let square = match self.side_to_move {
                    Side::White => target - 8,
                    Side::Black => target + 8,
                };

                // check presence of pawn to be captured
                assert!(chessboard.piece_bbs[index].data & (1 << square) != 0);

                // assert!(chessboard.mailbox[square] == Some(relevant_piece));
                if let Some(piece) = chessboard.mailbox[square] {
                    //note: assert hack
                    match self.side_to_move {
                        Side::White => match piece {
                            cpt!(p) => {}
                            _ => panic!(
                                "update_state error: square mailbox is not pawn, en_passant case!"
                            ),
                        },
                        Side::Black => match piece {
                            cpt!(P) => {}
                            _ => panic!(
                                "update_state error: square mailbox is not pawn, en_passant case!"
                            ),
                        },
                    }
                } else {
                    panic!("update_state error: en passant square mailbox is None!")
                }

                // deal with piece capture
                let square_data = match chessboard.mailbox[square] {
                    Some(x) => x,
                    None => panic!("update_state error: en passant square mailbox is None!"),
                };
                let jndex = cpt_index(square_data);
                chessboard.piece_bbs[jndex].data &= !(1u64 << square);

                // update mailbox
                chessboard.mailbox[source] = None;
                chessboard.mailbox[target] = Some(source_data);
                chessboard.mailbox[square] = None;
            }

            MoveType::Promotion => {
                // set bit of source to zero
                chessboard.piece_bbs[source_index].data &= !(1 << source);

                let promotion_piece = match chess_move.get_piece_data() {
                    Some(x) => x,
                    None => panic!(
                        "update_state error: chess_move is a promotion with None piece data!"
                    ),
                };

                let new_piece = (chessboard.side_to_move, promotion_piece);
                let target_index = cpt_index(new_piece);

                // update target bitboard
                chessboard.piece_bbs[target_index].data |= 1 << target;

                // if target is occupied, deal with piece capture
                if let Some(data_target) = chessboard.mailbox[target] {
                    chessboard.piece_bbs[cpt_index(data_target)].data &= !(1 << target);
                }

                // update mailbox
                chessboard.mailbox[source] = None;
                chessboard.mailbox[target] = Some(new_piece);
            }
        }

        chessboard.enpassant_bb = enpassant_bb;
        match chessboard.side_to_move {
            Side::Black => chessboard.full_move_counter += 1,
            _____ => {}
        }
        chessboard.side_to_move = chessboard.side_to_move.update();
        chessboard.half_move_clock += 1;

        // ['K','Q','N','B','R','P','k','q','n','b','r','p'];
        //check if move results in opponent's king to be in check
        match chessboard.king_is_in_check(chessboard.side_to_move) {
            true => {
                match chessboard.side_to_move {
                    Side::White => {
                        let blockers = chessboard.blockers();
                        if let Some(king_pos) = chessboard.piece_bbs[0].lsb_index() {
                            let mut check_bitboard = BB::ZERO;
                            //q
                            check_bitboard.data |= chessboard.piece_bbs[07].data
                                & get_queen_attack(king_pos, blockers).data;
                            //n
                            check_bitboard.data |=
                                chessboard.piece_bbs[08].data & KNIGHT_ATTACKS[king_pos].data;
                            //b
                            check_bitboard.data |= chessboard.piece_bbs[09].data
                                & get_bishop_attack(king_pos, blockers).data;
                            //r
                            check_bitboard.data |= chessboard.piece_bbs[10].data
                                & get_rook_attack(king_pos, blockers).data;
                            //p
                            check_bitboard.data |=
                                chessboard.piece_bbs[11].data & W_PAWN_ATTACKS[king_pos].data;
                            chessboard.check_bb = check_bitboard;
                        } else {
                            panic!("update_state error: white king bitboard is empty!");
                        }
                    }

                    Side::Black => {
                        let blockers = chessboard.blockers();
                        if let Some(king_pos) = chessboard.piece_bbs[6].lsb_index() {
                            let mut check_bitboard = BB::ZERO;
                            //Q
                            check_bitboard.data |= chessboard.piece_bbs[01].data
                                & get_queen_attack(king_pos, blockers).data;
                            //N
                            check_bitboard.data |=
                                chessboard.piece_bbs[02].data & KNIGHT_ATTACKS[king_pos].data;
                            //B
                            check_bitboard.data |= chessboard.piece_bbs[03].data
                                & get_bishop_attack(king_pos, blockers).data;
                            //R
                            check_bitboard.data |= chessboard.piece_bbs[04].data
                                & get_rook_attack(king_pos, blockers).data;
                            //P
                            check_bitboard.data |=
                                chessboard.piece_bbs[05].data & B_PAWN_ATTACKS[king_pos].data;
                            chessboard.check_bb = check_bitboard;
                        } else {
                            panic!("update_state error: black king bitboard is empty!");
                        }
                    }
                }
            }
            false => {
                chessboard.check_bb = BB::ZERO;
            }
        }
        return chessboard;
    }
}

pub fn print_mailbox(mailbox: [CPT; 64]) -> String {
    let mut s = String::new();
    // append characters according to piece
    for i in 1..=64usize {
        let c = match mailbox[64 - i] {
            opt_cpt!(K) => UNICODE_SYM[00],
            opt_cpt!(Q) => UNICODE_SYM[01],
            opt_cpt!(N) => UNICODE_SYM[02],
            opt_cpt!(B) => UNICODE_SYM[03],
            opt_cpt!(R) => UNICODE_SYM[04],
            opt_cpt!(P) => UNICODE_SYM[05],
            opt_cpt!(k) => UNICODE_SYM[06],
            opt_cpt!(q) => UNICODE_SYM[07],
            opt_cpt!(n) => UNICODE_SYM[08],
            opt_cpt!(b) => UNICODE_SYM[09],
            opt_cpt!(r) => UNICODE_SYM[10],
            opt_cpt!(p) => UNICODE_SYM[11],
            opt_cpt!(_) => '.',
        };
        s.push(c);
        if i % 8 == 0 {
            s.push('\n');
        }
    }

    s
}
