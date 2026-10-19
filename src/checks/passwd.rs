use crate::model::{ParseError, PasswdEntry};

pub fn parse_passwd_line (input: &str) -> Result<PasswdEntry, ParseError> {
    let fields: Vec<&str> = input.split(':').collect();

    if fields.len() != 7 {
        return Err(ParseError::InvalidFieldCount);
    }
        
    todo!()
}