//! This crate provides a lexer for Ry programming language.
//!
//! Lexer is a first stage of compilation, state machine that converts
//! source text into [`type@Token`]s.
//!
//! See [`Lexer`] for more information.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unnested_or_patterns
)]

use std::{mem, str::Chars, string::String};

use ry_ast::{
    token::{LexError, RawLexError, RawToken, Token, RESERVED},
    Token,
};
use ry_filesystem::span::Span;
use ry_interner::{Interner, Symbol};
use ry_stable_likely::unlikely;

mod number;

/// Represents a lexer state machine.
/// Lexer is fairly standart. It returns [`type@Token`] and then advances its state on
/// each iteration and stops at eof (always returns [`EndOfFile`]).
/// ```
/// # use ry_lexer::Lexer;
/// # use ry_ast::token::{Token, RawToken::EndOfFile};
/// # use ry_interner::Interner;
/// # use ry_filesystem::span::Span;
/// let mut interner = Interner::default();
/// let mut lexer = Lexer::new("", &mut interner);
///
/// assert_eq!(
///     lexer.next_token(),
///     Token {
///         raw: EndOfFile,
///         span: Span { start: 0, end: 1 }
///     }
/// );
/// ```
///
/// If error appeared in the process, [`Error`] token will be returned:
///
/// ```
/// # use ry_lexer::Lexer;
/// # use ry_ast::token::{RawLexError, RawToken::Error};
/// # use ry_interner::Interner;
/// let mut interner = Interner::default();
/// let mut lexer = Lexer::new("١", &mut interner);
///
/// assert_eq!(lexer.next_token().raw, Error(RawLexError::UnexpectedChar));
/// ```
///
/// # Note
/// The lexer makes use of the `ry_interner` crate to perform string interning,
/// a process of deduplicating strings, which can be highly beneficial when dealing with
/// identifiers.
///
/// [`EndOfFile`]: ry_ast::token::RawToken::EndOfFile
/// [`Error`]: ry_ast::token::RawToken::Error
#[derive(Debug)]
pub struct Lexer<'source, 'interner> {
    /// Content of the file being scanned.
    source: &'source str,

    /// Identifier interner.
    pub interner: &'interner mut Interner,

    /// Current character.
    current: char,
    /// Next character.
    next: char,

    /// Iterator through source text characters.
    chars: Chars<'source>,

    /// Location of the current character being processed.
    location: usize,

    /// Symbol corresponding to an identifier being processed early on.
    pub scanned_identifier: Symbol,
    /// Buffer for storing scanned characters (after processing escape sequences).
    pub scanned_char: char,
    /// Buffer for storing scanned strings (after processing escape sequences).
    scanned_string: String,
}

impl<'source, 'interner> Lexer<'source, 'interner> {
    /// Creates a new [`Lexer`] instance.
    #[must_use]
    pub fn new(source: &'source str, interner: &'interner mut Interner) -> Self {
        let mut chars = source.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            source,
            current,
            next,
            chars,
            interner,
            location: 0,
            scanned_identifier: 0,
            scanned_char: '\0',
            scanned_string: String::new(),
        }
    }

    /// Returns a string being scanned early on (after processing escape sequences) and
    /// cleans internal lexer string buffer. So it must be used only once!
    #[inline]
    #[must_use]
    pub fn scanned_string(&mut self) -> String {
        mem::take(&mut self.scanned_string)
    }

    /// Returns a string being scanned early on (after processing escape sequences).
    #[inline]
    #[must_use]
    pub fn scanned_string_slice(&self) -> &str {
        &self.scanned_string
    }

    /// Returns `true` if current character is EOF (`\0`).
    #[inline]
    const fn eof(&self) -> bool {
        self.current == '\0'
    }

    /// Skips whitespace characters. See [`Lexer::is_whitespace()`] for more details.
    fn eat_whitespaces(&mut self) {
        while is_whitespace(self.current) {
            self.advance();
        }
    }

    /// Advances the lexer state to the next character.
    fn advance(&mut self) {
        let previous = self.current;

        self.current = self.next;
        self.next = self.chars.next().unwrap_or('\0');

        self.location += previous.len_utf8();
    }

    /// Advances the lexer state to the next 2 characters
    /// (calls [`Lexer::advance()`] twice).
    #[inline]
    fn advance_twice(&mut self) {
        self.advance();
        self.advance();
    }

    /// Advances the lexer state to the next character, and returns the token
    /// with location being the current character location in the source text.
    fn advance_with(&mut self, raw: RawToken) -> Token {
        let token = Token {
            raw,
            span: self.current_char_span(),
        };

        self.advance();
        token
    }

    /// Returns a span of the current character.
    #[inline]
    const fn current_char_span(&self) -> Span {
        Span {
            start: self.location,
            end: self.location + 1,
        }
    }

    /// Returns a span ending with the current character's location.
    const fn span_from(&self, start_location: usize) -> Span {
        Span {
            start: start_location,
            end: self.location,
        }
    }

    /// Advances the lexer state to the next 2 characters, and returns the token
    /// with location being `self.location..self.location + 2`.
    fn advance_twice_with(&mut self, raw: RawToken) -> Token {
        let token = Token {
            raw,
            span: Span {
                start: self.location,
                end: self.location + 2,
            },
        };

        self.advance_twice();
        token
    }

    /// Advances the lexer state to the next character while `f` returns `true`,
    /// where its arguments are the current and next characters.
    /// Returns the string slice of source text between `start_location`
    /// and `self.location` when `f` returns `false` OR `self.eof() == true`.
    #[inline]
    fn advance_while<F>(&mut self, start_location: usize, mut f: F) -> &'source str
    where
        F: FnMut(char, char) -> bool,
    {
        while f(self.current, self.next) && !self.eof() {
            self.advance();
        }

        &self.source[start_location..self.location]
    }

    /// Parses an escape sequence.
    fn eat_escape(&mut self) -> Result<char, LexError> {
        self.advance(); // `\`
        let r = match self.current {
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '\0' => Err(LexError {
                raw: RawLexError::EmptyEscapeSequence,
                span: self.current_char_span(),
            }),
            'u' => {
                self.advance();

                if self.current != '{' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..4 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInUnicodeEscapeSequence,
                            span: self.current_char_span(),
                        });
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    }),
                }
            }
            'U' => {
                self.advance();

                if self.current != '{' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..8 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInUnicodeEscapeSequence,
                            span: self.current_char_span(),
                        });
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        span: self.current_char_span(),
                    }),
                }
            }
            'x' => {
                self.advance();

                if self.current != '{' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInByteEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..2 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInByteEscapeSequence,
                            span: self.current_char_span(),
                        });
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInByteEscapeSequence,
                        span: self.current_char_span(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidByteEscapeSequence,
                        span: Span {
                            start: self.location - 4,
                            end: self.location,
                        },
                    }),
                }
            }
            _ => Err(LexError {
                raw: RawLexError::UnknownEscapeSequence,
                span: self.current_char_span(),
            }),
        };

        self.advance();

        r
    }

    /// Parses a char literal.
    fn eat_char(&mut self) -> Token {
        let start_location = self.location;

        self.advance();

        let mut size = 0;

        while self.current != '\'' {
            if self.current == '\n' || self.eof() {
                return Token {
                    raw: RawToken::Error(RawLexError::UnterminatedCharLiteral),
                    span: self.span_from(start_location),
                };
            }

            if self.current == '\\' {
                let e = self.eat_escape();

                match e {
                    Ok(c) => {
                        self.scanned_char = c;
                    }
                    Err(e) => {
                        return Token {
                            span: e.span,
                            raw: RawToken::from(e.raw),
                        }
                    }
                }
            } else {
                self.scanned_char = self.current;
                self.advance();
            }

            size += 1;
        }

        self.advance();

        match size {
            2..=usize::MAX => {
                return Token {
                    raw: RawToken::Error(RawLexError::MoreThanOneCharInCharLiteral),
                    span: self.span_from(start_location),
                };
            }
            0 => {
                return Token {
                    raw: RawToken::Error(RawLexError::EmptyCharLiteral),
                    span: self.span_from(start_location),
                };
            }
            _ => {}
        }

        Token {
            raw: RawToken::CharLiteral,
            span: self.span_from(start_location),
        }
    }

    /// Parses a string literal.
    fn eat_string(&mut self) -> Token {
        self.scanned_string.clear();
        let start_location = self.location;

        self.advance();

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            if c == '\\' {
                let e = self.eat_escape();

                match e {
                    Ok(c) => {
                        self.scanned_string.push(c);
                    }
                    Err(e) => {
                        return Token {
                            span: e.span,
                            raw: RawToken::from(e.raw),
                        }
                    }
                }
            } else {
                self.scanned_string.push(c);
                self.advance();
            }
        }

        if self.eof() || self.current == '\n' {
            return Token {
                raw: RawToken::Error(RawLexError::UnterminatedStringLiteral),
                span: self.span_from(start_location),
            };
        }

        self.advance();

        Token {
            raw: RawToken::StringLiteral,
            span: self.span_from(start_location),
        }
    }

    /// Parses a wrapped identifier.
    fn eat_wrapped_id(&mut self) -> Token {
        let start_location = self.location;

        self.advance();

        let name = &self.advance_while(start_location, |current, _| {
            current.is_alphanumeric() || current == '_'
        })[1..];

        if self.current != '`' {
            return Token {
                raw: RawToken::Error(RawLexError::UnterminatedWrappedIdentifier),
                span: self.span_from(start_location),
            };
        }

        if name.is_empty() {
            return Token {
                raw: RawToken::Error(RawLexError::EmptyWrappedIdentifier),
                span: self.span_from(start_location),
            };
        }

        self.advance();

        self.scanned_identifier = self.interner.get_or_intern(name);

        Token {
            raw: RawToken::Identifier,
            span: self.span_from(start_location),
        }
    }

    /// Parses a usual comment (prefix is `//`).
    fn eat_comment(&mut self) -> Token {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance();

        self.advance_while(start_location + 2, |current, _| (current != '\n'));

        Token {
            raw: RawToken::Comment,
            span: self.span_from(start_location),
        }
    }

    /// Parses a doc comment.
    ///
    /// When [`global`] is true,  doc comment is describing
    /// the whole module (3-rd character is `!`) and
    /// when not doc comment is corresponding to trait method, enum variant, etc.
    /// (everything else and the character is `/`).
    fn eat_doc_comment(&mut self, global: bool) -> Token {
        // first `/` character is already consumed
        let start_location = self.location - 1;
        self.advance_twice(); // `/` and (`!` or `/`)

        self.advance_while(start_location + 3, |current, _| (current != '\n'));

        Token {
            span: self.span_from(start_location),
            raw: if global {
                RawToken::GlobalDocComment
            } else {
                RawToken::LocalDocComment
            },
        }
    }

    /// Parses weather an identifier or a keyword.
    fn eat_name(&mut self) -> Token {
        let start_location = self.location;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        if let Some(reserved) = RESERVED.get(name) {
            Token {
                raw: *reserved,
                span: self.span_from(start_location),
            }
        } else {
            self.scanned_identifier = self.interner.get_or_intern(name);
            Token {
                raw: RawToken::Identifier,
                span: self.span_from(start_location),
            }
        }
    }

    /// Works the same as [`Lexer::next_token`], but skips comments ([`RawToken::Comment`]).
    pub fn next_no_comments(&mut self) -> Token {
        loop {
            let t = self.next_token();
            if t.raw != RawToken::Comment {
                return t;
            }
        }
    }

    /// Proceeds to the next token and returns it (see [top level documentation](../index.html) for more details).
    pub fn next_token(&mut self) -> Token {
        self.eat_whitespaces();

        if unlikely(self.current == '\0') {
            return Token {
                raw: RawToken::EndOfFile,
                span: self.current_char_span(),
            };
        }

        match (self.current, self.next) {
            (':', _) => self.advance_with(Token![:]),
            ('@', _) => self.advance_with(Token![@]),

            ('"', _) => self.eat_string(),
            ('\'', _) => self.eat_char(),
            ('`', _) => self.eat_wrapped_id(),

            ('+', '+') => self.advance_twice_with(Token![++]),
            ('+', '=') => self.advance_twice_with(Token![+=]),
            ('+', _) => self.advance_with(Token![+]),
            ('-', '-') => self.advance_twice_with(Token![--]),
            ('-', '=') => self.advance_twice_with(Token![-=]),
            ('-', _) => self.advance_with(Token![-]),
            ('*', '*') => self.advance_twice_with(Token![**]),
            ('*', '=') => self.advance_twice_with(Token![*=]),
            ('*', _) => self.advance_with(Token![*]),

            ('#', _) => self.advance_with(Token![#]),

            ('/', '/') => {
                self.advance();

                match self.next {
                    '!' => self.eat_doc_comment(true),
                    '/' => self.eat_doc_comment(false),
                    _ => self.eat_comment(),
                }
            }

            ('/', '=') => self.advance_twice_with(Token![/=]),
            ('/', _) => self.advance_with(Token![/]),
            ('!', '=') => self.advance_twice_with(Token![!=]),
            ('!', _) => self.advance_with(Token![!]),
            ('>', '>') => self.advance_twice_with(Token![>>]),
            ('>', '=') => self.advance_twice_with(Token![>=]),
            ('>', _) => self.advance_with(Token![>]),
            ('<', '<') => self.advance_twice_with(Token![<<]),
            ('<', '=') => self.advance_twice_with(Token![<=]),
            ('<', _) => self.advance_with(Token![<]),
            ('=', '=') => self.advance_twice_with(Token![==]),
            ('=', '>') => self.advance_twice_with(Token![=>]),
            ('=', _) => self.advance_with(Token![=]),
            ('|', '=') => self.advance_twice_with(Token![|=]),
            ('|', '|') => self.advance_twice_with(Token![||]),
            ('|', _) => self.advance_with(Token![|]),
            ('?', _) => self.advance_with(Token![?]),
            ('&', '&') => self.advance_twice_with(Token![&&]),
            ('&', _) => self.advance_with(Token![&]),
            ('^', '=') => self.advance_twice_with(Token![^=]),
            ('^', _) => self.advance_with(Token![^]),
            ('~', _) => self.advance_with(Token![~]),
            ('(', _) => self.advance_with(Token!['(']),
            (')', _) => self.advance_with(Token![')']),
            ('[', _) => self.advance_with(Token!['[']),
            (']', _) => self.advance_with(Token![']']),
            ('{', _) => self.advance_with(Token!['{']),
            ('}', _) => self.advance_with(Token!['}']),
            (',', _) => self.advance_with(Token![,]),
            (';', _) => self.advance_with(Token![;]),
            ('%', '=') => self.advance_with(Token![%=]),
            ('%', _) => self.advance_with(Token![%]),

            ('.', '.') => self.advance_twice_with(Token![..]),

            (c, n) => {
                if number::decimal(c) || (c == '.' && number::decimal(n)) {
                    return self.eat_number();
                } else if is_id_start(c) {
                    return self.eat_name();
                } else if c == '.' {
                    return self.advance_with(Token![.]);
                }

                self.advance_with(RawToken::Error(RawLexError::UnexpectedChar))
            }
        }
    }
}

/// True if `c` is a whitespace.
const fn is_whitespace(c: char) -> bool {
    // Note that it is ok to hard-code the values, because
    // the set is stable and doesn't change with different
    // Unicode versions.
    matches!(
        c,
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

/// True if `c` is valid as a first character of an identifier.
fn is_id_start(c: char) -> bool {
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

/// True if `c` is valid as a non-first character of an identifier.
fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}
