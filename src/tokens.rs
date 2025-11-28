use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum TokenKind {
    #[token("~")]

    Tilde,

    #[token("`")]
    Grave,


    #[token("#")]
    Pound,

    #[token("+")]

    Plus,
    #[token("++")]
    PlusPlus,
    #[token("-")]
    Minus,
    #[token("--")]
    MinusMinus,


    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Mod,

    #[token("!")]
    Bang,

    #[token(">")]
    Greater,
    #[token(">>")]
    GreaterGreater,

    #[token("<")]
    Less,
    #[token("<<")]
    LessLess,

    #[token("&")]
    Amp,
    #[token("&&")]
    AmpAmp,

    #[token("|")]
    Pipe,

    #[token("||")]
    PipePipe,

    #[token("^")]
    Xor,

    // --- Keywords ---
    #[token("var")]
    Var,
    #[token("const")]
    Const,
    #[token("include")]
    Include,

    // --- Directives starting with '@' ---
    #[regex(r"@[A-Za-z_][A-Za-z0-9_]*")]
    AtDirective, // matches @define, @foo, @bar32, etc.

    // --- macro_rules! ---
    #[token("macro_rules!")]
    MacroRules,

    // --- for!(...) loop ---
    #[token("for!")]
    ForBang,

    // --- Identifiers ---
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // --- Literals ---
    #[regex(r"0x[0-9A-Fa-f]+", |lex| i64::from_str_radix(&lex.slice()[2..], 16).unwrap())]
    HexLit(i64),

    #[regex(r"0b[01]+", |lex| i64::from_str_radix(&lex.slice()[2..], 2).unwrap())]
    BinLit(i64),

    #[regex(r"0o[0-7]+", |lex| i64::from_str_radix(&lex.slice()[2..], 8).unwrap())]
    OctLit(i64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    IntLit(i64),

    // --- Strings ---
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StrLit(String),

    // --- Character literal ---
    #[regex(r#"'([^'\\]|\\.)'"#, |lex| {
        let s = lex.slice();

        parse_char(s) // you define this small helper fn
    })]
    CharLit(char),

    // --- Symbols & punctuation ---
    #[token("=")]
    Equal,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,

    // Labels like `.foo:` require DOT token.
    #[token(".")]
    Dot,
    
    #[token(";")]
    Semicolon,

    // Prefix for `::global:`
    #[token("::")]
    DoubleColon,

    // Ignore whitespace
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,
}

fn parse_char(s: &str) -> char {
    let inner = &s[1..s.len() - 1]; // remove quotes
    if inner.starts_with("\\") {
        match &inner[1..] {
            "n" => '\n',
            "t" => '\t',

            "r" => '\r',
            "'" => '\'',
            "\\" => '\\',
            _ => panic!("unknown escape {}", inner),
        }
    } else {
        inner.chars().next().unwrap()
    }
}

fn parse_content(content: &str) -> i64 {
    if content.starts_with("0x") || content.starts_with("0X") {
        i64::from_str_radix(&content[2..], 16).unwrap()
    } else if content.starts_with("0b") || content.starts_with("0B") {
        i64::from_str_radix(&content[2..], 2).unwrap()
    } else if content.starts_with("0o") || content.starts_with("0O") {
        i64::from_str_radix(&content[2..], 8).unwrap()
    } else if content.starts_with("'") && content.ends_with("'") {
        let char_content = &content[1..content.len() - 1];
        if char_content.len() == 1 {
            char_content.chars().next().unwrap() as i64
        } else if char_content.starts_with('\\') {
            match char_content {
                "\\n" => '\n' as i64,
                "\\t" => '\t' as i64,
                "\\r" => '\r' as i64,
                "\\0" => '\0' as i64,
                "\\'" => '\'' as i64,
                "\\\"" => '\"' as i64,
                "\\\\" => '\\' as i64,
                _ => '\\' as i64,
            }
        } else {
            -1
        }
    } else if content.chars().all(|c| c.is_ascii_digit() || c == '-') {
        content.parse::<i64>().unwrap()
    } else {
        -1
    }
}
fn parse_string(s: &str) -> String {
    let inner = &s[1..s.len() - 1];
    let mut result = String::new();
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('0') => result.push('\0'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('\"'),
                Some('\\') => result.push('\\'),
                Some(v) => result.push(v),
                None => break,
            }
        } else {
            result.push(c);
        }
    }

    result
}
