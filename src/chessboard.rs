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
    pub current_hash: u64,
    pub repeats: [u8; 1 << 14],
    pub pv: MovesArray,
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
            current_hash: 1544757369275567478, //assuming the constants aren't changed
            repeats: [0; 1 << 14],
            pv: MovesArray::new(),
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
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000}, // ♔
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000}, // ♕
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010}, // ♘
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100}, // ♗
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001}, // ♖
    BB { data: 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000}, // ♙
    BB { data: 0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♚
    BB { data: 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♛
    BB { data: 0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♞
    BB { data: 0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♝
    BB { data: 0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000}, // ♜
    BB { data: 0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000}, // ♟
];

pub const INITIAL_CHESS_POS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

type OCM = Option<ChessMove>;

impl ChessBoard {
    pub const fn search(&self, depth: usize) -> ChessMove {
        match self.negamax(isize::MIN + 1, isize::MAX - 1, depth).1 {
            Some(x) => x,
            None => panic!("search error: no legal move!"),
        }
    }

    const fn negamax(&self, alpha: isize, beta: isize, depth: usize) -> (isize, OCM) {
        if depth == 0 {
            return match self.side_to_move {
                Side::White => (self.naive_eval(), None),
                Side::Black => (-self.naive_eval(), None),
            };
        }

        let moves_array = self.generate_moves();
        //sort moves_array here

        if moves_array.len() == 0 && self.king_is_in_check(self.side_to_move) {
            return (((isize::MIN + 1) / 2) - (depth as isize), None);
        }
        let mut alpha = alpha;
        let mut value: isize = isize::MIN + 1;
        let mut i: usize = 0;
        let mut best_move: Option<ChessMove> = None;
        while i < moves_array.len() {
            let chess_move = match moves_array.data[i] {
                Some(x) => x,
                None => unreachable!(),
            };
            let chessboard = self.update_state(chess_move);
            let new_value = -chessboard.negamax(-beta, -alpha, depth - 1).0;
            // value = max(value, new_value)
            if new_value > value {
                value = new_value;
            }
            // alpha = max(alpha, value)
            if value > alpha {
                alpha = value;
                best_move = Some(chess_move);
            }

            // cutoff
            if alpha >= beta {
                break;
            }
            i += 1;
        }
        (value, best_move)
    }

    const fn old_negamax_neg(&self, alpha: isize, beta: isize, depth: usize) -> (isize, OCM) {
        let (x, y) = self.old_negamax(alpha, beta, depth);
        return (-x, y);
    }

    const fn old_negamax(&self, alpha: isize, beta: isize, depth: usize) -> (isize, OCM) {
        if depth == 0 {
            return match self.side_to_move {
                Side::White => (self.naive_eval(), None),
                Side::Black => (self.naive_eval(), None),
            };
        }

        let mut value: isize = isize::MIN + 1;
        let moves_array = self.generate_moves();

        if moves_array.len() == 0 {
            let side = self.side_to_move;
            if self.king_is_in_check(side) {
                return (((isize::MIN + 1) / 2) - (depth as isize), None);
            }
            return (0, None);
        }

        let mut alpha = alpha;
        //order moves array
        //moves_array.order();
        let mut i: usize = 0;
        let mut best_move: Option<ChessMove> = None;
        while i < moves_array.len() {
            let chess_move = match moves_array.data[i] {
                Some(x) => x,
                None => unreachable!(),
            };
            let chessboard = self.update_state(chess_move);
            if depth > 1 {}
            let (new_value, _) = match depth {
                0 => unreachable!(),
                //1 => match self.side_to_move {
                //    Side::White => todo!(),
                //    Side::Black => todo!(),
                //},
                _ => chessboard.old_negamax_neg(-beta, -alpha, depth - 1),
            };
            let new_value = -new_value;
            // value = max(value, new_value)
            if new_value > value {
                value = new_value;
            }

            // alpha = max(alpha, value)
            if value > alpha {
                alpha = value;
                // better move was found
                best_move = Some(chess_move);
            }

            // cut off
            if alpha >= beta {
                break;
            }
            i += 1;
        }
        return (value, best_move);
    }

    pub const fn naive_eval(&self) -> isize {
        let mut score: isize = 0;
        let mut i: usize = 0;
        while i < 64 {
            score += match self.mailbox[i] {
                opt_cpt!(K) => 100,
                opt_cpt!(Q) => 90,
                opt_cpt!(N) => 30,
                opt_cpt!(B) => 35,
                opt_cpt!(R) => 50,
                opt_cpt!(P) => 10,
                opt_cpt!(k) => -100,
                opt_cpt!(q) => -90,
                opt_cpt!(n) => -30,
                opt_cpt!(b) => -35,
                opt_cpt!(r) => -50,
                opt_cpt!(p) => -10,
                opt_cpt!(_) => 0,
            };
            i += 1;
        }
        score
    }

    pub fn from_fen(input: &str) -> ChessBoard {
        let mut chessboard = ChessBoard {
            piece_bbs: [BB::ZERO; 12],
            mailbox: [None; 64],
            castle_bools: [true; 4],
            enpassant_bb: BB::ZERO,
            check_bb: BB::ZERO,
            side_to_move: Side::White,
            half_move_clock: 0,
            full_move_counter: 0,
            current_hash: 0,
            repeats: [0; 1 << 14],
            pv: MovesArray::new(),
        };
        assert!(input.is_ascii());
        let input_vec: Vec<&str> = input.split_ascii_whitespace().collect();
        assert!(input_vec.len() == 6);

        // parse piece placement data
        let mut i: usize = 0;
        while i < 8 {
            let mut j = 0usize;
            while j < 8 {
                let square: usize = 8 * i + j;
                let mut k: usize = 0;
                while k < input_vec[0].len() {
                    let s = match input_vec[0].chars().nth(k) {
                        Some(x) => x,
                        None => unreachable!(),
                    };
                    if s.is_ascii_alphabetic() {
                        chessboard.piece_bbs[sym_index(s)].data |= 1u64 << square;
                        let piece_data = match s {
                            'K' => (Side::White, PieceType::King),
                            'Q' => (Side::White, PieceType::Queen),
                            'N' => (Side::White, PieceType::Knight),
                            'B' => (Side::White, PieceType::Bishop),
                            'R' => (Side::White, PieceType::Rook),
                            'P' => (Side::White, PieceType::Pawn),
                            'k' => (Side::Black, PieceType::King),
                            'q' => (Side::Black, PieceType::Queen),
                            'n' => (Side::Black, PieceType::Knight),
                            'b' => (Side::Black, PieceType::Bishop),
                            'r' => (Side::Black, PieceType::Rook),
                            'p' => (Side::Black, PieceType::Pawn),
                            _ => panic!("from_fen error!: invalidcharacter!"),
                        };
                        chessboard.mailbox[square] = Some(piece_data);
                    } else if s.is_ascii_digit() {
                        j += (s.to_digit(10).unwrap() as usize) - 1;
                        //break
                    } else {
                        panic!("from_fen error: invalid char in piece placement portion!")
                    }
                    k += 1;
                }
                j += 1;
            }
            i += 1
        }
        // parse active colour
        chessboard.side_to_move = match input_vec[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => panic!("from_fen error: invalid active side!"),
        };

        i = 0;
        // parse castling information
        while i < input_vec[2].len() {
            let s = match input_vec[2].chars().nth(i) {
                Some(x) => x,
                None => unreachable!(),
            };

            match s {
                '-' => continue,
                'K' => chessboard.castle_bools[0] = true,
                'Q' => chessboard.castle_bools[1] = true,
                'k' => chessboard.castle_bools[2] = true,
                'q' => chessboard.castle_bools[3] = true,
                _ => panic!("from_fen error: invalid castling information!"),
            }
            i += 1;
        }

        // parse en passant information
        if input_vec[3] != "-" {
            chessboard.enpassant_bb.data |= 1 << square_index(input_vec[3]);
        }
        //parse halfmove clock
        //assert!(input_vec[4].is_ascii_digit()); doesnt work for &str
        chessboard.half_move_clock = input_vec[4].parse::<usize>().unwrap();
        //parse fullmove number
        //assert!(input_vec[5].is_ascii_digit()); doesnt work for &str
        chessboard.full_move_counter = input_vec[5].parse::<usize>().unwrap();

        //calculate king_is_in_check information.
        assert!(chessboard.piece_bbs[0].data.count_ones() == 1);
        assert!(chessboard.piece_bbs[6].data.count_ones() == 1);
        let side = chessboard.side_to_move;
        if chessboard.king_is_in_check(side) {
            match side {
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
        chessboard.current_hash = ZH::hash(&chessboard) as u64;
        chessboard.repeats[chessboard.current_hash as usize] += 1;
        return chessboard;
    }

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
            current_hash: self.current_hash,
            repeats: self.repeats,
            pv: self.pv,
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
                total += self.update_state(chess_move).perft_count(depth - 1);
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

                                // special psuedo-pinned pawn case:
                                // R . p P k
                                // . . . ^ .
                                // . . . | .
                                // . . . . .

                                let row_bb = BB { data: 0b11111111u64 << (8 * ROWS[source]) };

                                //(enemy rook, enemy pawn, enemy pawn position)
                                let (r_index, p_index, p_pos) = match side {
                                    Side::White => (10, 11, target - 8),
                                    Side::Black => (04, 05, target + 8),
                                };

                                // if enemy rook and friendly king is in the same row, check for special case
                                if (ROWS[king_pos] == ROWS[source])
                                    && (self.piece_bbs[r_index].data & row_bb.data != 0)
                                {
                                    //debug
                                    //println!("source:{}", source);
                                    //println!("side:{:?}", side);

                                    // check if enpassant leaves king in check
                                    let mut test = self.const_clone();
                                    test.piece_bbs[i].data &= !(1u64 << source);
                                    test.piece_bbs[i].data |= 1u64 << target;
                                    test.piece_bbs[p_index].data &= !(1u64 << p_pos);

                                    //debug
                                    //println!("king_is_in_check:{}", test.king_is_in_check(side));

                                    if test.king_is_in_check(side) {
                                        targets = targets.pop_bit(target);
                                        continue;
                                    }
                                }

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

                //update hash
                chessboard.current_hash ^= ZH::get_piece_hash(source, source_data);
                chessboard.current_hash ^= ZH::get_piece_hash(target, source_data);

                // if target is occupied, deal with piece capture
                if let Some(target_data) = chessboard.mailbox[target] {
                    chessboard.piece_bbs[cpt_index(target_data)].data &= !(1 << target);
                    //update hash
                    chessboard.current_hash ^= ZH::get_piece_hash(target, target_data);
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

                //update hash
                chessboard.current_hash ^= ZH::get_piece_hash(source, source_data);
                chessboard.current_hash ^= ZH::get_piece_hash(target, source_data);

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

                        //update hash
                        chessboard.current_hash ^= ZH::get_piece_hash(00, cpt!(R));
                        chessboard.current_hash ^= ZH::get_piece_hash(02, cpt!(R));
                    }

                    // white queen-side castle
                    (Side::White, 5) => {
                        // check if rook is present
                        assert!(self.piece_bbs[04].data & 1u64 << 07 != 0);
                        chessboard.piece_bbs[04].data &= !(1u64 << 07);
                        chessboard.piece_bbs[04].data |= 1u64 << 04;
                        chessboard.mailbox[07] = None;
                        chessboard.mailbox[04] = opt_cpt!(R);

                        //update hash
                        chessboard.current_hash ^= ZH::get_piece_hash(07, cpt!(R));
                        chessboard.current_hash ^= ZH::get_piece_hash(02, cpt!(R));
                    }

                    // black king-side castle
                    (Side::Black, 57) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].data & 1u64 << 56 != 0);
                        chessboard.piece_bbs[10].data &= !(1u64 << 56);
                        chessboard.piece_bbs[10].data |= 1u64 << 58;
                        chessboard.mailbox[56] = None;
                        chessboard.mailbox[58] = opt_cpt!(r);

                        //update hash
                        chessboard.current_hash ^= ZH::get_piece_hash(56, cpt!(r));
                        chessboard.current_hash ^= ZH::get_piece_hash(58, cpt!(r));
                    }

                    (Side::Black, 61) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].data & 1u64 << 63 != 0);
                        chessboard.piece_bbs[10].data &= !(1u64 << 63);
                        chessboard.piece_bbs[10].data |= 1u64 << 60;
                        chessboard.mailbox[63] = None;
                        chessboard.mailbox[60] = opt_cpt!(r);

                        //update hash
                        chessboard.current_hash ^= ZH::get_piece_hash(63, cpt!(r));
                        chessboard.current_hash ^= ZH::get_piece_hash(60, cpt!(r));
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

                //update hash
                chessboard.current_hash ^= ZH::get_piece_hash(source, source_data);
                chessboard.current_hash ^= ZH::get_piece_hash(target, source_data);

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

                //update hash
                chessboard.current_hash ^= ZH::get_piece_hash(square, square_data);

                // update mailbox
                chessboard.mailbox[source] = None;
                chessboard.mailbox[target] = Some(source_data);
                chessboard.mailbox[square] = None;
            }

            MoveType::Promotion => {
                let promotion_piece = match chess_move.get_piece_data() {
                    Some(x) => x,
                    None => panic!(
                        "update_state error: chess_move is a promotion with None piece data!"
                    ),
                };

                let new_piece = (chessboard.side_to_move, promotion_piece);
                let target_index = cpt_index(new_piece);

                // update bitboards
                chessboard.piece_bbs[source_index].data &= !(1 << source);
                chessboard.piece_bbs[target_index].data |= 1 << target;

                //update hash
                chessboard.current_hash ^= ZH::get_piece_hash(source, source_data);
                chessboard.current_hash ^= ZH::get_piece_hash(target, new_piece);

                // if target is occupied, deal with piece capture
                if let Some(data_target) = chessboard.mailbox[target] {
                    chessboard.piece_bbs[cpt_index(data_target)].data &= !(1 << target);

                    //update hash
                    chessboard.current_hash ^= ZH::get_piece_hash(target, data_target);
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

        let mut enpassant_bb = chessboard.enpassant_bb;
        while enpassant_bb.data != 0 {
            let square = match enpassant_bb.lsb_index() {
                Some(x) => x,
                None => unreachable!(),
            };
            chessboard.current_hash ^= ZH_KEYS.1[4 + COLS[square]];
            enpassant_bb = enpassant_bb.pop_bit(square);
        }

        // lazy way to handle this
        let mut i: usize = 0;
        //castling hash
        while i < 4 {
            if chessboard.castle_bools[i] {
                chessboard.current_hash ^= ZH_KEYS.1[i];
            }
            i += 1;
        }

        //en passant hash
        let mut enpassant_bb = chessboard.enpassant_bb;
        while enpassant_bb.data != 0 {
            let square = match enpassant_bb.lsb_index() {
                Some(x) => x,
                None => unreachable!(),
            };
            chessboard.current_hash ^= ZH_KEYS.1[4 + COLS[square]];
            enpassant_bb = enpassant_bb.pop_bit(square);
        }

        //side to move hash
        chessboard.current_hash ^= ZH_KEYS.2[0];

        chessboard.repeats[((self.current_hash % usize::MAX as u64) as usize) % (1 << 14)] += 1;

        //move principal variation forward
        if self.pv.len() > 0 {
            chessboard.pv.count = self.pv.count - 1;
            chessboard.pv.data = [None; 256];
            let mut i: usize = 0;
            while i + 1 < self.pv.count {
                chessboard.pv.data[i] = self.pv.data[i + 1];
                i += 1;
            }
        }
        return chessboard;
    }

    pub fn parse_uci_move(&mut self, moves_str: &str) {
        let chess_moves = moves_str.split(' ');
        for uci_moves in chess_moves {
            let moves_arr = self.generate_moves();
            if uci_moves == "startpos" {
                *self = ChessBoard::default();
                continue;
            }
            let mut i: usize = 0;
            while i < moves_arr.len() {
                let chess_move: ChessMove = moves_arr.data[i].unwrap();
                if format!("{}", chess_move) == uci_moves {
                    //maybe parse into a source/target and do int compare
                    *self = self.update_state(chess_move);
                    break;
                }
                i += 1;
            }
        }
    }

    pub fn parse_uci_position_cmd(&mut self, cmd_str: &str) {
        //shitty debug flat
        //if true {
        //    println!("cmd_str:{}", cmd_str);
        //}
        let mut cmds = cmd_str.split(' ');
        while let Some(cmd) = cmds.next() {
            //shitty debug flat
            //if true {
            //    println!("processing cmd:{}", cmd);
            //};
            // UCI command - startpos
            if cmd == "startpos" {
                *self = ChessBoard::default();
                continue;
            }

            // UCI command - fen
            if cmd == "FEN" || cmd == "fen" {
                //todo:fix from_fen command;
                let fen: &str = match cmds.next() {
                    Some(x) => x,
                    None => INITIAL_CHESS_POS_FEN,
                };

                *self = ChessBoard::from_fen(fen);
                continue;
            }
            // UCI command - moves
            //else if cmd == "MOVES" || cmd == "moves" {
            else {
                let moves_arr = self.generate_moves();
                let mut i: usize = 0;
                while i < moves_arr.len() {
                    let chess_move: ChessMove = moves_arr.data[i].unwrap();
                    if format!("{}", chess_move) == cmd {
                        //todo: maybe parse into a source/target and do int compare
                        *self = self.update_state(chess_move);
                        //break;
                    }
                    i += 1;
                }
            }
        }
    }
    pub fn parse_uci_go_cmd(&mut self, cmd_str: &str) -> String {
        let mut depth: usize = 6;

        let mut cmds = cmd_str.split(' ');

        while let Some(cmd) = cmds.next() {
            // fix depth search
            if cmd == "depth" {
                //todo: fix this lazy shit
                depth =
                    cmds.next().expect("Invalid input").parse::<usize>().expect("Invalid input");
            }
            // other cases placeholder
            else {
                depth = 6;
            }
            //other cases?
        }
        //search_position(depth)
        format!("bestmove {}", self.search(depth))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZorbristHash {
    piece_data: [[u64; 12]; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZorbristHashKey {
    piece_data: [[u64; 12]; 64],
    extra_data: [u64; 12],
    side_data: u64,
}

pub type ZH = ZorbristHash;
// extra 12 data are:
// WKCastle, WQCastle, BKCastle, BQCastle,
// eight En Passant file,

#[rustfmt::skip]
pub const ZH_KEYS: ([[u64; 12]; 64], [u64; 12], [u64; 1]) = (
    [
        [11649360378367592562, 15874266044322398818, 13358298267811385711, 11517226955169310093, 7785449912579291613, 11987742406595484964, 5273849987073312900, 6414093628429638632, 6073891907990312073, 12267466340209826993, 13879235981486158591, 4650897482345366460],
        [10212951969080192299, 14859470432780376578, 15132549311343058648, 18021385666288400224, 8903090014600179490, 11955416193095048771, 6369310106067506704, 3215505740984848128, 6167858331790515798, 7906321719453278453, 4260076893754002488, 8658779822187701468],
        [9791658838455668095, 16911306289016467386, 14732354847742049260, 16711138440490065886, 4980209479366463420, 15930748531858109729, 10197789719261113864, 519154468693253581, 9869543112804289986, 16300995609657583306, 724639724172017244, 15857835098612520538],
        [7419224766397067241, 7655273564913272805, 7190784303332036412, 2612450910397931720, 17177723060047616633, 14003928210563933209, 17577063202657060597, 4057291853745572672, 894350242472182428, 18440654079916620548, 18370059514739227136, 18100487786937295816],
        [4540697281450478151, 10959315846913131452, 12374997617922651221, 16078445359423361668, 5506757285683615719, 11708265826211409921, 7634379723654910818, 1786897194385660405, 8478156720766491889, 13179421190112654803, 15432346513071886782, 12235790734059416573,],
        [15102075098821060567, 5864094832784938916, 8693808734616718812, 4774167416311541591, 9407827853871600158, 5036986972913483074, 16459914423964944697, 9599910928403443224, 8680345405212658711, 34948736271424203, 11140815635374034978, 6219589680077917427],
        [12685910156127355126, 3046996110540266629, 256185952445463788, 6369104699509403249, 15822365175139727492, 9489968820115722130, 387077999123901078, 845253841740752820, 5804803258547760215, 5920043842717762116, 11533142499127151891, 13884357894666687217],
        [15656533194976535714, 2361430565911213262, 13154008255490341451, 16237020840377979104, 16490353220749911253, 11442710689065807954, 9110004028231952951, 16222947167539680132, 7284994485224751478, 18390488009929442273, 8533255380738342499, 9110326413241463636,],
        [5728272574160103686, 8510500721195889288, 6676417758086664951, 9621981165712524989, 4548250075198197901, 5049120513785868241, 12146822937348725698, 10491190948140135956, 7821226172416184137, 18254990867928570397, 2378520867832291233, 135477415055244568],
        [2825901155640567254, 1903186817871594688, 18101891542441717261, 13782983792399316694, 14929380146268669129, 733686471241133128, 14514887047502535660, 14128207014108399184, 9537102421749467161, 13984066737992781297, 16286588233924965098, 12186085620686453059,],
        [17108093811667803831, 7774020209605121760, 5598639951922459026, 8461063609934752839, 11468739846136377917, 6253027074136317495, 12996746564202689587, 5588842227303907148, 575346816583814888, 5165573188679135782, 8978433054722353030, 1109680854363233478],
        [4099512801345881190, 623426345973902716, 3025405725915864229, 4111361583264376951, 7067355212846562988, 17345698829597963755, 10807651001060047365, 8214844237607600263, 17773686292963642226, 4233654098456459867, 14497594808517371979, 11817265094736638436],
        [2522853440704040191, 394172442374815231, 4061687696991137611, 12519871651236660794, 5009431739462177668, 1522885171263983017, 9155487816117475350, 10379892324061127512, 2570899594048510401, 17829530844389191229, 16741564134617282736, 16074639296428420606],
        [17031139621282256570, 14446758980250240958, 7541038076982086770, 16088975869770947266, 534276942063249974, 10801018471703363169, 1281913608069445867, 4784876212295801413, 6186105731722332030, 13158753642979731039, 11710955968211907138, 8963724090984417843],
        [10423183156429107238, 14475862455648204345, 16433193990077648657, 539777526157295179, 3026514590487639782, 9192136930260507488, 12360655971976786594, 17788250481037544293, 14074469617942599326, 3961697193287028436, 287987646090265232, 11280503099944194275],
        [1913152888649924419, 3269426874178497226, 9428233852841292233, 16470942717070933557, 13452720376914387498, 13517426250165022571, 8256313042623743525, 9662869942092388345, 14839722753575977734, 604382169373409530, 13813365047748863926, 8461363947254555976],
        [7051560120674943910, 17985416276776935465, 9531623638418471576, 8589820347481854177, 2611087442692960155, 16269839662507311440, 515823671014626000, 3555074465477212238, 2688301892161775445, 88838723982144517, 7751011220428361773, 17922180761480764551],
        [1898719085908631131, 16426645275766390517, 6093576246148170473, 17801026100383629872, 12688464693598880167, 8557679046670243647, 8920698185217601075, 6550138302318073128, 10484885717437618230, 5358940818823898783, 7576553479986284767, 12551644455927962853],
        [4637413408460158438, 4043155840418721536, 14499151105319776988, 6383021898439792130, 11821252872135312159, 6902813725040227399, 14586374082623718940, 13397646266841062959, 6562694058000810671, 3396892223741729417, 9867208269966420158, 15425723550680316166],
        [14509254912679775515, 1996909855476090391, 13587629168178434969, 4388512496119655575, 2660541327200606243, 14227857020031000847, 14819991844180209549, 6240052298345718833, 16273241714414119179, 5782673831649668988, 13504215107165993868, 7079827799905107232],
        [1996319421570217506, 11850607300421941872, 16022874389166950878, 18230504802099662588, 5700631588375789420, 12971152530683578448, 8475596356384462820, 1040084389067865812, 9303151451014384621, 1146605896731570937, 13059199954820821782, 8013712852755022720],
        [11664771491438771298, 14968766297662776643, 9900668672724301371, 15074620372851619271, 8692348565957058499, 7289276081503637247, 1506460004333627080, 4044608072737426751, 4024210190085478367, 9377229361568556667, 17442337467538803252, 7667694678511660105],
        [1901717266972022127, 1489317447548150563, 10900227721999928486, 5155047589176544839, 15351073696767319929, 9066793902005053567, 16776409776193876668, 15347774395029809445, 13461538646184025396, 11120338830509378379, 15644022483297001950, 13926918745067351434],
        [16724441622009597096, 14905166961920450858, 5048874899947978383, 11623942677618291314, 6880938561958517309, 2959301735303470313, 4275365721514248011, 9423372437337068058, 996910645382467023, 12093770737115355200, 7894458230267358831, 8633048957208510212],
        [18018861988934300385, 2192846716005156847, 12572708700556646838, 16852007020208723977, 8737743119725332217, 14350784061198371713, 15755216428127363499, 15501480637333113369, 6726388682169324057, 7908747796310430134, 1074331102876512479, 7625590068174158416],
        [7726599961164250131, 4437103229335835429, 10444256016372087720, 3848236506421412943, 11677561258355233068, 12606494596626438984, 6019161811771640969, 14975301070601423127, 2766131991660486390, 3371251209876643798, 9672209364465937988, 12868086664070183883],
        [13157214796524654726, 16438178797328130695, 17014328388810498148, 763033128311551695, 5820228081816009262, 9710721650982085535, 4428240097178537050, 4735972952824747783, 15651795860577686668, 12277156006165336471, 9903140119850425222, 9448142080063829897],
        [16127467651928606731, 1029121599393349917, 3249760006773968563, 3423900128617146365, 5741088830295163051, 8269882731514212266, 345920645567487910, 17467323433118184843, 7336171185419306667, 16398189352267768862, 6785374836136972362, 15541563933181111715],
        [16296338373549602974, 2399847202736333121, 6457218135334097074, 9892043294137290185, 17418513261090101125, 3193778736692157669, 17746666740936125879, 13975082082865428911, 6366314881976333647, 11013814988137485632, 7682646320665970981, 15831504833070606543],
        [3858918878224865663, 9682340893715503028, 773715170651108574, 17105661411642031257, 3374371553245629608, 846571004775987117, 10869860616868166516, 942911828403752732, 9628961084257737873, 9128355034410454945, 7234720835206917161, 12276649814551948110],
        [1707605985507806188, 9856470993262782934, 3563180606085273142, 9202464764198970970, 13030484218578811506, 13243482587076584816, 5095658230590361466, 3533466595860681290, 15423437439616988477, 3050123904168647698, 6420150630630199, 14746961147872008718],
        [7945948287104201568, 15919672660904011191, 15959543236241220221, 4243252883191817403, 8185121602118098451, 8645154702303406914, 17808666749113965944, 16673893026104913483, 2787452220991419144, 5341356203311155684, 10993155192531413489, 10676309324351214391],
        [9403624166868065802, 1631652596505707410, 2165550500723918615, 14169678794003595746, 11748246248613471994, 13975415903795209595, 17907584190081425122, 10238802817980040784, 10021154633279389343, 3621311125475484658, 4489853721588530517, 6545734766553909154],
        [11548531987151270326, 56182312929063301, 9171348281144810972, 1340468383244777284, 9753266015384913927, 12334993034181524043, 18341316721492439301, 5772358945016970000, 6368111864546354838, 1431260908470665454, 3061143471963230715, 1804742611754570537],
        [18004325925855546027, 3578242346395756486, 13867178033836693037, 440959796429860652, 6457757406990995433, 13215405668782503425, 2893339371449068371, 16794839887386943567, 2422179348595144347, 10432722667333529467, 2304221295360263497, 9004423124779613242],
        [4780640132243547868, 12486835589638200952, 5593040757232725147, 4846963207318785069, 14302167071168006162, 969394369303588361, 16645044010864465709, 4973639224598977872, 6434279539158563771, 2970193780204685764, 7779951228037915001, 13082833757200868480],
        [9181695439878108605, 13481216026501151440, 3105544132992597373, 1579764249997924947, 13424586327497683345, 13752752049667061484, 10271389343174182960, 908772482972385029, 18294448384462430745, 2489837195784110876, 9958562889371265272, 781770860506621640],
        [3532729058774911412, 1468225476671373283, 9465254559064654757, 12065282485963376483, 9543593262572317227, 14764802197071310266, 8268617637764220825, 13664319657697261117, 13412314385792144554, 9113372987644799584, 6850721789814051208, 14677150171664649159],
        [12012674393292520466, 13225216703068957132, 5074969637877649077, 12596152717656783132, 4212012078606977471, 688059095959643865, 7068874165307879852, 7158937047750052770, 2959759328733957697, 2572983706976621047, 17027878642558314292, 7696913091689018042],
        [16660228394166940975, 8763940628030545055, 2596423577486868656, 5368005687735534372, 9304053779580184306, 6205948915449215390, 9623824342034812229, 9099110422636570523, 11985441228481300994, 8890585119504843884, 16677712712303765567, 5121621932497360567],
        [9429243113595265492, 11333637286045337048, 1449824856126674050, 4219368182328577109, 3933118071044586270, 6070542068533332916, 9447652564263290155, 15020743576880487026, 14617038334477895611, 16337902362826744578, 7294259090476023907, 12985600540067480445],
        [12639090864812335739, 5986323066941974989, 12849201187012362008, 14445200826329839099, 15757096077056423107, 13962792505839318705, 2412781398818678195, 1794537608597023642, 14332945934490615251, 6162052929763598875, 8018028428512893408, 15401552011325186687,],
        [17418024433016343680, 154658362412119856, 17624045206926931049, 17667094802924108994, 2524907524654013630, 11691614272073992649, 12217351380013177432, 1046952567719130872, 13478597270016577403, 15363490358716084995, 17810084088630844409, 13304314575826710842],
        [10961332313759616381, 17425280925873325302, 7314610107812140681, 15173432190439556873, 12724239072607697054, 369582615098740547, 407981161771773742, 11679162202110710932, 14695316249178901248, 6481395869484507381, 9942761678540909031, 5548777092099040698],
        [8617815372809097852, 15396475639111198483, 2907665357909953323, 9923799893662657107, 6653898221998197824, 3910394422151425755, 8082798537938225187, 9147291240487843499, 605316962796958975, 7294979221453019982, 16656651689234602902, 8382176006044578926],
        [17224632516517742569, 2074154260670213273, 2187055750898258118, 7983529408927387567, 15368590123555950358, 11896564840137064188, 10604924104507091993, 13499821009993535846, 14615140731666045110, 1386254814744884479, 13837986624883582980, 11668022960163972993],
        [6607580305892864866, 7466444628116424835, 281569652911041835, 8529202816761637322, 11212170960466084034, 12302007445845717611, 10631334147477572701, 6707109700611398031, 4942019920882703384, 3494870730776605372, 8761866537108496782, 5312076346401448397],
        [4626622475221586073, 17177794175935518113, 4585202408753728986, 2382270296530091226, 12877562708770497947, 15830024476617316409, 796400106903254174, 14506628651603577474, 3696307527975268801, 11381197106673289213, 12284847373298903541, 12046841225705000489],
        [15408650559269228869, 8840961163398090393, 12714092965619010364, 5492813788369046011, 5841482987093391031, 4450271265434470277, 4304833121550333368, 10926194810606148986, 4072328349225561499, 13061640880157562375, 11399600823687809125, 7934571350773788585],
        [18132287508814884379, 2335626321508079278, 14548073588572346205, 17385325654354375648, 10145441223753776752, 5116953369440379088, 7310637563801410088, 17568468466256863885, 11803995727502435032, 7228086822873151709, 4555881724419008596, 17033211763294847675,],
        [14966195936965067268, 2981794491948293810, 15737233950896504907, 355644489266926232, 12395252928030301219, 4951717485796359611, 276868438401368473, 4605346221684965672, 2878474811982588943, 327846554916075059, 14536349767292904263, 9157833909856160450],
        [15849775923686871417, 7812514181046648521, 1698966955095773220, 9754795879602217510, 10988978593591888362, 10608349262077442522, 14718531275288777409, 1289199202592693942, 8198427420404896128, 18400345310620875786, 17848569192607044862, 16923443992651250353,],
        [1521316042797202328, 7618659390924720069, 5709057395656590556, 12530438736221791991, 13138707564512177191, 8499226405446953788, 16433617613651427545, 17036279805642919025, 5461591032143862168, 5968743073609601167, 4141252120808893442, 1620780364965425968],
        [2871089070988458225, 14832980351046814149, 4075148700516593835, 10033698742491673414, 4343324512457267198, 16336293077703256472, 3868111892777241579, 14609035761974265208, 16239204863649173590, 1355548061563048743, 5364456187523758374, 15090989499591999011],
        [15335968336957651022, 4215241480372090071, 16499051111983610712, 2944365776102793696, 8299646434879747883, 8095057462549752477, 14636958375945640377, 10425130494281608272, 6338921575082711513, 14215683278132053505, 16703380114496149140, 16817557548483454650,],
        [4214826225491115416, 5815941313169302256, 10127930881517675232, 7775114229482020638, 11908744976251770115, 16680700255536980941, 13226581127357126284, 9955002670333632400, 14868444775694583780, 2148234486308254762, 7443805731371408727, 4675886902804534256],
        [1767798586630170348, 7741113346166806494, 5824431231697559544, 7842540555583090040, 208250988282093809, 1921423784632396118, 15998358970975421052, 9890189812625803203, 15599116339307040754, 6362564462494965410, 9680592959688683494, 43565806057224665],
        [18198319421367184191, 8663754226505354340, 16998794507631819983, 4004195935232345974, 10841249478188513983, 3868065188536708351, 14187164756509610351, 9345443470304482489, 11517888978562984214, 6613045958006149247, 15340950616015439872, 2127004569619823416],
        [7106194589355722260, 16844956113032083190, 11571717347216480519, 15033876840362233177, 11959072950473206121, 11672342327679140694, 8276697531791234613, 4605715867015334891, 2708206516895737253, 16189970906814429166, 1743169389719177280, 17519541824404500404,],
        [15426709069634003169, 3494412864914728295, 12894984109088038056, 4328052962870574963, 13357305285237091988, 14472829109379320901, 14843686616576784331, 6867520683696448787, 2047571578897519757, 9562018503583484511, 10729276512414756020, 12121247056749329052,],
        [5070680753802680711, 14822106932491528595, 5078028793030918623, 12377065439522508756, 14189984527744086631, 7943664908235575360, 8498578566168762609, 12000076286189310276, 6362826919407724554, 11530434703208190475, 17033410690648979955, 7554313653292127635],
        [7046963210819048226, 11397316884184533145, 10985983304199029471, 3291409556853573065, 7565093165262767077, 16337303391361231194, 17669908151434013333, 3913365214893631933, 5117328184434525441, 15100288741277907332, 9473795786220201977, 15688017444614126281],
        [16744832919829438496, 18235465113706085460, 8278038647281859056, 20244351167557056, 16210054571851336238, 13325649176133256824, 15131600813396970063, 12859914437817650451, 96670914650647945, 629016116328170513, 2343778633396251847, 13832913956104484387],
        [6284197652488151500, 4643929950577355398, 11589034196701729348, 4255276912148789280, 16453239592173970793, 14280499481704223381, 14463101049584335816, 14482677479086724261, 13894809945481435820, 15513757828565910155, 270246162587239947, 1327566141813347118]
    ],
    
    [
        7646256629535137843, 7686626262289413131, 8375753454377820881, 17655566769599551318, 2160563305958692209, 3038424709401290825, 13694627374606792971, 17438082743793298002, 17019090133566225818, 3587443712359091964, 3966153725277649354, 16770344834736168106
    ], 
    
    [
        8110401300783452258
    ],
);

impl ZorbristHash {
    pub const fn hash(chessboard: &ChessBoard) -> usize {
        let mut val = match chessboard.side_to_move {
            Side::White => 0u64,
            Side::Black => ZH_KEYS.2[0],
        };

        let mut i = 0usize;
        while i < 64 {
            let p = match chessboard.mailbox[i] {
                opt_cpt!(K) => 00,
                opt_cpt!(Q) => 01,
                opt_cpt!(N) => 02,
                opt_cpt!(B) => 03,
                opt_cpt!(R) => 04,
                opt_cpt!(P) => 05,
                opt_cpt!(k) => 06,
                opt_cpt!(q) => 07,
                opt_cpt!(n) => 08,
                opt_cpt!(b) => 09,
                opt_cpt!(r) => 10,
                opt_cpt!(p) => 11,
                opt_cpt!(_) => 12,
            };

            if p != 12 {
                val ^= ZH_KEYS.0[i][p];
            }
            i += 1;
        }

        i = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                val ^= ZH_KEYS.1[i];
            }
            i += 1;
        }

        let mut enpassant_bb = chessboard.enpassant_bb;
        while enpassant_bb.data != 0 {
            let square = match enpassant_bb.lsb_index() {
                Some(x) => x,
                None => unreachable!(),
            };
            val ^= ZH_KEYS.1[4 + COLS[square]];
            enpassant_bb = enpassant_bb.pop_bit(square);
        }

        (val % (usize::MAX as u64)) as usize
    }

    pub const fn get_piece_hash(square: usize, piece_type: (Side, PieceType)) -> u64 {
        ZH_KEYS.0[square][cpt_index(piece_type)]
    }
}
