//! Grammar specification (warn: not context-free I suppose)
//! ```plaintext
//! <cmd> ::= (<component> \s?)*
//! <component> ::= (^ | \s) <pos:string>
//!               | (^ | \s) <key:ident> = <val:string>
//!               | (+ | -) <flag:ident>
//! <ident> ::= [a-zA-Z_][a-zA-Z0-9_]*
//! <string> ::= [^][\s'"{}()=+-|]*
//!            | ∀x :: ℤ+ : '{x} <string_ex_wo('{x},'{x})> '{x}
//!            | ∀x :: ℤ+ : "{x} <string_ex_wo("{x},"{x})> "{x}
//!            | ∀x :: <bracks> : <x> <string_ex_wo(x,rev(x))> <rev(x)>
//! <string_ex_wo(l,r)> : (* like <string_ex> but ends before the next <r> occurs; must not have
//!                        another occurence of <l> *)
//! <string_ex> ::= .* (* must conform to utf8 *)
//! <bracks> ::= <bracket>* (<bracket> | BAR)
//! <bracket> ::= PAREN | BRACK | BRACE
//! ```

#[derive(Clone, Debug)]
pub enum Arg<'a> {
    // A positional argument; <string>
    Pos(&'a str),
    // A keyword argument; <ident>=<string>
    Kw(&'a str, &'a str),
    // A flag argument; +<ident> (on) or -<ident> (off)
    Flag(bool, &'a str),
}

// definition section; play nicely with editor paren pairings
const QUOTE: char = '\'';
const DQUOTE: char = '"';
const BQUOTE: char = '`';
const PLUS: char = '+';
const MINUS: char = '-';
const EQUAL: char = '=';
const BAR: char = '|';
const LPAREN: char = '(';
const RPAREN: char = ')';
const LBRACK: char = '[';
const RBRACK: char = ']';
const LBRACE: char = '{';
const RBRACE: char = '}';
const BRACK_OPEN_CHARS: &'static str = "([{|"; // }])
const NAKED_STRING_BAN: &'static str = "'\"`|{[(=+-)]}";

pub fn parse<'a, 'b: 'a>(s: &'b str) -> Result<Vec<Arg<'a>>, String> {
    let s_trimmed = s.trim_end();
    let mut i = 0usize;
    let mut acc = vec![];
    while i < s_trimmed.len() {
        // let (j, a) = _expect_next_arg(s_trimmed, i)?;
        let (j, a) = match _expect_next_arg(s_trimmed, i) {
            Ok(x) => x,
            Err(m) => {
                eprintln!("{:?} | {}", acc, i);
                return Err(m);
            }
        };
        acc.push(a);
        i = j;
    }
    Ok(acc)
}

// NOTE: for anything from here, leading underscore means "may panic for improper input"

fn _expect_next_arg<'a, 'b: 'a>(s: &'b str, i: usize) -> Result<(usize, Arg<'a>), String> {
    if s.len() <= i { panic!("(internal) undetected end-of-input in _expect_next_arg"); }
    let fst = s[i..].chars().next().unwrap();
    if fst == PLUS || fst == MINUS {
        let (i, flagname) = expect_ident(s, i + fst.len_utf8())?;
        Ok((i, Arg::Flag(fst == PLUS, flagname)))
    } else {
        let mut j = i;
        for c in s[i..].chars().take_while(|c| c.is_whitespace()) {
            j += c.len_utf8();
        }
        if 0 < i && i == j {
            let fst = s[i..].chars().next().unwrap();
            Err(format!("Expected flag, or some whitespace before more input; found {:?}", fst))
        } else {
            _expect_immediate_arg(s, j)
        }
    }
}

fn _expect_immediate_arg<'a, 'b: 'a>(s: &'b str, i: usize) -> Result<(usize, Arg<'a>), String> {
    if s.len() <= i {
        panic!("(internal) undetected end-of-input in _expect_immediate_arg");
    }
    let fst = s[i..].chars().next().unwrap();
    if fst.is_whitespace() {
        panic!("(internal) unhandled whitespace in _expect_immediate_arg");
    } else if fst == PLUS || fst == MINUS {
        let enabled = fst == PLUS;
        let (i, flagname) = expect_ident(s, i + fst.len_utf8())?;
        Ok((i, Arg::Flag(enabled, flagname)))
    } else {
        let mut found_eq = false;
        let (i, a) = expect_ident(s, i)
            .and_then(|(i, ident)| if let Some('=') = s[i..].chars().next() {
                found_eq = true;
                expect_string(s, i + '='.len_utf8())
                    .map(|(i, x)| (i, Arg::Kw(ident, x)))
            } else {
                Err("Didn't find <ident>= (this error message shouldn't show up)".to_owned())
            })
            .or_else(|m1| {
                expect_string(s, i)
                    .map(|(i, x)| (i, Arg::Pos(x)))
                    .map_err(|m2| if found_eq { m1 } else { m2 })
            })?;
        if let Some(c) = s[i..].chars().next() {
            if !c.is_whitespace() {
                Err("Strings should be followed by whitespace or end-of-input".to_owned())
            } else {
                Ok((i, a))
            }
        } else {
            Ok((i, a))
        }
    }
    // } else if let Ok((i, ident)) = expect_followed(s, i, expect_ident, |s, i| expect_exact(s, i, "=")) {
    //     let (i, value) = expect_followed(
    //         s, i, expect_string,
    //         |s, i|
    //             if s.len() == i || s[i..].chars().next().map(char::is_whitespace).unwrap_or(false) { Ok(i) }
    //             else { Err("Strings should always be followed with whitespace or end-of-input".to_owned()) },
    //     )?;
    //     Ok((i, Arg::Kw(ident, value)))
    // } else {
    //     let (i, content) = expect_followed(
    //         s, i, expect_string,
    //         |s, i|
    //             if s.len() == i || s[i..].chars().next().map(char::is_whitespace).unwrap_or(false) { Ok(i) }
    //             else { Err("Strings should always be followed with whitespace or end-of-input".to_owned()) },
    //     )?;
    //     Ok((i, Arg::Pos(content)))
    // }
}

fn expect_ident<'a, 'b: 'a>(s: &'b str, i: usize) -> Result<(usize, &'a str), String> {
    fn qualify_ident_char_helper(c: char, isfirst: bool) -> bool {
        c == '_' || c.is_ascii_alphabetic() || (!isfirst && c.is_ascii_digit())
    }
    let mut j = i;
    let mut isfst = true;
    for c in s[i..].chars() {
        if !qualify_ident_char_helper(c, isfst) { break; }
        isfst = false;
        j += c.len_utf8();
    }
    if i == j {
        if let Some(fst) = s[i..].chars().next() {
            Err(format!("Expected ASCII letter to begin identifier; found {:?}", fst))
        } else {
            Err("Expected ASCII letter to begin identifier; found end-of-input".to_owned())
        }
    } else {
        Ok((j, &s[i..j]))
    }
}

fn expect_string<'a, 'b: 'a>(s: &'b str, i: usize) -> Result<(usize, &'a str), String> {
    if s.len() <= i {
        return Err("Expected string, found end of input".to_owned());
    }
    let fst = s[i..].chars().next().unwrap();
    let (i, open) = if BRACK_OPEN_CHARS.chars().any(|c| c == fst) {
        _expect_brackplus(s, i)
    } else if fst == QUOTE || fst == DQUOTE || fst == BQUOTE {
        _expect_quoteplus(s, i)
    } else if fst == PLUS || fst == MINUS || fst == EQUAL {
        return Err(format!("Expected string, found {:?}", fst));
    } else {
        (i, "")
    };
    if open.len() == 0 {
        expect_naked_string(s, i)
    } else {
        let close = reversed_string_delimiter(open);
        expect_string_ex_wo(s, i, open, &close)
    }
}

#[inline]
fn reversed_string_delimiter(d: &str) -> String {
    d.chars().rev().map(|c| match c {
        LPAREN => RPAREN,
        LBRACK => RBRACK,
        LBRACE => RBRACE,
        BAR | QUOTE | DQUOTE | BQUOTE => c,
        _ => panic!("(internal) bad string delim {:?}", c),
    }).collect()
}

#[inline]
fn _expect_quoteplus(s: &str, i: usize) -> (usize, &str) {
    let mut j = i;
    if s.len() <= i { panic!("(internal) empty at _expect_quoteplus"); }
    let c = match s[i..].chars().take(1).next().unwrap() {
        c @ (QUOTE | BQUOTE | DQUOTE) => c,
        _ => panic!("(internal) non-quote at _expect_quoteplus"),
    };
    let len1 = c.len_utf8();
    for _ in s[i..].chars().take_while(|d| *d == c) {
        j += len1;
    }

    (j, &s[i..j])
}

#[inline]
fn _expect_brackplus(s: &str, i: usize) -> (usize, &str) {
    let mut j = i;
    for c in s[i..].chars() {
        if BRACK_OPEN_CHARS.chars().all(|d| c != d) { break; }
        j += c.len_utf8();
        if c == BAR { break; } // stop at first '|'
    }
    if i == j { panic!("(internal) empty at _expect_brackplus"); }

    (j, &s[i..j])
}

struct Kmp<'a>(&'a str, Vec<usize>, usize);
impl<'a> Kmp<'a> {
    pub fn new<'b: 'a>(template: &'a str) -> Self {
        if template.len() == 0 { return Self(template, vec![0], 0); }
        let t = template;
        let mut v = vec![0usize; t.len() + 1];
        let mut l = 0; // no characters
        let mut r = t.chars().next().unwrap().len_utf8(); // one character
        for c in t[r..].chars() {
            debug_assert!(l < r);
            let k = c.len_utf8();
            while l != 0 && c != t[l..].chars().next().unwrap() { l = v[l]; }
            if c == t[l..].chars().next().unwrap() { l += k; }
            r += k;
            debug_assert!(l < r);
            v[r] = l;
        }

        Self(t, v, 0)
    }
    #[allow(dead_code)]
    pub fn clear(&mut self) { self.2 = 0; }
    pub fn step(&mut self, c: char) -> bool {
        if self.0.len() == 0 { return true; }
        if self.2 == self.0.len() { self.2 = self.1[self.2]; }
        while self.2 != 0 && self.0[self.2..].chars().next().unwrap() != c {
            self.2 = self.1[self.2];
        }
        if self.0[self.2..].chars().next().unwrap() == c {
            self.2 += c.len_utf8();
        }

        self.is_matched()
    }
    pub fn is_matched(&self) -> bool { self.0.len() == self.2 }
}
impl<'a> AsRef<str> for Kmp<'a> {
    fn as_ref(&self) -> &str { self.0 }
}

fn expect_string_ex_wo<'a, 'b: 'a>(s: &'b str, i: usize, l: &str, r: &str) -> Result<(usize, &'a str), String> {
    let mut lkmp = Kmp::new(l);
    let mut rkmp = Kmp::new(r);
    let mut j = i;
    for c in s[i..].chars() {
        j += c.len_utf8();
        if rkmp.step(c) {
            let k = j - r.len();
            return Ok((j, &s[i..k]));
        }
        if lkmp.step(c) {
            return Err(format!("Opening delimiter (here: {:?}) not allowed in string content", l));
        }
    }

    Err(format!("Unclosed string at end-of-input (left: {:?}, expected right: {:?})", l, r))
}

fn expect_naked_string<'a, 'b: 'a>(s: &'b str, i: usize) -> Result<(usize, &'a str), String> {
    let mut j = i;
    for c in s[i..].chars() {
        if NAKED_STRING_BAN.chars().any(|d| d == c) {
            return Err(format!("In naked string, encountered banned character {:?}; try wrapping the string in quotes or delimiters.", c));
        }
        if c.is_whitespace() { break; }
        j += c.len_utf8();
    }

    Ok((j, &s[i..j]))
}
