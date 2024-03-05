#![allow(dead_code)]
#![allow(long_running_const_eval)]

use std::fmt::{Debug, Display};

use crate::bitboard::*;

/* indexing the 64-squares:
  |-----------------------| BLACK KING SIDE
8 |63 62 61 60 59 58 57 56|
7 |55 54 53 52 51 50 49 48|
6 |47 46 45 44 43 42 41 40|
5 |39 38 37 36 35 34 33 32|
4 |31 30 29 28 27 26 25 24| //30
3 |23 22 21 20 19 18 17 16| //20
2 |15 14 13 12 11 10  9  8|
1 | 7  6  5  4  3  2  1  0|
  |-----------------------| WHITE KING SIDE
    A  B  C  D  E  F  G  H                  */

/*  binary masks           description         hexidecimal masks
0000 0000 0011 1111    source square       0x3f
0000 1111 1100 0000    target square       0xfc0
0011 0000 0000 0000    promoted piece data 0x3000
1100 0000 0000 0000    move type           0xc000

note: move types are encoded as follows
00 - normal move
01 - castle move
10 - en passant
11 - promotion

note: promoted piece data are encoded as follows
00 - knight
01 - bishop
10 - rook
11 - queen                                                   */

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessMove {
    pub data: u16,
}

pub type CM = ChessMove;

impl Display for ChessMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.print_move();
        write!(f, "{}", s)
    }
}

impl Debug for ChessMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.print_move();
        s.push_str(format!(" {:?}", self.get_move_type()).as_str());
        write!(f, "{}", s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion,
}
pub type MT = MoveType;

impl ChessMove {
    // get functions
    pub const fn get_source_index(&self) -> usize {
        ((self.data & 0b111111u16) as usize) >> 0
    }
    pub const fn get_target_index(&self) -> usize {
        ((self.data & 0b111111_000000u16) as usize) >> 6
    }
    pub const fn get_piece_data(&self) -> Option<PieceType> {
        if let MoveType::Promotion = self.get_move_type() {
            match ((self.data & 0b11_000000_000000u16) as usize) >> 12 {
                0b00 => Some(PieceType::Knight),
                0b01 => Some(PieceType::Bishop),
                0b10 => Some(PieceType::Rook),
                0b11 => Some(PieceType::Queen),
                _ => panic!("get_piece_data error: invalid piece data"),
            }
        } else {
            None
        }
    }
    pub const fn get_piece_data_raw(&self) -> usize {
        ((self.data & 0b11_000000_000000u16) as usize) >> 12
    }
    pub const fn get_move_type(&self) -> MoveType {
        match ((self.data & 0b11_00_000000_000000) as usize) >> 14 {
            0 => MoveType::Normal,
            1 => MoveType::Castle,
            2 => MoveType::EnPassant,
            3 => MoveType::Promotion,
            _ => panic!("get_move_type error: invalid move type data!"),
        }
    }
    pub const fn get_move_type_raw(&self) -> usize {
        ((self.data & 0b11_00_000000_000000) as usize) >> 14
    }

    // shorter function names
    pub const fn source(&self) -> usize {
        ((self.data & 0b111111) as usize) >> 0
    }
    pub const fn target(&self) -> usize {
        ((self.data & 0b111111_000000) as usize) >> 6
    }
    pub const fn piece(&self) -> Option<PieceType> {
        if let MoveType::Promotion = self.get_move_type() {
            match ((self.data & 0b11_00_000000_000000) as usize) >> 12 {
                0b00 => Some(PieceType::Knight),
                0b01 => Some(PieceType::Bishop),
                0b10 => Some(PieceType::Rook),
                0b11 => Some(PieceType::Queen),
                _ => panic!("get_piece_data error: invalid piece data"),
            }
        } else {
            None
        }
    }

    // set functions
    pub fn set_source_index(&mut self, index: usize) {
        self.data &= ((index << 0) & 0b111111) as u16;
    }
    pub fn set_target_index(&mut self, index: usize) {
        self.data &= ((index << 6) & 0b111111_000000) as u16;
    }
    pub fn set_piece_data(&mut self, piece_data: Option<PieceType>) {
        //doesn't check piece_data == None <-> move_type != Promotion!
        if piece_data == None {
            return;
        } else {
            let piece_data: usize = match piece_data {
                Some(PieceType::Knight) => 0b00,
                Some(PieceType::Bishop) => 0b01,
                Some(PieceType::Rook) => 0b10,
                Some(PieceType::Queen) => 0b11,
                _ => panic!("set_piece_data error: invalid piece_data!"),
            };
            self.data &= ((piece_data << 12) & 0b11_00_000000_000000) as u16;
        }
    }
    // pub fn set_piece_data_raw(&mut self, piece_data: usize) {
    //     self.data &= ((piece_data << 12) & 0x3000) as u16;
    // }
    pub fn set_move_type(&mut self, move_type: MoveType) {
        let move_type_data = match move_type {
            MoveType::Normal => 0,
            MoveType::Castle => 1,
            MoveType::EnPassant => 2,
            MoveType::Promotion => 3,
        };
        self.data &= ((move_type_data << 14) & 0b11_00_000000_000000) as u16;
    }
    // pub fn set_move_type_raw(&mut self, move_type: usize) {
    //     self.data &= ((move_type  << 14) & 0xc000) as u16;
    // }

    // helper functions
    pub fn set_data(
        &mut self,
        source_index: usize,
        target_index: usize,
        piece_data: Option<PieceType>,
        move_type: MoveType,
    ) {
        assert!((piece_data == None) == (move_type != MoveType::Promotion));
        self.set_source_index(source_index);
        self.set_target_index(target_index);
        self.set_piece_data(piece_data);
        self.set_move_type(move_type);
    }

    pub const fn new(
        source_index: usize,
        target_index: usize,
        piece_data: Option<PieceType>,
        move_type: MoveType,
    ) -> Self {
        //assert!((piece_data == None) == (move_type != MoveType::Promotion));
        //assert! hack
        match piece_data {
            Some(_) => match move_type {
                MoveType::Promotion => {}
                _________ => {
                    panic!("ChessMove::new() error: not a promotion MoveType with Some(piece_data)")
                }
            },
            None => match move_type {
                MoveType::Promotion => {
                    panic!("ChessMove::new() error: promotion MoveType with piece_data = None!")
                }
                _________ => {}
            },
        }

        let mut data: u16 = 0;
        data |= (((source_index << 0) & 0b111111) | ((target_index << 6) & 0b111111_000000)) as u16;
        if piece_data.is_some() {
            let piece_data: usize = match piece_data {
                Some(PieceType::Knight) => 0b00,
                Some(PieceType::Bishop) => 0b01,
                Some(PieceType::Rook) => 0b10,
                Some(PieceType::Queen) => 0b11,
                _ => panic!("set_piece_data error: invalid piece_data!"),
            };
            data |= ((piece_data << 12) & 0b00_11_000000_000000) as u16;
        }
        let move_type_data: usize = match move_type {
            MoveType::Normal => 0,
            MoveType::Castle => 1,
            MoveType::EnPassant => 2,
            MoveType::Promotion => 3,
        };
        data |= ((move_type_data << 14) & 0b11_00_000000_000000) as u16;
        Self { data }
    }

    pub fn print_move(&self) -> String {
        if self.piece().is_some() {
            let piece = self.piece().unwrap();
            format!("{}{}{}", SQUARE_SYM[self.source()], SQUARE_SYM[self.target()], piece.to_char())
        } else {
            format!("{}{}", SQUARE_SYM[self.source()], SQUARE_SYM[self.target()])
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MovesArray {
    data: [Option<ChessMove>; 256], //note: possible moves in any given position is less than 256
    count: usize,
}

impl MovesArray {
    pub const fn new() -> Self {
        Self { data: [None; 256], count: 0 }
    }

    pub const fn new_add(&self, chess_move: ChessMove) -> MovesArray {
        let mut data = self.data;
        data[self.count] = Some(chess_move);
        MovesArray { data, count: self.count + 1 }
    }

    pub const fn new_raw(
        &self,
        source: usize,
        target: usize,
        piece_data: Option<PieceType>,
        move_type: MoveType,
    ) -> MovesArray {
        let chess_move = ChessMove::new(source, target, piece_data, move_type);
        self.new_add(chess_move)
    }

    pub const fn new_pop(&self) -> (MovesArray, Option<ChessMove>) {
        if self.count == 0 {
            return (*self, None);
        } else {
            let chess_move = self.data[self.count - 1];
            let mut data = self.data;
            data[self.count - 1] = None;
            return (MovesArray { data, count: self.count - 1 }, chess_move);
        }
    }

    pub const fn new_promotions(&self, source: usize, target: usize) -> Self {
        let mut new_arr = self.const_clone();
        new_arr.data[new_arr.count + 0] =
            Some(ChessMove::new(source, target, Some(PieceType::Queen), MT::Promotion));
        new_arr.data[new_arr.count + 1] =
            Some(ChessMove::new(source, target, Some(PieceType::Rook), MT::Promotion));
        new_arr.data[new_arr.count + 2] =
            Some(ChessMove::new(source, target, Some(PieceType::Bishop), MT::Promotion));
        new_arr.data[new_arr.count + 3] =
            Some(ChessMove::new(source, target, Some(PieceType::Knight), MT::Promotion));
        new_arr.count += 4;
        new_arr
    }
    pub fn push(&mut self, chess_move: ChessMove) {
        self.data[self.count] = Some(chess_move);
        self.count += 1;
    }

    pub fn pop(&mut self) -> Option<ChessMove> {
        if self.count == 0 {
            return None;
        } else {
            let chess_move = self.data[self.count - 1];
            self.data[self.count - 1] = None;
            self.count = self.count - 1;
            return chess_move;
        }
    }
    pub fn to_vec(&self) -> Vec<ChessMove> {
        self.data[0..self.len()].into_iter().map(|x| x.unwrap()).collect()
    }

    pub fn sort(&mut self) {
        let moves_vec: Vec<ChessMove> = self.to_vec();
        let str_vec =
            moves_vec.clone().into_iter().map(|x| format!("{}", x)).collect::<Vec<String>>();
        let mut pair_vec: Vec<(String, ChessMove)> =
            str_vec.into_iter().zip(moves_vec.into_iter()).collect();
        pair_vec.sort();
        for i in 0..self.len() {
            self.data[i] = Some(pair_vec[i].1);
        }
    }

    pub const fn const_clone(&self) -> MovesArray {
        MovesArray { data: self.data, count: self.count }
    }

    pub const fn len(&self) -> usize {
        self.count
    }
    pub const fn data(&self) -> [Option<ChessMove>; 256] {
        self.data
    }
}
