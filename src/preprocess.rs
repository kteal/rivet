use std::collections::{HashMap, HashSet};

use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MacroDef {
    ObjectLike(Vec<Token>),
    FunctionLike {
        params: Vec<String>,
        replacement: Vec<Token>,
    },
}

struct TokenScanner {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenScanner {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn is_done(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        if !self.is_done() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        if let Some(token) = self.peek() {
            Some(&token.kind)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_done() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &TokenKind, name_span: Span) -> Result<Token, PreprocessError> {
        let token = self.advance();

        if let Some(token) = &token
            && &token.kind == expected
        {
            Ok(token.clone())
        } else if let Some(token) = token {
            Err(PreprocessError {
                message: format!("expected {expected:?}, found {:?}", token.kind),
                span: token.span,
            })
        } else {
            Err(PreprocessError {
                message: format!("reached EOF"),
                span: name_span,
            })
        }
    }

    fn parse_macro_args(&mut self, name_span: Span) -> Result<Vec<Vec<Token>>, PreprocessError> {
        let open_token = self.expect(&TokenKind::LParen, name_span)?;
        let mut args = vec![];
        let mut current_arg = vec![];
        let mut paren_depth = 0;

        if self.peek_kind() == Some(&TokenKind::RParen) {
            self.advance();
            return Ok(args);
        }

        loop {
            let Some(token) = self.advance() else {
                return Err(PreprocessError {
                    message: "reached unterminated '(' in macro invocation".to_string(),
                    span: name_span,
                });
            };

            if token.kind == TokenKind::LParen {
                paren_depth += 1;
                current_arg.push(token);
            } else if token.kind == TokenKind::RParen {
                if paren_depth == 0 {
                    args.push(current_arg);
                    break;
                }
                paren_depth -= 1;
                current_arg.push(token);
            } else if token.kind == TokenKind::Comma && paren_depth == 0 {
                args.push(current_arg);
                current_arg = vec![];
            } else if token.kind == TokenKind::Eof {
                return Err(PreprocessError {
                    message: "unterminated '(' (reached EOF), needs ')'".to_string(),
                    span: open_token.span,
                });
            } else {
                current_arg.push(token);
            }
        }
        Ok(args)
    }
}

struct Preprocessor {
    tokens: Vec<Token>,
    pos: usize,
    macros: HashMap<String, MacroDef>,
}

impl Preprocessor {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            macros: HashMap::new(),
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

    fn parse_macro_params(&mut self) -> Result<Vec<String>, PreprocessError> {
        self.expect(&TokenKind::LParen)?;
        let mut params = vec![];

        if self.peek_kind() == &TokenKind::RParen {
            self.advance();
            return Ok(params);
        }

        loop {
            let (param, _) = self.expect_ident()?;
            params.push(param);

            if self.peek_kind() == &TokenKind::Comma {
                self.advance();
                if self.peek_kind() == &TokenKind::RParen {
                    let token = self.peek();
                    return Err(PreprocessError {
                        message: "trailing comma".to_string(),
                        span: token.span,
                    });
                }
                continue;
            }

            self.expect(&TokenKind::RParen)?;
            return Ok(params);
        }
    }

    fn parse_define(&mut self) -> Result<(), PreprocessError> {
        self.expect(&TokenKind::Hash)?;
        self.expect_ident()?; // "define"
        let (macro_name, _) = self.expect_ident()?;

        // FunctionLike macro definition
        if self.peek_kind() == &TokenKind::LParen {
            let params = self.parse_macro_params()?;
            let replacement = self.collect_until_newline();
            self.macros.insert(
                macro_name,
                MacroDef::FunctionLike {
                    params,
                    replacement,
                },
            );
        } else {
            let replacement = self.collect_until_newline();
            self.macros
                .insert(macro_name, MacroDef::ObjectLike(replacement));
        }
        Ok(())
    }

    fn collect_until_newline(&mut self) -> Vec<Token> {
        let mut replacement = vec![];
        while !matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof) {
            replacement.push(self.advance());
        }
        // We should only consume a newline, not eof
        self.advance_if(&TokenKind::Newline);
        replacement
    }

    fn collect_normal_tokens(&mut self) -> Vec<Token> {
        let mut output = vec![];
        while !matches!(
            self.peek_kind(),
            TokenKind::Newline | TokenKind::Hash | TokenKind::Eof
        ) {
            output.push(self.advance());
        }
        output
    }

    fn expand_function_like(
        params: &[String],
        replacement: &[Token],
        args: &[Vec<Token>],
        call_span: Span,
    ) -> Result<Vec<Token>, PreprocessError> {
        if params.len() != args.len() {
            return Err(PreprocessError {
                message: format!(
                    "macro was defined with '{}' parameters, cannot be called with '{}' arguments",
                    params.len(),
                    args.len()
                ),
                span: call_span,
            });
        }
        let map: HashMap<&String, &Vec<Token>> = params.iter().zip(args.iter()).collect();

        let mut output = vec![];
        for token in replacement {
            if let TokenKind::Ident(name) = &token.kind
                && let Some(arg) = map.get(&name)
            {
                output.extend(arg.iter().cloned());
            } else {
                output.push(token.clone());
            }
        }
        Ok(output)
    }

    fn expand_macro_use(
        &self,
        scanner: &mut TokenScanner,
        name: &str,
        name_token: Token,
    ) -> Result<Vec<Token>, PreprocessError> {
        match self.macros.get(name).unwrap().clone() {
            MacroDef::ObjectLike(replacement) => Ok(replacement),
            MacroDef::FunctionLike {
                params,
                replacement,
            } => {
                if scanner.peek_kind() != Some(&TokenKind::LParen) {
                    return Ok(vec![name_token]);
                }

                let args = scanner.parse_macro_args(name_token.span)?;
                let expanded_replacement =
                    Self::expand_function_like(&params, &replacement, &args, name_token.span)?;
                Ok(expanded_replacement)
            }
        }
    }

    fn expand_tokens(
        &mut self,
        tokens: Vec<Token>,
        active_macros: &mut HashSet<String>,
    ) -> Result<Vec<Token>, PreprocessError> {
        let mut scanner = TokenScanner::new(tokens);
        let mut output = vec![];

        while !scanner.is_done() {
            let token = scanner.advance().expect("should not have reached EOF");

            if let TokenKind::Ident(name) = &token.kind
                && self.macros.contains_key(name)
            {
                if active_macros.contains(name) {
                    output.push(token);
                } else {
                    active_macros.insert(name.to_string());
                    let replacement = self.expand_macro_use(&mut scanner, name, token.clone())?;
                    let rescanned = self.expand_tokens(replacement, active_macros)?;
                    active_macros.remove(name);
                    output.extend(rescanned.iter().cloned());
                }
            } else {
                output.push(token);
            }
        }

        Ok(output)
    }

    fn preprocess(&mut self) -> Result<Vec<Token>, PreprocessError> {
        let mut output = vec![];
        while self.peek_kind() != &TokenKind::Eof {
            match self.peek_kind() {
                // #define
                TokenKind::Hash => match self.peek_nth(1) {
                    Token {
                        kind: TokenKind::Ident(name),
                        span,
                    } if name == "define" => self.parse_define()?,
                    Token {
                        kind: TokenKind::Ident(name),
                        span,
                    } => {
                        return Err(PreprocessError {
                            message: format!("unsupported preprocessor directive '{name}'"),
                            span: *span,
                        });
                    }
                    token => {
                        return Err(PreprocessError {
                            message: format!(
                                "expected preprocessor directive after '#', got '{:?}'",
                                token.kind
                            ),
                            span: token.span,
                        });
                    }
                },
                TokenKind::Newline => {
                    self.advance();
                }
                // macros used after definition
                _ => {
                    let chunk = self.collect_normal_tokens();
                    let expanded = self.expand_tokens(chunk, &mut HashSet::new())?;
                    output.extend(expanded.iter().cloned());
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
