use std::{iter::Peekable, slice::Iter};

use crate::utils::UtilityStruct;

#[derive(Debug)]
pub struct SetCommand {
    pub key: String,
    pub value: String,
    pub nx: bool,
    pub xx: bool,
}

impl SetCommand {
    pub fn new(message_iter: &mut Peekable<Iter<'_, &str>>) -> SetCommand {
        let key = UtilityStruct::split_pair(message_iter).unwrap();
        let value = UtilityStruct::split_pair(message_iter).unwrap();
        let mut com = SetCommand { key, value, nx: false, xx: false };

        while let Some(&option) = message_iter.next() {
            match option.to_lowercase().as_str() {
                "nx" => {
                    com.nx = true;
                },
                "xx" => {
                    com.xx = true;
                }
                _ => {}
            }
        }
        
        com
    }
}