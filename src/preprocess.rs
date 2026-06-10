use std::collections::HashMap;

use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessError {
    pub message: String,
    pub span: Span,
}

struct Preprocessor {
    tokens: Vec<Token>,
    pos: usize,
    definitions: HashMap<String, Vec<Token>>,
}

impl Preprocessor {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            definitions: HashMap::new(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_nth(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("parser token stream should end with EOF")
        })
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn peek_nth_kind(&self, n: usize) -> &TokenKind {
        &self.peek_nth(n).kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn advance_if(&mut self, token: &TokenKind) -> Option<Token> {
        if self.peek_kind() == token {
            let out_token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(out_token)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, PreprocessError> {
        let token = self.advance();

        if &token.kind == expected {
            Ok(token)
        } else {
            Err(PreprocessError {
                message: format!("expected {expected:?}, found {:?}", token.kind),
                span: token.span,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), PreprocessError> {
        let token = self.advance();

        match token {
            Token {
                kind: TokenKind::Ident(name),
                span,
            } => Ok((name, span)),

            token => Err(PreprocessError {
                message: format!("expected identifier token, got '{:?}'", token.kind),
                span: token.span,
            }),
        }
    }

    fn preprocess(&mut self) -> Result<Vec<Token>, PreprocessError> {
        let mut output = vec![];
        while self.peek_kind() != &TokenKind::Eof {
            match self.peek_kind() {
                // #define
                TokenKind::Hash
                    if self.peek_nth_kind(1) == &TokenKind::Ident("define".to_string()) =>
                {
                    self.expect(&TokenKind::Hash)?;
                    self.expect_ident()?;

                    let (macro_name, _) = self.expect_ident()?;
                    let mut replacement = vec![];
                    while self.peek_kind() != &TokenKind::Newline
                        && self.peek_kind() != &TokenKind::Eof
                    {
                        replacement.push(self.advance());
                    }
                    // We should only consume a newline, not eof
                    self.advance_if(&TokenKind::Newline);

                    self.definitions.insert(macro_name, replacement);
                }
                // macros used after definition
                TokenKind::Ident(name) if self.definitions.contains_key(name) => {
                    let name = name.clone();
                    self.advance();

                    output.extend(
                        self.definitions
                            .get(&name)
                            .expect("cannot use undefined preprocessor directive")
                            .iter()
                            .cloned(),
                    );
                }
                TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    let token = self.advance();
                    output.push(token.clone());
                }
            }
        }

        let eof = self.expect(&TokenKind::Eof)?;
        output.push(eof);
        Ok(output)
    }
}

/// Expands the supported preprocessing directives from a token stream.
///
/// # Errors
///
/// Returns a [`PreprocessError`] when a supported directive is malformed, such as
/// a `#define` without an identifier macro name.
pub fn preprocess(tokens: Vec<Token>) -> Result<Vec<Token>, PreprocessError> {
    let mut preprocessor = Preprocessor::new(tokens);
    preprocessor.preprocess()
}
