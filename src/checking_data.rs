#![allow(dead_code)]
#![allow(long_running_const_eval)]

use crate::bitboard::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckData {
    pub data: (Side,BitBoard),
}

pub type CD = CheckData;
