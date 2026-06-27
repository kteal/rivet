use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lexer::{Token, TokenKind, lex};
use crate::source::{SourceMap, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessError {
    Local(PreprocessError),
    File(PreprocessFileError),
}

impl From<PreprocessError> for ProcessError {
    fn from(value: PreprocessError) -> Self {
        Self::Local(value)
    }
}

impl From<PreprocessFileError> for ProcessError {
    fn from(value: PreprocessFileError) -> Self {
        Self::File(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreprocessFileError {
    Io {
        path: PathBuf,
        message: String,
    },
    Lex {
        path: PathBuf,
        source: String,
        span: Span,
        message: String,
    },
    Preprocess {
        path: PathBuf,
        source: String,
        span: Span,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedFile {
    pub tokens: Vec<Token>,
    pub source_map: SourceMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MacroDef {
    ObjectLike(Vec<Token>),
    FunctionLike {
        params: Vec<String>,
        replacement: Vec<Token>,
    },
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConditionalFrame {
    parent_active: bool,
    current_active: bool,
    branch_taken: bool,
    saw_else: bool,
}

struct TokenScanner {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenScanner {
    const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    const fn is_done(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_done() {
            None
        } else {
            Some(&self.tokens[self.pos])
        }
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_done() {
            None
        } else {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
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
                message: "reached EOF".to_string(),
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

enum IncludeKind {
    Quoted,
    Angle,
}

struct IncludeName {
    kind: IncludeKind,
    name: String,
    span: Span,
}

struct InputScanner {
    tokens: Vec<Token>,
    pos: usize,
    path: Option<PathBuf>,
}

impl InputScanner {
    fn new(tokens: Vec<Token>, path: Option<&Path>) -> Self {
        Self {
            tokens,
            pos: 0,
            path: path.map(Path::to_path_buf),
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

    fn expect_include_name(&mut self) -> Result<IncludeName, PreprocessError> {
        let token = self.advance();
        match token {
            Token {
                kind: TokenKind::StringLiteral(bytes),
                span,
            } => {
                let name = String::from_utf8(bytes).map_err(|_| PreprocessError {
                    message: "failed to convert bytes to string".to_string(),
                    span,
                })?;
                return Ok(IncludeName {
                    kind: IncludeKind::Quoted,
                    name,
                    span,
                });
            }
            Token {
                kind: TokenKind::Less,
                span,
            } => {
                let mut include_path = String::new();
                loop {
                    match self.peek() {
                        Token {
                            kind: TokenKind::Ident(name),
                            ..
                        } => {
                            include_path.push_str(name);
                            self.advance();
                        }
                        Token {
                            kind: TokenKind::Dot,
                            ..
                        } => {
                            include_path.push('.');
                            self.advance();
                        }
                        Token {
                            kind: TokenKind::IntLiteral { value, .. },
                            ..
                        } => {
                            include_path.push_str(&value.to_string());
                            self.advance();
                        }
                        Token {
                            kind: TokenKind::Slash,
                            ..
                        } => {
                            include_path.push('/');
                            self.advance();
                        }
                        Token {
                            kind: TokenKind::Greater,
                            ..
                        } => {
                            self.advance();
                            break;
                        }
                        token => {
                            return Err(PreprocessError {
                                message: format!("unexpected '{:?}' in include path", token.kind),
                                span: token.span,
                            });
                        }
                    }
                }
                if include_path.is_empty() {
                    return Err(PreprocessError {
                        message: "empty angle include path".to_string(),
                        span,
                    });
                }
                Ok(IncludeName {
                    kind: IncludeKind::Angle,
                    name: include_path,
                    span,
                })
            }
            token => {
                return Err(PreprocessError {
                    message: "expected include path".to_string(),
                    span: token.span,
                });
            }
        }
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

    fn skip_until_newline(&mut self) {
        while !matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof) {
            self.advance();
        }
        // We should only consume a newline, not eof
        self.advance_if(&TokenKind::Newline);
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

    fn resolve_quoted_include_path(
        &self,
        file_name: &str,
        span: Span,
    ) -> Result<PathBuf, PreprocessError> {
        self.path.as_ref().map_or_else(
            || {
                Err(PreprocessError {
                    message: "cannot resolve quoted include without source file path".to_string(),
                    span,
                })
            },
            |path| {
                Ok(path
                    .parent()
                    .expect("file not in a directory")
                    .join(file_name))
            },
        )
    }
}

struct Preprocessor {
    macros: HashMap<String, MacroDef>,
    conditionals: Vec<ConditionalFrame>,
    source_map: SourceMap,
    include_dirs: Vec<PathBuf>,
}

impl Preprocessor {
    fn new() -> Self {
        Self {
            macros: HashMap::new(),
            conditionals: vec![],
            source_map: SourceMap::new(),
            include_dirs: vec![PathBuf::from("tests/programs/include")],
        }
    }

    fn is_active(&self) -> bool {
        self.conditionals
            .last()
            .is_none_or(|frame| frame.current_active)
    }

    fn resolve_angle_include_path(
        &self,
        file_name: &str,
        span: Span,
    ) -> Result<PathBuf, PreprocessError> {
        for dir in &self.include_dirs {
            let candidate = dir.join(file_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(PreprocessError {
            message: format!("cannot resolve angle include '<{file_name}>'"),
            span,
        })
    }

    fn parse_macro_params(scanner: &mut InputScanner) -> Result<Vec<String>, PreprocessError> {
        scanner.expect(&TokenKind::LParen)?;
        let mut params = vec![];

        if scanner.peek_kind() == &TokenKind::RParen {
            scanner.advance();
            return Ok(params);
        }

        loop {
            let (param, _) = scanner.expect_ident()?;
            params.push(param);

            if scanner.peek_kind() == &TokenKind::Comma {
                scanner.advance();
                if scanner.peek_kind() == &TokenKind::RParen {
                    let token = scanner.peek();
                    return Err(PreprocessError {
                        message: "trailing comma".to_string(),
                        span: token.span,
                    });
                }
                continue;
            }

            scanner.expect(&TokenKind::RParen)?;
            return Ok(params);
        }
    }

    fn parse_define(&mut self, scanner: &mut InputScanner) -> Result<(), PreprocessError> {
        if !self.is_active() {
            scanner.skip_until_newline();
            return Ok(());
        }

        scanner.expect(&TokenKind::Hash)?;
        scanner.expect_ident()?; // "define"
        let (macro_name, _) = scanner.expect_ident()?;

        // FunctionLike macro definition
        if scanner.peek_kind() == &TokenKind::LParen {
            let params = Self::parse_macro_params(scanner)?;
            let replacement = scanner.collect_until_newline();
            self.macros.insert(
                macro_name,
                MacroDef::FunctionLike {
                    params,
                    replacement,
                },
            );
        } else {
            let replacement = scanner.collect_until_newline();
            self.macros
                .insert(macro_name, MacroDef::ObjectLike(replacement));
        }
        Ok(())
    }

    fn parse_ifdef(
        &mut self,
        scanner: &mut InputScanner,
        inverted: bool,
    ) -> Result<(), PreprocessError> {
        scanner.expect(&TokenKind::Hash)?;
        scanner.expect_ident()?; // "ifdef" or "ifndef"
        let (macro_name, _) = scanner.expect_ident()?;
        scanner.skip_until_newline();

        let condition_met = self.macros.contains_key(&macro_name) ^ inverted;
        let parent_active = self.is_active();
        self.conditionals.push(ConditionalFrame {
            parent_active,
            current_active: parent_active && condition_met,
            branch_taken: condition_met,
            saw_else: false,
        });
        Ok(())
    }

    fn parse_else(&mut self, scanner: &mut InputScanner) -> Result<(), PreprocessError> {
        scanner.expect(&TokenKind::Hash)?;
        let else_token = scanner.expect(&TokenKind::KwElse)?; // "else"
        if self.conditionals.is_empty() {
            return Err(PreprocessError {
                message: "cannot use #else without opening conditional macro".to_string(),
                span: else_token.span,
            });
        }
        if let Some(frame) = self.conditionals.last()
            && frame.saw_else
        {
            return Err(PreprocessError {
                message: "cannot use duplicate #else".to_string(),
                span: else_token.span,
            });
        }
        if let Some(frame) = self.conditionals.last_mut() {
            frame.current_active = frame.parent_active && !frame.branch_taken;
            frame.branch_taken = true;
            frame.saw_else = true;
        }
        scanner.skip_until_newline();
        Ok(())
    }

    fn parse_endif(&mut self, scanner: &mut InputScanner) -> Result<(), PreprocessError> {
        scanner.expect(&TokenKind::Hash)?;
        let (_, span) = scanner.expect_ident()?; // "endif"
        if self.conditionals.is_empty() {
            return Err(PreprocessError {
                message: "cannot use #endif without opening conditional macro".to_string(),
                span,
            });
        }
        self.conditionals.pop();
        scanner.skip_until_newline();
        Ok(())
    }

    fn parse_include(&mut self, scanner: &mut InputScanner) -> Result<Vec<Token>, ProcessError> {
        if !self.is_active() {
            scanner.skip_until_newline();
            return Ok(vec![]);
        }

        scanner.expect(&TokenKind::Hash)?;
        scanner.expect_ident()?; // "include"
        let include = scanner.expect_include_name()?;
        scanner.skip_until_newline();

        let resolved_path = match include.kind {
            IncludeKind::Quoted => {
                scanner.resolve_quoted_include_path(&include.name, include.span)?
            }
            IncludeKind::Angle => self.resolve_angle_include_path(&include.name, include.span)?,
        };
        let mut included_tokens = self.process_file(&resolved_path)?;
        if included_tokens.last().expect("got no included tokens").kind == TokenKind::Eof {
            included_tokens.pop();
        }
        Ok(included_tokens)
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
                    active_macros.insert(name.clone());
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

    fn process_scanner(&mut self, scanner: &mut InputScanner) -> Result<Vec<Token>, ProcessError> {
        let mut output = vec![];
        let conditional_depth = self.conditionals.len();
        while scanner.peek_kind() != &TokenKind::Eof {
            match scanner.peek_kind() {
                TokenKind::Hash => {
                    let token = scanner.peek_nth(1).clone();

                    match token.kind {
                        TokenKind::Ident(name) => match name.as_str() {
                            "define" => self.parse_define(scanner)?,
                            "ifdef" => self.parse_ifdef(scanner, false)?,
                            "ifndef" => self.parse_ifdef(scanner, true)?,
                            "endif" => self.parse_endif(scanner)?,
                            "include" => {
                                let included = self.parse_include(scanner)?;
                                output.extend(included);
                            }
                            _ => {
                                if self.is_active() {
                                    return Err(ProcessError::Local(PreprocessError {
                                        message: format!(
                                            "unsupported preprocessor directive '{name}'"
                                        ),
                                        span: token.span,
                                    }));
                                }
                                scanner.skip_until_newline();
                            }
                        },
                        TokenKind::KwElse => self.parse_else(scanner)?,
                        kind => {
                            return Err(ProcessError::Local(PreprocessError {
                                message: format!(
                                    "expected preprocessor directive after '#', got '{kind:?}'"
                                ),
                                span: token.span,
                            }));
                        }
                    }
                }
                TokenKind::Newline => {
                    scanner.advance();
                }
                // macros used after definition
                _ => {
                    let chunk = scanner.collect_normal_tokens();
                    if self.is_active() {
                        let expanded = self.expand_tokens(chunk, &mut HashSet::new())?;
                        output.extend(expanded.iter().cloned());
                    }
                }
            }
        }

        let eof = scanner.expect(&TokenKind::Eof)?;
        if self.conditionals.len() != conditional_depth {
            return Err(ProcessError::Local(PreprocessError {
                message: "unterminated conditional directive".to_string(),
                span: eof.span,
            }));
        }
        output.push(eof);
        Ok(output)
    }

    fn process_file(&mut self, path: &Path) -> Result<Vec<Token>, PreprocessFileError> {
        let source = fs::read_to_string(path).map_err(|err| PreprocessFileError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        let source = splice_escaped_newlines(&source);
        let file_id = self.source_map.add_file(path, source.clone());
        let tokens = lex(&source, file_id).map_err(|err| PreprocessFileError::Lex {
            path: path.to_path_buf(),
            source: source.clone(),
            span: err.span,
            message: err.message,
        })?;
        let mut scanner = InputScanner::new(tokens, Some(path));
        match self.process_scanner(&mut scanner) {
            Ok(tokens) => Ok(tokens),
            Err(ProcessError::Local(err)) => Err(PreprocessFileError::Preprocess {
                path: path.to_path_buf(),
                source,
                span: err.span,
                message: err.message,
            }),
            Err(ProcessError::File(err)) => Err(err),
        }
    }
}

/// Expands the supported preprocessing directives from a token stream.
///
/// # Errors
///
/// Returns a [`PreprocessError`] when a supported directive is malformed, such as
/// a `#define` without an identifier macro name.
pub fn preprocess(tokens: Vec<Token>) -> Result<Vec<Token>, PreprocessError> {
    let mut preprocessor = Preprocessor::new();
    let mut scanner = InputScanner::new(tokens, None);
    match preprocessor.process_scanner(&mut scanner) {
        Ok(tokens) => Ok(tokens),
        Err(ProcessError::Local(err)) => Err(err),
        Err(ProcessError::File(_)) => unreachable!(),
    }
}

/// Expands the supported preprocessing directives from a source file.
///
/// # Errors
///
/// Returns a [`PreprocessFileError`] when the source file cannot be read, when
/// lexing fails, or when preprocessing fails.
pub fn preprocess_file(path: &Path) -> Result<PreprocessedFile, PreprocessFileError> {
    let mut preprocessor = Preprocessor::new();
    let tokens = preprocessor.process_file(path)?;
    Ok(PreprocessedFile {
        tokens,
        source_map: preprocessor.source_map,
    })
}

#[must_use]
pub fn splice_escaped_newlines(source: &str) -> String {
    source.replace("\\\r\n", "").replace("\\\n", "")
}
