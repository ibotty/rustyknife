//! [Header extensions for non-ASCII text]
//!
//! [Header extensions for non-ASCII text]: https://tools.ietf.org/html/rfc2047

use std::borrow::Cow;

use encoding_rs::{Encoding, UTF_8}; // TODO: was ASCII

use base64::prelude::{Engine as _, BASE64_STANDARD};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::util::*;
use crate::rfc3461::hexpair;

fn token(input: &[u8]) -> NomResult<&[u8]> {
    take_while1(|c| (33..=126).contains(&c) && !b"()<>@,;:\\\"/[]?.=".contains(&c))(input)
}

fn encoded_text(input: &[u8]) -> NomResult<&[u8]> {
    take_while1(|c| match c {33..=62 | 64..=126 => true, _ => false})(input)
}

fn _qp_encoded_text(input: &[u8]) -> NomResult<Vec<u8>> {
    many0(alt((
        preceded(tag("="), hexpair),
        map(tag("_"), |_| b' '),
        take1_filter(|_| true),
    )))(input)
}

// Decode the modified quoted-printable as defined by this RFC.
fn decode_qp(input: &[u8]) -> Option<Vec<u8>>
{
    exact!(input, _qp_encoded_text).ok().map(|(_, o)| o)
}

// Undoes the quoted-printable or base64 encoding.
fn decode_text(encoding: &[u8], text: &[u8]) -> Option<Vec<u8>>
{
    match &encoding.to_ascii_lowercase()[..] {
        b"q" => decode_qp(text),
        b"b" => BASE64_STANDARD.decode(&text).ok(),
        _ => None,
    }
}

fn _encoded_word(input: &[u8]) -> NomResult<(Cow<str>, Vec<u8>)> {
    map(tuple((preceded(tag("=?"), token),
               opt(preceded(tag("*"), token)),
               delimited(tag("?"), token, tag("?")),
               terminated(encoded_text, tag("?=")))),
        |(charset, _lang, encoding, text)| {
            (charset::decode_ascii(charset), decode_text(encoding, text).unwrap_or_else(|| text.to_vec()))
        })(input)
}

fn decode_charset((charset, bytes): (Cow<str>, Vec<u8>)) -> String
{
    Encoding::for_label(charset.as_bytes()).unwrap_or(UTF_8).decode_without_bom_handling(&bytes).0.to_string()
}

/// Decode an encoded word.
///
/// # Examples
/// ```
/// use rustyknife::rfc2047::encoded_word;
///
/// let (_, decoded) = encoded_word(b"=?x-sjis?B?lEWWQI7Kg4GM9ZTygs6CtSiPzik=?=").unwrap();
/// assert_eq!(decoded, "忍法写メ光飛ばし(笑)");
/// ```
pub fn encoded_word(input: &[u8]) -> NomResult<String> {
    map(_encoded_word, decode_charset)(input)
}
