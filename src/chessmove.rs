#![allow(dead_code)]
#![allow(long_running_const_eval)]

/*
    binary masks           description         hexidecimal masks
    0000 0000 0011 1111    source square       0x3f
    0000 1111 1100 0000    target square       0xfc0
    0011 0000 0000 0000    promoted piece      0x3000
    1100 0000 0000 0000    move type           0xc000

    note: move types are encoded as follows
    00 - normal move
    01 - castle move
    10 - en passant
    11 - promotion
*/
#[derive(Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub data: u16
}

pub type CM = ChessMove;

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
        ((self.data & 0x3fu16) as usize) >> 0
    }
    pub const fn get_target_index(&self) -> usize {
        ((self.data & 0xfc0u16) as usize) >> 6
    }
    pub const fn get_piece_data(&self) -> usize {
        ((self.data & 0x3000u16) as usize) >> 12
    }
    pub const fn get_move_type(&self) -> MoveType {
        match ((self.data & 0xc000u16) as usize) >> 14 {
            0 => MT::Normal,
            1 => MT::Castle,
            2 => MT::EnPassant,
            3 => MT::Promotion,
            _ => panic!("get_move_type error: invalid move type data!"),
        }
    }
    pub const fn get_move_type_raw(&self) -> usize {
        ((self.data & 0xc000u16) as usize) >> 14 
    }

    // set functions
    pub fn set_source_index(&mut self, index: usize) {
        self.data &= ((index << 0) & 0x3f) as u16;
    }
    pub fn set_target_index(&mut self, index: usize) {
        self.data &= ((index << 6) & 0xfc0) as u16;
    }
    pub fn set_piece_data(&mut self, piece_data: usize) {
        self.data &= ((piece_data << 12) & 0x3000) as u16;
    }
    pub fn set_move_type(&mut self, move_type: MoveType) {
        let move_type_data = match move_type {
            MT::Normal => 0,
            MT::Castle => 1,
            MT::EnPassant => 2,
            MT::Promotion => 3,
        };
        self.data &= ((move_type_data  << 14) & 0xc000) as u16;
    }
    pub fn set_move_type_raw(&mut self, move_type: usize) {
        self.data &= ((move_type  << 14) & 0xc000) as u16;
    }
    
    // compound setters
    pub fn set_data(&mut self, source_index: usize, target_index: usize, piece_data: usize, move_type: MoveType) {
        self.set_source_index(source_index);
        self.set_target_index(target_index);
        self.set_piece_data(piece_data);
        self.set_move_type(move_type);
    }
    pub fn set_data_raw(&mut self, source_index: usize, target_index: usize, piece_data: usize, move_type: usize) {
        self.set_source_index(source_index);
        self.set_target_index(target_index);
        self.set_piece_data(piece_data);
        self.set_move_type_raw(move_type);
    }
}

//probably unecessary
pub type MoveList = Vec<ChessMove>; 

impl ChessMove {
    pub fn print_move(&self) -> String {
        let string: String = String
    }
}
