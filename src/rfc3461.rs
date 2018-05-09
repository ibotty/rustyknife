//! SMTP DSN extension

use std::str;

use util::*;

use nom::is_hex_digit;
use rfc5322::atom;

named!(pub hexpair<CBS, u8>,
    map_res!(take_while_m_n!(2, 2, is_hex_digit),
             |x: CBS| u8::from_str_radix(str::from_utf8(x.0).unwrap(), 16))
);

named!(hexchar<CBS, u8>,
    do_parse!(
        tag!("+") >>
        a: hexpair >>
        (a)
    )
);

named!(xchar<CBS, CBS>,
    take_while1!(|c: u8| (33..=42).contains(&c) || (44..=60).contains(&c) || (62..=126).contains(&c))
);

named!(pub xtext<CBS, Vec<u8>>,
    fold_many0!(alt!(
        map!(xchar, |x| x.0.to_vec()) |
        map!(hexchar, |x| vec![x])), Vec::new(), |mut acc: Vec<_>, x| {acc.extend(x); acc} )
);

named!(_printable_xtext<CBS, Vec<u8>>,
    map_opt!(xtext, |out: Vec<_>| {
        if out.iter().all(|c: &u8| (32..=126).contains(c) || b"\t\x0a\x0b\x0c\x0d".contains(c)) {
            Some(out)
        } else {
            None
        }
    })
);

named!(_original_recipient_address<CBS, (String, String)>,
    do_parse!(
        a: atom >> tag!(";") >> b: _printable_xtext >>
        ((ascii_to_string(a.0), ascii_to_string(&b)))
    )
);

#[derive(Debug)]
pub enum DSNRet {
    Full,
    Hdrs,
}

#[derive(Debug)]
pub struct DSNMailParams {
    pub envid: Option<String>,
    pub ret: Option<DSNRet>,
}

pub fn dsn_mail_params<'a>(input: Vec<(&'a str, Option<&'a str>)>) -> Result<(DSNMailParams, Vec<(&'a str, Option<&'a str>)>), &'static str>
{
    let mut out = Vec::new();
    let mut envid_val : Option<String> = None;
    let mut ret_val : Option<DSNRet> = None;

    for (name, value) in input {
        match name.to_lowercase().as_str() {
            "ret" => {
                if ret_val.is_some() { return Err("Duplicate RET"); }
                if value.is_none() { return Err("RET without value"); }

                ret_val = match value.unwrap().to_lowercase().as_str() {
                    "full" => Some(DSNRet::Full),
                    "hdrs" => Some(DSNRet::Hdrs),
                    _ => return Err("Invalid RET")
                }
            },
            "envid" => {
                if envid_val.is_some() { return Err("Duplicate ENVID"); }
                if value.is_none() { return Err("ENVID without value"); }
                let inascii = string_to_ascii(value.unwrap());
                if inascii.len() > 100 {
                    return Err("ENVID over 100 bytes");
                }
                if let Ok((_, parsed)) = exact!(CBS(&inascii), _printable_xtext) {
                    envid_val = Some(ascii_to_string(&parsed));
                } else {
                    return Err("Invalid ENVID");
                }
            },
            _ => {
                out.push((name, value))
            }
        }
    }

    Ok((DSNMailParams{envid: envid_val, ret: ret_val}, out))
}

pub fn orcpt_address(input: &Vec<u8>) -> KResult<&[u8], (String, String)>
{
    wrap_cbs_result(_original_recipient_address(CBS(input)))
}