use std::borrow::Cow;
use std::ops::Deref;
use std::str;

use nom;
use nom::types::CompleteByteSlice;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::ASCII;

pub(crate) type KResult<I, O, E = u32> = Result<(I, O), nom::Err<I, E>>;
pub type CBS<'a> = CompleteByteSlice<'a>;

#[allow(non_snake_case)]
pub fn CBS(input: &[u8]) -> CBS {
    CompleteByteSlice(input)
}

pub fn ascii_to_string<'a, T: Deref<Target=&'a [u8]>>(i: T) -> Cow<'a, str> {
    String::from_utf8_lossy(&i)
}

pub fn ascii_to_string_vec(i: Vec<u8>) -> String {
    if i.is_ascii() {
        String::from_utf8(i).unwrap()
    } else {
        ASCII.decode(&i, DecoderTrap::Replace).unwrap()
    }
}

pub fn string_to_ascii(i: &str) -> Vec<u8> {
    ASCII.encode(&i, EncoderTrap::Replace).unwrap()
}

pub fn wrap_cbs_result<T> (r: nom::IResult<CBS, T, u32>) -> nom::IResult<&[u8], T, u32> {
    r.map(|(r, o)| (r.0, o)).map_err(|e| match e {
        nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
        nom::Err::Error(c) => nom::Err::Error(convert_context(c)),
        nom::Err::Failure(c) => nom::Err::Failure(convert_context(c)),
    })
}

pub fn convert_context(c: nom::Context<CBS>) -> nom::Context<&[u8]> {
    match c {
        nom::Context::Code(r, e) => nom::Context::Code(r.0, e),
        #[cfg(feature = "nom-verbose-errors")]
        nom::Context::List(mut v) => nom::Context::List(v.drain(..).map(|(r, e)| (r.0, e)).collect()),
    }
}
