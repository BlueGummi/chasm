use crate::tokens::TokenKind;
use logos::Logos;
#[derive(Debug)]

pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

pub struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    pub fn new(input: &str) -> Self {
        let lex = TokenKind::lexer(input);

        let tokens = lex
            .spanned()
            .filter_map(|(tok, span)| match tok {
                Ok(kind) => Some(Token {
                    kind,
                    text: input[span.clone()].to_string(),
                }),
                Err(_) => None,
            })
            .collect();

        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    pub fn expect(&mut self, expected: TokenKind) {
        let next = self.next().expect("Unexpected EOF");
        if next.kind != expected {
            panic!("Expected {:?} but found {:?}", expected, next.kind);
        }
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

#[derive(Debug)]
pub enum Statement {
    VarAssign {
        name: String,
        expr: i64,
    },
    ConstAssign {
        name: String,
        expr: i64,
    },
    Label(String),
    Instruction {
        name: String,
        args: Vec<String>,
    },

    Directive {
        name: String,
        args: Vec<String>,
    },
    Include(String),

    MacroDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },

    ForLoop {
        var: String,
        start: i64,
        end: i64,
        body: Vec<Statement>,
    },

    Block(Vec<Statement>),
}

pub struct Parser {
    stream: TokenStream,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            stream: TokenStream::new(input),
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut stmts = vec![];

        while !self.stream.eof() {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.stream.next(); // skip unknown
            }
        }

        stmts
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let tok = self.stream.peek()?.kind.clone();

        match tok {
            TokenKind::Var => return self.parse_var(),
            TokenKind::Const => return self.parse_const(),

            TokenKind::Ident(_) if self.lookahead_is_label() => {
                return self.parse_label();
            }

            TokenKind::AtDirective => return self.parse_directive(),

            TokenKind::Include => return self.parse_include(),

            TokenKind::MacroRules => return self.parse_macro(),

            TokenKind::ForBang => return self.parse_for_loop(),

            TokenKind::LeftBrace => return self.parse_block(),

            TokenKind::Ident(_) => return self.parse_instruction(),

            _ => {
                self.stream.next();
                None
            }
        }
    }

    fn lookahead_is_label(&self) -> bool {
        let a = self.stream.tokens.get(self.stream.pos);
        let b = self.stream.tokens.get(self.stream.pos + 1);

        match (a.map(|t| &t.kind), b.map(|t| &t.kind)) {
            (Some(TokenKind::Ident(_)), Some(TokenKind::Colon)) => true,
            (Some(TokenKind::Dot), Some(TokenKind::Ident(_))) => {
                matches!(
                    self.stream.tokens.get(self.stream.pos + 2).map(|t| &t.kind),
                    Some(TokenKind::Colon)
                )
            }
            (Some(TokenKind::Colon), Some(TokenKind::Colon)) => {
                matches!(
                    self.stream.tokens.get(self.stream.pos + 2).map(|t| &t.kind),
                    Some(TokenKind::Ident(_))
                ) && matches!(
                    self.stream.tokens.get(self.stream.pos + 3).map(|t| &t.kind),
                    Some(TokenKind::Colon)
                )
            }
            _ => false,
        }
    }

    fn parse_label(&mut self) -> Option<Statement> {
        if let TokenKind::Ident(name) = self.stream.next()?.kind.clone() {
            self.stream.expect(TokenKind::Colon);
            Some(Statement::Label(name))
        } else {
            None
        }
    }
    fn parse_instruction(&mut self) -> Option<Statement> {
        // eat the name
        let name = match self.stream.next()?.kind.clone() {
            TokenKind::Ident(n) => n,
            _ => return None,
        };

        // parse zero or more arguments until newline or symbol

        let mut args = Vec::new();

        while let Some(tok) = self.stream.peek() {
            match tok.kind {
                TokenKind::Ident(ref s) => {
                    args.push(s.clone());
                    self.stream.next();
                }
                TokenKind::IntLit(n) => {
                    args.push(n.to_string());

                    self.stream.next();
                }
                TokenKind::StrLit(ref s) => {
                    args.push(s.clone());
                    self.stream.next();
                }
                TokenKind::CharLit(c) => {
                    args.push(format!("{}", c));
                    self.stream.next();
                }
                _ => break,
            }
        }

        Some(Statement::Instruction { name, args })
    }

    fn parse_directive(&mut self) -> Option<Statement> {
        // read @something
        let at_tok = self.stream.next()?.text.clone();

        let name = at_tok.trim_start_matches('@').to_string();

        // now parse args
        let mut args = vec![];

        while let Some(tok) = self.stream.peek() {
            match tok.kind {
                TokenKind::Ident(ref s) | TokenKind::StrLit(ref s) => {
                    args.push(s.clone());
                    self.stream.next();
                }
                TokenKind::IntLit(n) => {
                    args.push(n.to_string());
                    self.stream.next();
                }
                _ => break,
            }
        }

        Some(Statement::Directive { name, args })
    }
    fn parse_include(&mut self) -> Option<Statement> {
        self.stream.expect(TokenKind::Include);

        let file = match self.stream.next()?.kind.clone() {
            TokenKind::StrLit(s) => s,
            t => panic!("Expected string literal after include, got {:?}", t),
        };

        Some(Statement::Include(file))
    }
    fn parse_macro(&mut self) -> Option<Statement> {
        self.stream.expect(TokenKind::MacroRules);

        let name = match self.stream.next()?.kind.clone() {
            TokenKind::Ident(n) => n,

            t => panic!("Expected macro name, got {:?}", t),
        };

        // parse param list: (a, b, c)
        self.stream.expect(TokenKind::LeftParen);

        let mut params = Vec::new();

        loop {
            match self.stream.next()?.kind.clone() {
                TokenKind::Ident(p) => params.push(p),
                TokenKind::RightParen => break,
                TokenKind::Comma => continue,
                t => panic!("Unexpected token in macro param list: {:?}", t),
            }
        }

        // body is a block
        let body = match self.parse_block()? {
            Statement::Block(stmts) => stmts,
            _ => panic!("Expected a block in macro definition"),
        };

        Some(Statement::MacroDef { name, params, body })
    }
    fn parse_for_loop(&mut self) -> Option<Statement> {
        self.stream.expect(TokenKind::ForBang);

        self.stream.expect(TokenKind::LeftParen);

        // initializer: var i = 0
        self.stream.expect(TokenKind::Var);
        let var = match self.stream.next()?.kind.clone() {
            TokenKind::Ident(n) => n,

            _ => panic!("expected loop variable name"),
        };
        self.stream.expect(TokenKind::Equal);
        let start = match self.stream.next()?.kind.clone() {
            TokenKind::IntLit(v) => v,
            _ => panic!("expected integer literal in for loop start"),
        };

        self.stream.expect(TokenKind::Semicolon);

        // condition: i < limit
        self.stream.expect(TokenKind::Ident(var.clone()));
        self.stream.expect(TokenKind::Less);
        let end = match self.stream.next()?.kind.clone() {
            TokenKind::IntLit(v) => v,
            _ => panic!("expected integer literal in for loop end"),
        };

        self.stream.expect(TokenKind::Semicolon);

        // increment: i++
        self.stream.expect(TokenKind::Ident(var.clone()));
        self.stream.expect(TokenKind::PlusPlus);

        self.stream.expect(TokenKind::RightParen);

        // parse body block {...}
        let body = match self.parse_block()? {
            Statement::Block(stmts) => stmts,
            _ => panic!("Expected a block in for loop"),
        };

        Some(Statement::ForLoop {
            var,
            start,
            end,
            body,
        })
    }
    fn parse_block(&mut self) -> Option<Statement> {
        self.stream.expect(TokenKind::LeftBrace);

        let mut body = Vec::new();

        while let Some(tok) = self.stream.peek() {
            if let TokenKind::RightBrace = tok.kind {
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                self.stream.next();
            }
        }

        self.stream.expect(TokenKind::RightBrace);

        Some(Statement::Block(body))
    }

    fn parse_var(&mut self) -> Option<Statement> {
        self.stream.next(); // eat 'var'

        let name = match self.stream.next()?.kind.clone() {
            TokenKind::Ident(n) => n,
            t => panic!("expected identifier, got {:?}", t),
        };

        self.stream.expect(TokenKind::Equal);

        let expr = match self.stream.next()?.kind.clone() {
            TokenKind::IntLit(v) => v,
            t => panic!("expected integer literal, got {:?}", t),
        };

        Some(Statement::VarAssign { name, expr })
    }

    fn parse_const(&mut self) -> Option<Statement> {
        self.stream.next(); // eat 'const'

        let name = match self.stream.next()?.kind.clone() {
            TokenKind::Ident(n) => n,
            t => panic!("expected identifier, got {:?}", t),
        };

        self.stream.expect(TokenKind::Equal);

        let expr = match self.stream.next()?.kind.clone() {
            TokenKind::IntLit(v) => v,
            t => panic!("expected integer literal, got {:?}", t),
        };

        Some(Statement::ConstAssign { name, expr })
    }
}
