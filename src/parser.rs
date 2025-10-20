use vec1::{vec1, Vec1};

use crate::ast::*;
use crate::err::{ParsingError, ParsingErrorLevel};
use crate::filesystem::FileIndex;
use crate::interner::*;
use crate::ir_common::*;
use crate::tokenizer::*;
use crate::*;

#[derive(Debug)]
pub struct Parser<'src> {
    file: FileIndex,
    text: &'src str,
    tokenizer: Tokenizer<'src>,
    errs: Vec<ParsingError>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug)]
pub struct ParserResult {
    pub file: FileIndex,
    pub ast: TopLevel,
    pub errs: Vec<ParsingError>,
}

fn get_prefix_op(t: &Option<Token>) -> Option<PrefixOp> {
    if t.is_none() {
        return None;
    }
    let t = t.as_ref().unwrap();

    match t {
        Token {
            data: TokenData::Punctuation(p),
            ..
        } => match p {
            Punctuation::Plus => Some(PrefixOp::Plus),
            Punctuation::Minus => Some(PrefixOp::Minus),
            Punctuation::Increment => Some(PrefixOp::Increment),
            Punctuation::Decrement => Some(PrefixOp::Decrement),
            Punctuation::LogicalNot => Some(PrefixOp::LogicalNot),
            Punctuation::BitwiseNot => Some(PrefixOp::BitwiseNot),
            _ => None,
        },
        Token {
            data: TokenData::Keyword(k),
            ..
        } => match k {
            Keyword::SizeOf => Some(PrefixOp::SizeOf),
            Keyword::AlignOf => Some(PrefixOp::AlignOf),
            _ => None,
        },
        _ => None,
    }
}

fn get_postfix_op(t: &Option<Token>) -> Option<PostfixOp> {
    if t.is_none() {
        return None;
    }
    let t = t.as_ref().unwrap();

    if let Token {
        data: TokenData::Punctuation(p),
        ..
    } = t
    {
        match p {
            Punctuation::Increment => Some(PostfixOp::Increment),
            Punctuation::Decrement => Some(PostfixOp::Decrement),
            _ => None,
        }
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum InfixOp {
    Binary(BinaryOp),
    LeftRound,
    LeftSquare,
    Ternary,
}

fn get_infix_op(t: &Option<Token>) -> Option<InfixOp> {
    if t.is_none() {
        return None;
    }
    let t = t.as_ref().unwrap();

    match t {
        Token {
            data: TokenData::Punctuation(p),
            ..
        } => match p {
            Punctuation::Plus => Some(InfixOp::Binary(BinaryOp::Add)),
            Punctuation::Minus => Some(InfixOp::Binary(BinaryOp::Subtract)),
            Punctuation::Times => Some(InfixOp::Binary(BinaryOp::Times)),
            Punctuation::Divide => Some(InfixOp::Binary(BinaryOp::Divide)),
            Punctuation::Modulo => Some(InfixOp::Binary(BinaryOp::Modulo)),
            Punctuation::Raise => Some(InfixOp::Binary(BinaryOp::Raise)),

            Punctuation::LeftShift => Some(InfixOp::Binary(BinaryOp::LeftShift)),
            Punctuation::RightShift => Some(InfixOp::Binary(BinaryOp::RightShift)),
            Punctuation::UnsignedRightShift => Some(InfixOp::Binary(BinaryOp::UnsignedRightShift)),

            Punctuation::DotDot => Some(InfixOp::Binary(BinaryOp::Concat)),

            Punctuation::LeftAngle => Some(InfixOp::Binary(BinaryOp::LessThan)),
            Punctuation::LessThanEquals => Some(InfixOp::Binary(BinaryOp::LessThanEquals)),
            Punctuation::RightAngle => Some(InfixOp::Binary(BinaryOp::GreaterThan)),
            Punctuation::GreaterThanEquals => Some(InfixOp::Binary(BinaryOp::GreaterThanEquals)),
            Punctuation::Equals => Some(InfixOp::Binary(BinaryOp::Equals)),
            Punctuation::NotEquals => Some(InfixOp::Binary(BinaryOp::NotEquals)),
            Punctuation::ApproxEquals => Some(InfixOp::Binary(BinaryOp::ApproxEquals)),
            Punctuation::ThreeWayComp => Some(InfixOp::Binary(BinaryOp::ThreeWayComp)),

            Punctuation::LogicalAnd => Some(InfixOp::Binary(BinaryOp::LogicalAnd)),
            Punctuation::BitwiseAnd => Some(InfixOp::Binary(BinaryOp::BitwiseAnd)),
            Punctuation::LogicalOr => Some(InfixOp::Binary(BinaryOp::LogicalOr)),
            Punctuation::BitwiseOr => Some(InfixOp::Binary(BinaryOp::BitwiseOr)),
            Punctuation::BitwiseXor => Some(InfixOp::Binary(BinaryOp::BitwiseXor)),

            Punctuation::DoubleColon => Some(InfixOp::Binary(BinaryOp::Scope)),
            Punctuation::Dot => Some(InfixOp::Binary(BinaryOp::MemberAccess)),

            Punctuation::Assign => Some(InfixOp::Binary(BinaryOp::Assign)),
            Punctuation::PlusAssign => Some(InfixOp::Binary(BinaryOp::PlusAssign)),
            Punctuation::MinusAssign => Some(InfixOp::Binary(BinaryOp::MinusAssign)),
            Punctuation::TimesAssign => Some(InfixOp::Binary(BinaryOp::TimesAssign)),
            Punctuation::DivideAssign => Some(InfixOp::Binary(BinaryOp::DivideAssign)),
            Punctuation::ModuloAssign => Some(InfixOp::Binary(BinaryOp::ModuloAssign)),
            Punctuation::LeftShiftAssign => Some(InfixOp::Binary(BinaryOp::LeftShiftAssign)),
            Punctuation::RightShiftAssign => Some(InfixOp::Binary(BinaryOp::RightShiftAssign)),
            Punctuation::UnsignedRightShiftAssign => {
                Some(InfixOp::Binary(BinaryOp::UnsignedRightShiftAssign))
            }
            Punctuation::BitwiseOrAssign => Some(InfixOp::Binary(BinaryOp::BitwiseOrAssign)),
            Punctuation::BitwiseAndAssign => Some(InfixOp::Binary(BinaryOp::BitwiseAndAssign)),
            Punctuation::BitwiseXorAssign => Some(InfixOp::Binary(BinaryOp::BitwiseXorAssign)),

            Punctuation::QuestionMark => Some(InfixOp::Ternary),
            Punctuation::LeftSquare => Some(InfixOp::LeftSquare),
            Punctuation::LeftRound => Some(InfixOp::LeftRound),

            _ => None,
        },
        Token {
            data: TokenData::Keyword(k),
            ..
        } => match k {
            Keyword::Dot => Some(InfixOp::Binary(BinaryOp::DotProd)),
            Keyword::Cross => Some(InfixOp::Binary(BinaryOp::CrossProd)),
            Keyword::Is => Some(InfixOp::Binary(BinaryOp::Is)),
            _ => None,
        },
        _ => None,
    }
}

fn get_prefix_precedence(_: PrefixOp) -> ((), usize) {
    ((), 27)
}

fn get_postfix_precedence(_: PostfixOp) -> (usize, ()) {
    (27, ())
}

fn get_infix_precedence(b: InfixOp) -> (usize, usize) {
    use BinaryOp::*;
    match b {
        InfixOp::Ternary => (3, 2),

        InfixOp::LeftRound => (30, 31),
        InfixOp::LeftSquare => (30, 31),

        InfixOp::Binary(b) => match b {
            Assign => (1, 0),
            PlusAssign => (1, 0),
            MinusAssign => (1, 0),
            TimesAssign => (1, 0),
            DivideAssign => (1, 0),
            ModuloAssign => (1, 0),
            LeftShiftAssign => (1, 0),
            RightShiftAssign => (1, 0),
            UnsignedRightShiftAssign => (1, 0),
            BitwiseOrAssign => (1, 0),
            BitwiseAndAssign => (1, 0),
            BitwiseXorAssign => (1, 0),

            LogicalOr => (4, 5),

            LogicalAnd => (6, 7),

            Equals => (8, 9),
            NotEquals => (8, 9),
            ApproxEquals => (8, 9),

            LessThan => (10, 11),
            LessThanEquals => (10, 11),
            GreaterThan => (10, 11),
            GreaterThanEquals => (10, 11),
            ThreeWayComp => (10, 11),
            Is => (10, 11),

            Concat => (12, 13),

            BitwiseOr => (14, 15),

            BitwiseXor => (16, 17),

            BitwiseAnd => (18, 19),

            LeftShift => (20, 21),
            RightShift => (20, 21),
            UnsignedRightShift => (20, 21),

            Add => (22, 23),
            Subtract => (22, 23),

            Times => (24, 25),
            Divide => (24, 25),
            Modulo => (24, 25),
            CrossProd => (24, 25),
            DotProd => (24, 25),

            Raise => (26, 27),

            MemberAccess => (30, 31),

            Scope => (32, 33),
        },
    }
}

enum Extend {
    Class(ExtendClass),
    Struct(ExtendStruct),
}

impl<'src> Parser<'src> {
    pub fn new(file: FileIndex, text: &'src str) -> Self {
        Self {
            file,
            text,
            tokenizer: Tokenizer::new(file, text),
            errs: vec![],
        }
    }

    pub fn to_errs(self) -> Vec<ParsingError> {
        self.errs
    }

    fn err(&mut self, e: ParsingError) {
        self.errs.push(e);
    }

    fn get_keyword(&mut self, keywords: &[Keyword]) -> Option<(Keyword, Span)> {
        if let Some(Token {
            data: TokenData::Keyword(k),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            for key in keywords {
                if *key == *k {
                    let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
                    let span = t.span(self.file, self.text);
                    if let Token {
                        data: TokenData::Keyword(k),
                        ..
                    } = t
                    {
                        return Some((k, span));
                    } else {
                        unreachable!()
                    }
                }
            }
            None
        } else {
            None
        }
    }
    fn get_punc(&mut self, puncs: &[Punctuation]) -> Option<(Punctuation, Span)> {
        if let Some(Token {
            data: TokenData::Punctuation(p),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            for punc in puncs {
                if *punc == *p {
                    let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
                    let span = t.span(self.file, self.text);
                    if let Token {
                        data: TokenData::Punctuation(p),
                        ..
                    } = t
                    {
                        return Some((p, span));
                    } else {
                        unreachable!()
                    }
                }
            }
            None
        } else {
            None
        }
    }
    fn get_right_angle(&mut self) -> Option<(Punctuation, Span)> {
        let (punc, span) = self.get_punc(&[
            Punctuation::RightAngle,
            Punctuation::RightShift,
            Punctuation::RightShiftAssign,
            Punctuation::UnsignedRightShift,
            Punctuation::UnsignedRightShiftAssign,
            Punctuation::GreaterThanEquals,
        ])?;
        match punc {
            Punctuation::RightAngle => Some((punc, span)),
            Punctuation::RightShift
            | Punctuation::RightShiftAssign
            | Punctuation::UnsignedRightShift
            | Punctuation::UnsignedRightShiftAssign
            | Punctuation::GreaterThanEquals => {
                self.tokenizer.move_to_position(span.start + 1);
                Some((
                    Punctuation::RightAngle,
                    Span {
                        file: span.file,
                        start: span.start,
                        end: span.start + 1,
                    },
                ))
            }
            _ => unreachable!(),
        }
    }
    fn get_specific_ident(&mut self, idents: &[&str]) -> Option<Identifier> {
        if let Some(Token {
            data: TokenData::Identifier(i),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let i_sym = intern_name(i);
            for id in idents {
                let id_sym = intern_name(id);
                if id_sym == i_sym {
                    let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
                    return Some(Identifier {
                        span: t.span(self.file, self.text),
                        symbol: i_sym,
                    });
                }
            }
            None
        } else {
            None
        }
    }
    fn get_ident(&mut self) -> Option<Identifier> {
        if let Some(Token {
            data: TokenData::Identifier(s),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let sym = intern_name(s);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(Identifier {
                span: t.span(self.file, self.text),
                symbol: sym,
            })
        } else {
            None
        }
    }
    fn get_nws(&mut self) -> Option<NonWhitespace> {
        if let Some(Token {
            data: TokenData::NonWhitespace(s),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let sym = intern_name(s);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(NonWhitespace {
                span: t.span(self.file, self.text),
                symbol: sym,
            })
        } else {
            None
        }
    }
    fn get_string(&mut self) -> Option<StringConst> {
        if let Some(Token {
            data: TokenData::String(s),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let sym = intern_string(s);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(StringConst {
                span: t.span(self.file, self.text),
                symbol: sym,
            })
        } else {
            None
        }
    }
    fn get_string_concat(&mut self) -> Option<StringConst> {
        if let Some(Token {
            data: TokenData::String(s),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let mut sym = intern_string(s);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            let mut span = t.span(self.file, self.text);
            while let Some(Token {
                data: TokenData::String(s),
                ..
            }) = self.tokenizer.peek_no_doc(&mut self.errs)
            {
                let new = sym.string().to_string() + s;
                let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
                sym = intern_string(&new);
                span = span.combine(t.span(self.file, self.text));
            }
            Some(StringConst { span, symbol: sym })
        } else {
            None
        }
    }
    fn get_name(&mut self) -> Option<NameConst> {
        if let Some(Token {
            data: TokenData::Name(s),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let sym = intern_name(s);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(NameConst {
                span: t.span(self.file, self.text),
                symbol: sym,
            })
        } else {
            None
        }
    }
    fn get_int(&mut self) -> Option<IntConst> {
        if let Some(Token {
            data:
                TokenData::Int {
                    val,
                    long,
                    unsigned,
                },
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let (val, long, unsigned) = (*val, *long, *unsigned);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(IntConst {
                span: t.span(self.file, self.text),
                val,
                long,
                unsigned,
            })
        } else {
            None
        }
    }
    fn get_float(&mut self) -> Option<FloatConst> {
        if let Some(Token {
            data: TokenData::Float { val, double },
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let (val, double) = (*val, *double);
            let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();
            Some(FloatConst {
                span: t.span(self.file, self.text),
                val,
                double,
            })
        } else {
            None
        }
    }

    fn get_bool(&mut self) -> Option<(bool, Span)> {
        if let Some((x, s)) = self.get_keyword(&[Keyword::True, Keyword::False]) {
            Some((x == Keyword::True, s))
        } else {
            None
        }
    }

    fn get_const(&mut self) -> Option<Const> {
        if let Some(s) = self.get_string_concat() {
            Some(Const {
                span: s.span,
                kind: ConstKind::String(s),
            })
        } else if let Some(s) = self.get_name() {
            Some(Const {
                span: s.span,
                kind: ConstKind::Name(s),
            })
        } else if let Some(c) = self.get_int() {
            Some(Const {
                span: c.span,
                kind: ConstKind::Int(c),
            })
        } else if let Some(c) = self.get_float() {
            Some(Const {
                span: c.span,
                kind: ConstKind::Float(c),
            })
        } else if let Some((b, s)) = self.get_bool() {
            Some(Const {
                span: s,
                kind: ConstKind::Bool(b),
            })
        } else if let Some((_, s)) = self.get_keyword(&[Keyword::Null]) {
            Some(Const {
                span: s,
                kind: ConstKind::Null,
            })
        } else {
            None
        }
    }

    fn get_doc_comment(&mut self) -> Option<StringSymbol> {
        if let Some(Token {
            data: TokenData::DocComment(s),
            ..
        }) = self.tokenizer.peek_doc(&mut self.errs)
        {
            let mut sym = intern_string(s);
            let t = self.tokenizer.next_doc(&mut self.errs).unwrap();
            let mut span = t.span(self.file, self.text);
            while let Some(Token {
                data: TokenData::DocComment(s),
                ..
            }) = self.tokenizer.peek_doc(&mut self.errs)
            {
                let new = sym.string().to_string() + s;
                let t = self.tokenizer.next_doc(&mut self.errs).unwrap();
                sym = intern_string(&new);
                span = span.combine(t.span(self.file, self.text));
            }
            Some(sym)
        } else {
            None
        }
    }

    pub fn expect<T>(&mut self, val: Option<T>, msg: &str) -> Result<T, ParsingError> {
        if let Some(v) = val {
            Ok(v)
        } else {
            let next = self.tokenizer.peek_no_doc(&mut self.errs);
            let err_msg = match next {
                Some(n) => format!("expected {}, got {}", msg, n.data),
                None => format!("expected {}, got `EOF`", msg),
            };
            let span = match next {
                Some(n) => n.span(self.file, self.text),
                None => Span {
                    start: self.text.len(),
                    end: self.text.len(),
                    file: self.file,
                },
            };
            Err(ParsingError {
                level: ParsingErrorLevel::Error,
                msg: err_msg,
                main_spans: vec1![span],
                info_spans: vec![],
            })
        }
    }

    fn get_lump_version(&mut self) -> Result<Option<VersionInfo>, ParsingError> {
        if self.get_keyword(&[Keyword::Version][..]).is_some() {
            let ex = self.get_string();
            let s = self.expect(ex, "string literal")?;
            let ver_str = s.symbol.string();

            match parse_lump_version(&ver_str) {
                Some(v) => Ok(Some(v)),
                None => Err(ParsingError {
                    level: ParsingErrorLevel::Error,
                    msg: "invalid version directive".to_string(),
                    main_spans: vec1![s.span],
                    info_spans: vec![],
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn get_dottable_id(&mut self) -> Result<Option<DottableId>, ParsingError> {
        let id = match self.get_ident() {
            None => return Ok(None),
            Some(id) => id,
        };
        let mut ids = vec1![id];

        while self.get_punc(&[Punctuation::Dot]).is_some() {
            let ex = self.get_ident();
            let id = self.expect(ex, "identifier")?;
            ids.push(id);
        }

        let ret = DottableId {
            span: ids[0].span.combine(ids[ids.len() - 1].span),
            ids,
        };
        Ok(Some(ret))
    }

    fn get_primary_expr(&mut self) -> Result<Option<Expression>, ParsingError> {
        if let Some((_, s)) = self.get_keyword(&[Keyword::Super]) {
            return Ok(Some(Expression {
                span: Some(s),
                kind: ExpressionKind::Super,
            }));
        }
        if let Some(c) = self.get_const() {
            return Ok(Some(Expression {
                span: Some(c.span),
                kind: ExpressionKind::Const(c),
            }));
        }
        if let Some((_, s0)) = self.get_punc(&[Punctuation::LeftRound]) {
            return if self.get_keyword(&[Keyword::Class]).is_some() {
                let ex = self.get_punc(&[Punctuation::LeftAngle]);
                self.expect(ex, "`<`")?;

                let ex = self.get_ident();
                let cls = self.expect(ex, "an identifier")?;

                let ex = self.get_right_angle();
                self.expect(ex, "`>`")?;

                let ex = self.get_punc(&[Punctuation::RightRound]);
                self.expect(ex, "`)`")?;

                let ex = self.get_punc(&[Punctuation::LeftRound]);
                self.expect(ex, "`(`")?;

                let (params, s1) = self.get_function_call_args(s0)?;

                Ok(Some(Expression {
                    span: Some(s0.combine(s1)),
                    kind: ExpressionKind::ClassCast(cls, params),
                }))
            } else {
                let ex = self.get_expr()?;
                let expr0 = self.expect(ex, "an expression")?;

                if let Some((_, s1)) = self.get_punc(&[Punctuation::RightRound]) {
                    let e = Expression {
                        span: Some(s0.combine(s1)),
                        kind: expr0.kind,
                    };
                    return Ok(Some(e));
                }

                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,`, `)`, or an operator")?;

                let ex = self.get_expr()?;
                let expr1 = self.expect(ex, "an expression")?;

                if let Some((_, s1)) = self.get_punc(&[Punctuation::RightRound]) {
                    return Ok(Some(Expression {
                        span: Some(s0.combine(s1)),
                        kind: ExpressionKind::Vector2(Box::new((expr0, expr1))),
                    }));
                }

                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,`, `)`, or an operator")?;

                let ex = self.get_expr()?;
                let expr2 = self.expect(ex, "an expression")?;

                let ex = self.get_punc(&[Punctuation::RightRound]);
                let (_, s1) = self.expect(ex, "`)`, or an operator")?;

                Ok(Some(Expression {
                    span: Some(s0.combine(s1)),
                    kind: ExpressionKind::Vector3(Box::new((expr0, expr1, expr2))),
                }))
            };
        }

        let op_tok = self.tokenizer.peek_no_doc(&mut self.errs);
        if let Some(op) = get_prefix_op(op_tok) {
            let op_tok = self.tokenizer.next_no_doc(&mut self.errs);
            let (_, r_prec) = get_prefix_precedence(op);

            let ex = self.get_primary_expr()?;
            let p = self.expect(ex, "an expression")?;
            let rhs = self.get_expr_inner(p, r_prec)?;

            return Ok(Some(Expression {
                span: Some(
                    op_tok
                        .unwrap()
                        .span(self.file, self.text)
                        .combine(rhs.span.unwrap()),
                ),
                kind: ExpressionKind::Prefix {
                    op,
                    expr: Box::new(rhs),
                },
            }));
        }

        let id = if let Some((_, s)) = self.get_keyword(&[Keyword::Default]) {
            Identifier {
                span: s,
                symbol: intern_name("default"),
            }
        } else if let Some(id) = self.get_ident() {
            id
        } else {
            return Ok(None);
        };
        Ok(Some(Expression {
            span: Some(id.span),
            kind: ExpressionKind::Ident(id),
        }))
    }

    fn get_expr_inner(
        &mut self,
        lhs: Expression,
        min_precedence: usize,
    ) -> Result<Expression, ParsingError> {
        let mut lhs = lhs;
        loop {
            let op_tok = self.tokenizer.peek_no_doc(&mut self.errs);

            if let Some(op) = get_postfix_op(op_tok) {
                let (l_prec, _) = get_postfix_precedence(op);
                if l_prec < min_precedence {
                    break;
                }
                let op_tok = self.tokenizer.next_no_doc(&mut self.errs);

                lhs = Expression {
                    span: Some(
                        lhs.span
                            .unwrap()
                            .combine(op_tok.unwrap().span(self.file, self.text)),
                    ),
                    kind: ExpressionKind::Postfix {
                        op,
                        expr: Box::new(lhs),
                    },
                };

                continue;
            }

            if let Some(op) = get_infix_op(op_tok) {
                let (l_prec, r_prec) = get_infix_precedence(op);
                if l_prec < min_precedence {
                    break;
                }
                let op_tok = self.tokenizer.next_no_doc(&mut self.errs);

                lhs = match op {
                    InfixOp::Binary(op) => {
                        let ex = self.get_primary_expr()?;
                        let p = self.expect(ex, "an expression")?;
                        let rhs = self.get_expr_inner(p, r_prec)?;

                        Expression {
                            span: Some(lhs.span.unwrap().combine(rhs.span.unwrap())),
                            kind: ExpressionKind::Binary {
                                op,
                                exprs: Box::new(BinaryOpExprs { lhs, rhs }),
                            },
                        }
                    }
                    InfixOp::Ternary => {
                        let ex = self.get_expr()?;
                        let mhs = self.expect(ex, "an expression")?;

                        let ex = self.get_punc(&[Punctuation::Colon]);
                        self.expect(ex, "`:`")?;

                        let ex = self.get_primary_expr()?;
                        let p = self.expect(ex, "an expression")?;
                        let rhs = self.get_expr_inner(p, r_prec)?;

                        Expression {
                            span: Some(lhs.span.unwrap().combine(rhs.span.unwrap())),
                            kind: ExpressionKind::Ternary(Box::new(TernaryOpExprs {
                                cond: lhs,
                                if_true: mhs,
                                if_false: rhs,
                            })),
                        }
                    }
                    InfixOp::LeftSquare => {
                        let ex = self.get_expr()?;
                        let rhs = self.expect(ex, "an expression")?;

                        let ex = self.get_punc(&[Punctuation::RightSquare]);
                        let (_, s1) = self.expect(ex, "`]`")?;

                        Expression {
                            span: Some(lhs.span.unwrap().combine(s1)),
                            kind: ExpressionKind::ArrayIndex(Box::new(ArrayIndexExprs {
                                lhs,
                                index: rhs,
                            })),
                        }
                    }
                    InfixOp::LeftRound => {
                        let (exprs, s1) = self
                            .get_function_call_args(op_tok.unwrap().span(self.file, self.text))?;
                        Expression {
                            span: Some(lhs.span.unwrap().combine(s1)),
                            kind: ExpressionKind::FunctionCall {
                                lhs: Box::new(lhs),
                                exprs,
                            },
                        }
                    }
                };

                continue;
            }

            break;
        }
        Ok(lhs)
    }

    fn get_function_call_args(
        &mut self,
        s0: Span,
    ) -> Result<(Vec<FunctionCallArg>, Span), ParsingError> {
        if let Some((_, s1)) = self.get_punc(&[Punctuation::RightRound]) {
            Ok((vec![], s0.combine(s1)))
        } else {
            let mut exprs = vec![];
            loop {
                let mut seen = false;
                if let Some(Token {
                    data: TokenData::Identifier { .. },
                    ..
                }) = self.tokenizer.peek_no_doc(&mut self.errs)
                {
                    if let Some(Token {
                        data: TokenData::Punctuation(Punctuation::Colon),
                        ..
                    }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
                    {
                        let id = self.get_ident().expect("should get an ident at this point");
                        self.get_punc(&[Punctuation::Colon])
                            .expect("should get a `:` at this point");

                        let ex = self.get_expr()?;
                        let expr = self.expect(ex, "an expression")?;

                        exprs.push(FunctionCallArg {
                            span: id.span.combine(expr.span.unwrap()),
                            kind: FunctionCallArgKind::Named(id, expr),
                        });

                        seen = true;
                    }
                }
                if !seen {
                    let ex = self.get_expr()?;
                    let expr = self.expect(ex, "an identifier or expression")?;

                    exprs.push(FunctionCallArg {
                        span: expr.span.unwrap(),
                        kind: FunctionCallArgKind::Unnamed(expr),
                    });
                }

                if let Some((_, s1)) = self.get_punc(&[Punctuation::RightRound]) {
                    break Ok((exprs, s0.combine(s1)));
                }

                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,`")?;
            }
        }
    }

    fn get_expr(&mut self) -> Result<Option<Expression>, ParsingError> {
        let p = if let Some(p) = self.get_primary_expr()? {
            p
        } else {
            return Ok(None);
        };
        self.get_expr_inner(p, 0).map(Some)
    }

    fn get_expr_list(&mut self) -> Result<Option<ExprList>, ParsingError> {
        let expr = if let Some(e) = self.get_expr()? {
            e
        } else {
            return Ok(None);
        };
        let mut list = vec1![expr];

        while self.get_punc(&[Punctuation::Comma]).is_some() {
            let ex = self.get_expr()?;
            let expr = self.expect(ex, "an expression")?;

            list.push(expr);
        }

        Ok(Some(ExprList {
            span: list[0]
                .span
                .unwrap()
                .combine(list[list.len() - 1].span.unwrap()),
            list,
        }))
    }

    fn get_compound_statement(&mut self) -> Result<Option<CompoundStatement>, ParsingError> {
        let p = self.get_punc(&[Punctuation::LeftCurly]);
        if p.is_none() {
            return Ok(None);
        }
        let (_, s0) = p.unwrap();

        let mut statements = vec![];

        let s1 = loop {
            if let Some((_, s)) = self.get_punc(&[Punctuation::RightCurly]) {
                break s;
            }

            let ex = self.get_statement()?;
            let stmt = self.expect(ex, "a statement")?;

            match stmt.kind {
                StatementKind::Empty => {}
                _ => {
                    statements.push(stmt);
                }
            }
        };

        Ok(Some(CompoundStatement {
            span: s0.combine(s1),
            statements,
        }))
    }

    fn get_multi_assign_declaration(
        &mut self,
    ) -> Result<Option<LocalVariableDefinition>, ParsingError> {
        if let Some(Token {
            data: TokenData::Keyword(Keyword::Let),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            if let Some(Token {
                data: TokenData::Punctuation(Punctuation::LeftSquare),
                ..
            }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
            {
                let let_span = self
                    .tokenizer
                    .next_no_doc(&mut self.errs)
                    .unwrap()
                    .span(self.file, self.text);
                self.tokenizer.next_no_doc(&mut self.errs).unwrap();
                let mut idents = vec![];
                let mut first = true;
                loop {
                    if !first {
                        let ex = self.get_punc(&[Punctuation::RightSquare]);
                        if ex.is_some() {
                            break;
                        }

                        let ex = self.get_punc(&[Punctuation::Comma]);
                        self.expect(ex, "`,` or `]`")?;
                    }
                    first = false;

                    let ex = self.get_ident();
                    let ident = self.expect(ex, "an identifier")?;
                    idents.push(ident);
                }
                let idents = Vec1::try_from(idents).unwrap();

                let ex = self.get_punc(&[Punctuation::Assign]);
                self.expect(ex, "`=`")?;

                let ex = self.get_expr()?;
                let expr = self.expect(ex, "an expression")?;

                return Ok(Some(LocalVariableDefinition {
                    span: let_span.combine(expr.span.unwrap()),
                    var_type: Type {
                        span: let_span,
                        kind: TypeKind::Let,
                    },
                    inits: vec1![VarInit {
                        span: idents[0].span.combine(expr.span.unwrap()),
                        kind: VarInitKind::Multi {
                            names: idents,
                            val: expr,
                        }
                    }],
                }));
            }
        }
        Ok(None)
    }

    fn get_variable_declaration(
        &mut self,
    ) -> Result<Option<LocalVariableDefinition>, ParsingError> {
        if let Some(Token {
            data: TokenData::Identifier(id),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            let id_sym = intern_name(id);
            if id_sym != intern_name("array")
                && id_sym != intern_name("map")
                && id_sym != intern_name("mapiterator")
                && !matches!(
                    self.tokenizer.peek_twice_no_doc(&mut self.errs),
                    Some(Token {
                        data: TokenData::Identifier { .. },
                        ..
                    })
                )
            {
                return Ok(None);
            }
        }

        let var_type = if let Some(var_type) = self.get_single_type()? {
            var_type
        } else {
            return Ok(None);
        };

        let mut ret: Option<Vec1<VarInit>> = None;

        loop {
            let ex = self.get_ident();
            let name = self.expect(ex, "an identifier")?;
            let mut s1 = name.span;

            let sizes = self.get_array_sizes()?;
            let init = match sizes {
                Some(sizes) => {
                    s1 = sizes.span;
                    let vals = if self.get_punc(&[Punctuation::Assign]).is_some() {
                        let ex = self.get_punc(&[Punctuation::LeftCurly]);
                        self.expect(ex, "`{`")?;

                        let ex = self.get_expr_list()?;
                        let vals = self.expect(ex, "an expression")?;

                        let ex = self.get_punc(&[Punctuation::RightCurly]);
                        let (_, s) = self.expect(ex, "`}`")?;
                        s1 = s;

                        Some(vals)
                    } else {
                        None
                    };

                    VarInit {
                        span: name.span.combine(s1),
                        kind: VarInitKind::Array {
                            name,
                            sizes: Some(sizes),
                            vals,
                        },
                    }
                }
                None => {
                    if self.get_punc(&[Punctuation::Assign]).is_some() {
                        if self.get_punc(&[Punctuation::LeftCurly]).is_some() {
                            let ex = self.get_expr_list()?;
                            let vals = self.expect(ex, "an expression")?;

                            let ex = self.get_punc(&[Punctuation::RightCurly]);
                            let (_, s) = self.expect(ex, "`}`")?;
                            s1 = s;

                            VarInit {
                                span: name.span.combine(s1),
                                kind: VarInitKind::Array {
                                    name,
                                    sizes: None,
                                    vals: Some(vals),
                                },
                            }
                        } else {
                            let ex = self.get_expr()?;
                            let expr = self.expect(ex, "an expression or `{`")?;
                            s1 = expr.span.unwrap();
                            VarInit {
                                span: name.span.combine(s1),
                                kind: VarInitKind::Single {
                                    name,
                                    val: Some(expr),
                                },
                            }
                        }
                    } else {
                        VarInit {
                            span: name.span.combine(s1),
                            kind: VarInitKind::Single { name, val: None },
                        }
                    }
                }
            };

            match &mut ret {
                Some(v) => {
                    v.push(init);
                }
                None => {
                    ret = Some(vec1![init]);
                }
            };

            if self.get_punc(&[Punctuation::Comma]).is_some() {
                continue;
            }

            break;
        }

        let inits = ret.unwrap();

        let span = var_type.span.combine(inits[inits.len() - 1].span);
        Ok(Some(LocalVariableDefinition {
            span,
            var_type,
            inits,
        }))
    }

    fn get_if_body(&mut self, s0: Span) -> Result<Option<Statement>, ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let ex = self.get_expr()?;
        let cond = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        self.expect(ex, "`)`")?;

        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;
        let mut s1 = body.span;

        let else_body = if self.get_keyword(&[Keyword::Else]).is_some() {
            let ex = self.get_statement()?;
            let else_body = self.expect(ex, "a statement")?;
            s1 = else_body.span;
            Some(else_body)
        } else {
            None
        }
        .map(Box::new);
        let body = Box::new(body);

        Ok(Some(Statement {
            span: s0.combine(s1),
            kind: StatementKind::If {
                cond,
                body,
                else_body,
            },
        }))
    }

    fn get_switch_body(&mut self, s0: Span) -> Result<Option<Statement>, ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let ex = self.get_expr()?;
        let val = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        self.expect(ex, "`)`")?;

        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;
        let body = Box::new(body);

        Ok(Some(Statement {
            span: s0.combine(body.span),
            kind: StatementKind::Switch { val, body },
        }))
    }

    fn get_while_until_body(
        &mut self,
        start: Keyword,
        s0: Span,
    ) -> Result<Option<Statement>, ParsingError> {
        let iter_type = match start {
            Keyword::While => CondIterType::While,
            Keyword::Until => CondIterType::Until,
            _ => panic!("get_while_until_body called with invalid starting keyword"),
        };

        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let ex = self.get_expr()?;
        let cond = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        self.expect(ex, "`)`")?;

        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;
        let body = Box::new(body);

        Ok(Some(Statement {
            span: s0.combine(body.span),
            kind: StatementKind::CondIter {
                cond,
                body,
                iter_type,
            },
        }))
    }

    fn get_do_body(&mut self, s0: Span) -> Result<Option<Statement>, ParsingError> {
        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;

        let ex = self.get_keyword(&[Keyword::While, Keyword::Until]);
        let (iter_type, _) = self.expect(ex, "`while` or `until`")?;
        let iter_type = match iter_type {
            Keyword::While => CondIterType::DoWhile,
            Keyword::Until => CondIterType::DoUntil,
            _ => unreachable!(),
        };

        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let ex = self.get_expr()?;
        let cond = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        let (_, s1) = self.expect(ex, "`)`")?;

        let body = Box::new(body);
        Ok(Some(Statement {
            span: s0.combine(s1),
            kind: StatementKind::CondIter {
                cond,
                body,
                iter_type,
            },
        }))
    }

    fn get_for_init(&mut self) -> Result<Option<ForInit>, ParsingError> {
        if let Some(var_def) = self.get_variable_declaration()? {
            Ok(Some(ForInit {
                span: var_def.span,
                kind: ForInitKind::VarDef(var_def),
            }))
        } else if let Some(expr_list) = self.get_expr_list()? {
            Ok(Some(ForInit {
                span: expr_list.span,
                kind: ForInitKind::ExprList(expr_list),
            }))
        } else {
            Ok(None)
        }
    }

    fn get_for_body(&mut self, s0: Span) -> Result<Option<Statement>, ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let init = self.get_for_init()?;

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        self.expect(ex, "`;`")?;

        let cond = self.get_expr()?;

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        self.expect(ex, "`;`")?;

        let update = self.get_expr_list()?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        self.expect(ex, "`)`")?;

        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;
        let body = Box::new(body);

        Ok(Some(Statement {
            span: s0.combine(body.span),
            kind: StatementKind::For {
                init,
                cond,
                update,
                body,
            },
        }))
    }

    fn get_foreach_to_colon(&mut self) -> Result<ForEachKind, ParsingError> {
        let ex = self.get_ident();
        let first = self.expect(ex, "an identifier")?;

        let typed_second_maybe = self.get_ident();

        if let Some(second) = typed_second_maybe {
            let ex = self.get_punc(&[Punctuation::Colon]);
            self.expect(ex, "`:`")?;

            return Ok(ForEachKind::Typed {
                ty: first,
                name: second,
            });
        }

        let ex = self.get_punc(&[Punctuation::Comma, Punctuation::Colon]);
        let (p, _) = self.expect(ex, "`,`, `:` or an identifier")?;
        if p == Punctuation::Colon {
            return Ok(ForEachKind::Single { name: first });
        }

        let ex = self.get_ident();
        let second = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Comma, Punctuation::Colon]);
        let (p, _) = self.expect(ex, "`,` or `:`")?;
        if p == Punctuation::Colon {
            return Ok(ForEachKind::KeyVal {
                key: first,
                val: second,
            });
        }

        let ex = self.get_ident();
        let third = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Colon]);
        self.expect(ex, "`:`")?;

        Ok(ForEachKind::ThreeVal {
            var: first,
            pos: second,
            flags: third,
        })
    }

    fn get_foreach_body(&mut self, s0: Span) -> Result<Option<Statement>, ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftRound]);
        self.expect(ex, "`(`")?;

        let kind = self.get_foreach_to_colon()?;

        let ex = self.get_expr()?;
        let iterator = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightRound]);
        self.expect(ex, "`)`")?;

        let ex = self.get_statement()?;
        let body = self.expect(ex, "a statement")?;
        let body = Box::new(body);

        Ok(Some(Statement {
            span: s0.combine(body.span),
            kind: StatementKind::ForEach {
                kind,
                iterator,
                body,
            },
        }))
    }

    fn get_static_const_body(
        &mut self,
        s0: Span,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<StaticConstArray>, ParsingError> {
        if self.get_keyword(&[Keyword::Const]).is_none() {
            return Ok(None);
        }

        let ex = self.get_single_type()?;
        let arr_type = self.expect(ex, "a type")?;

        let got_brackets = if self.get_punc(&[Punctuation::LeftSquare]).is_some() {
            let ex = self.get_punc(&[Punctuation::RightSquare]);
            self.expect(ex, "`]`")?;
            true
        } else {
            false
        };

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;

        if !got_brackets {
            let ex = self.get_punc(&[Punctuation::LeftSquare]);
            self.expect(ex, "`[`")?;

            let ex = self.get_punc(&[Punctuation::RightSquare]);
            self.expect(ex, "`]`")?;
        }

        let ex = self.get_punc(&[Punctuation::Assign]);
        self.expect(ex, "`=`")?;

        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        self.expect(ex, "`{`")?;

        let ex = self.get_expr_list()?;
        let exprs = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::RightCurly]);
        self.expect(ex, "`}`")?;

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        let (_, s1) = self.expect(ex, "`;`")?;

        Ok(Some(StaticConstArray {
            doc_comment,
            span: s0.combine(s1),
            arr_type,
            name,
            exprs,
        }))
    }

    fn get_statement(&mut self) -> Result<Option<Statement>, ParsingError> {
        if let Some(c) = self.get_compound_statement()? {
            return Ok(Some(Statement {
                span: c.span,
                kind: StatementKind::Compound(c),
            }));
        }
        if let Some(stmt) = self.get_multi_assign_declaration()? {
            let ex = self.get_punc(&[Punctuation::Semicolon]);
            self.expect(ex, "`;`")?;

            return Ok(Some(Statement {
                span: stmt.span,
                kind: StatementKind::LocalVariableDefinition(stmt),
            }));
        }
        if let Some(stmt) = self.get_variable_declaration()? {
            let ex = self.get_punc(&[Punctuation::Semicolon]);
            self.expect(ex, "`;`")?;

            return Ok(Some(Statement {
                span: stmt.span,
                kind: StatementKind::LocalVariableDefinition(stmt),
            }));
        }
        if let Some((k, s0)) = self.get_keyword(&[
            Keyword::If,
            Keyword::Switch,
            Keyword::For,
            Keyword::ForEach,
            Keyword::While,
            Keyword::Until,
            Keyword::Do,
            Keyword::Case,
            Keyword::Default,
            Keyword::Static,
            Keyword::Continue,
            Keyword::Break,
            Keyword::Return,
        ]) {
            return match k {
                Keyword::If => self.get_if_body(s0),
                Keyword::Switch => self.get_switch_body(s0),
                Keyword::For => self.get_for_body(s0),
                Keyword::ForEach => self.get_foreach_body(s0),
                Keyword::While | Keyword::Until => self.get_while_until_body(k, s0),
                Keyword::Do => self.get_do_body(s0),
                Keyword::Static => {
                    let ex = self.get_static_const_body(s0, None)?;
                    let b = self.expect(ex, "const")?;
                    Ok(Some(Statement {
                        span: s0.combine(b.span),
                        kind: StatementKind::StaticConstArray(b),
                    }))
                }
                Keyword::Case => {
                    let ex = self.get_expr()?;
                    let expr = self.expect(ex, "an expression")?;

                    let ex = self.get_punc(&[Punctuation::Colon]);
                    let (_, s1) = self.expect(ex, "`:`")?;

                    let expr = Box::new(expr);
                    Ok(Some(Statement {
                        span: s0.combine(s1),
                        kind: StatementKind::Labeled(LabeledStatement::Case(expr)),
                    }))
                }
                Keyword::Default => {
                    let ex = self.get_punc(&[Punctuation::Colon]);
                    let (_, s1) = self.expect(ex, "`:`")?;

                    Ok(Some(Statement {
                        span: s0.combine(s1),
                        kind: StatementKind::Labeled(LabeledStatement::Default),
                    }))
                }
                Keyword::Continue => {
                    let ex = self.get_punc(&[Punctuation::Semicolon]);
                    let (_, s1) = self.expect(ex, "`;`")?;

                    Ok(Some(Statement {
                        span: s0.combine(s1),
                        kind: StatementKind::Continue,
                    }))
                }
                Keyword::Break => {
                    let ex = self.get_punc(&[Punctuation::Semicolon]);
                    let (_, s1) = self.expect(ex, "`;`")?;

                    Ok(Some(Statement {
                        span: s0.combine(s1),
                        kind: StatementKind::Break,
                    }))
                }
                Keyword::Return => {
                    let ret = self.get_expr_list()?;

                    let ex = self.get_punc(&[Punctuation::Semicolon]);
                    let (_, s1) = self.expect(ex, "`;`")?;

                    Ok(Some(Statement {
                        span: s0.combine(s1),
                        kind: StatementKind::Return(ret),
                    }))
                }
                _ => unreachable!(),
            };
        }
        if let Some(expr) = self.get_expr()? {
            let ex = self.get_punc(&[Punctuation::Semicolon]);
            self.expect(ex, "`;`")?;

            return Ok(Some(Statement {
                span: expr.span.unwrap(),
                kind: StatementKind::Expression(expr),
            }));
        }
        if let Some((_, s0)) = self.get_punc(&[Punctuation::LeftSquare]) {
            let ex = self.get_expr_list()?;
            let assignees = self.expect(ex, "an expression")?;

            let ex = self.get_punc(&[Punctuation::RightSquare]);
            self.expect(ex, "`]` or `,`")?;

            let ex = self.get_punc(&[Punctuation::Assign]);
            self.expect(ex, "`=`")?;

            let ex = self.get_expr()?;
            let rhs = self.expect(ex, "an expression")?;

            let ex = self.get_punc(&[Punctuation::Semicolon]);
            let (_, s1) = self.expect(ex, "`;`")?;

            return Ok(Some(Statement {
                span: s0.combine(s1),
                kind: StatementKind::MultiAssign { assignees, rhs },
            }));
        }
        if let Some((_, s)) = self.get_punc(&[Punctuation::Semicolon]) {
            return Ok(Some(Statement {
                span: s,
                kind: StatementKind::Empty,
            }));
        }
        Ok(None)
    }

    fn get_single_type(&mut self) -> Result<Option<Type>, ParsingError> {
        if let Some((_, s)) = self.get_keyword(&[Keyword::Let]) {
            Ok(Some(Type {
                span: s,
                kind: TypeKind::Let,
            }))
        } else if let Some((_, s0)) = self.get_keyword(&[Keyword::Class]) {
            if self.get_punc(&[Punctuation::LeftAngle]).is_some() {
                let ex = self.get_dottable_id()?;
                let inner = self.expect(ex, "an identifier")?;

                let ex = self.get_right_angle();
                let (_, s1) = self.expect(ex, "`>`")?;

                Ok(Some(Type {
                    span: s0.combine(s1),
                    kind: TypeKind::Class(Some(inner)),
                }))
            } else {
                Ok(Some(Type {
                    span: s0,
                    kind: TypeKind::Class(None),
                }))
            }
        } else if let Some((_, s0)) = self.get_keyword(&[Keyword::ReadOnly]) {
            let ex = self.get_punc(&[Punctuation::LeftAngle]);
            self.expect(ex, "`<`")?;

            let native = self.get_punc(&[Punctuation::AtSign]).is_some();

            let ex = self.get_ident();
            let inner = self.expect(ex, "an identifier")?;

            let ex = self.get_right_angle();
            let (_, s1) = self.expect(ex, "`>`")?;

            Ok(Some(if native {
                Type {
                    span: s0.combine(s1),
                    kind: TypeKind::ReadonlyNativeType(inner),
                }
            } else {
                Type {
                    span: s0.combine(s1),
                    kind: TypeKind::ReadonlyType(inner),
                }
            }))
        } else if let Some(id) =
            self.get_specific_ident(&["array", "map", "mapiterator", "function"])
        {
            if intern_name("array") == id.symbol {
                let ex = self.get_punc(&[Punctuation::LeftAngle]);
                self.expect(ex, "`<`")?;

                let ex = self.get_type_or_array()?;
                let inner = self.expect(ex, "a type or array")?;

                let ex = self.get_right_angle();
                let (_, s1) = self.expect(ex, "`>`")?;

                Ok(Some(Type {
                    span: id.span.combine(s1),
                    kind: TypeKind::DynArray(Box::new(inner)),
                }))
            } else if intern_name("map") == id.symbol {
                let ex = self.get_punc(&[Punctuation::LeftAngle]);
                self.expect(ex, "`<`")?;

                let ex = self.get_type_or_array()?;
                let key = self.expect(ex, "a type or array")?;

                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,`")?;

                let ex = self.get_type_or_array()?;
                let value = self.expect(ex, "a type or array")?;

                let ex = self.get_right_angle();
                let (_, s1) = self.expect(ex, "`>`")?;

                Ok(Some(Type {
                    span: id.span.combine(s1),
                    kind: TypeKind::Map(Box::new((key, value))),
                }))
            } else if intern_name("mapiterator") == id.symbol {
                let ex = self.get_punc(&[Punctuation::LeftAngle]);
                self.expect(ex, "`<`")?;

                let ex = self.get_type_or_array()?;
                let key = self.expect(ex, "a type or array")?;

                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,`")?;

                let ex = self.get_type_or_array()?;
                let value = self.expect(ex, "a type or array")?;

                let ex = self.get_right_angle();
                let (_, s1) = self.expect(ex, "`>`")?;

                Ok(Some(Type {
                    span: id.span.combine(s1),
                    kind: TypeKind::MapIterator(Box::new((key, value))),
                }))
            } else if intern_name("function") == id.symbol {
                let ex = self.get_punc(&[Punctuation::LeftAngle]);
                self.expect(ex, "`<`")?;

                if self.get_specific_ident(&["void"]).is_some() {
                    let ex = self.get_right_angle();
                    let right_angle_span = self.expect(ex, "`>`")?.1;

                    return Ok(Some(Type {
                        span: id.span.combine(right_angle_span),
                        kind: TypeKind::FuncPtr(FuncPtrKind::Void),
                    }));
                }

                let ex = self.get_keyword(&[Keyword::UI, Keyword::Play, Keyword::ClearScope]);
                let keyword = self.expect(ex, "`ui`, `play` or `clearscope`")?;
                let flag = match keyword.0 {
                    Keyword::UI => FuncPtrFlag::Ui,
                    Keyword::Play => FuncPtrFlag::Play,
                    Keyword::ClearScope => FuncPtrFlag::ClearScope,
                    _ => unreachable!(),
                };

                let ex = self.get_types_or_void()?;
                let return_types = self.expect(ex, "a type list or void")?;

                let ex = self.get_punc(&[Punctuation::LeftRound]);
                let left_span = self.expect(ex, "`(`")?.1;

                let params_kind = if self.get_specific_ident(&["void"]).is_some() {
                    FuncPtrParamsKind::Void
                } else {
                    let mut params = vec![];
                    let mut first = true;
                    loop {
                        if !first && self.get_punc(&[Punctuation::Comma]).is_none() {
                            break;
                        }
                        first = false;
                        let (mut flags, s0) = self.get_func_param_flags()?;

                        let ex = self.get_single_type()?;
                        let param_type = if let Some(t) = ex {
                            t
                        } else {
                            return if !flags.is_empty() {
                                Err(self.expect(ex, "`in`, `out` or a type").unwrap_err())
                            } else {
                                break;
                            };
                        };
                        if let Some((_, span)) = self.get_punc(&[Punctuation::BitwiseAnd]) {
                            flags.push(ParamFlagItem {
                                span,
                                kind: ParamFlagItemKind::Out,
                            });
                        }
                        let s0 = s0.unwrap_or(param_type.span);
                        params.push(FuncPtrParam {
                            span: s0,
                            flags,
                            ty: param_type,
                        });
                    }
                    FuncPtrParamsKind::List(params)
                };

                let ex = self.get_punc(&[Punctuation::RightRound]);
                let right_span = self.expect(ex, "`)`")?.1;

                let params = FuncPtrParams {
                    span: left_span.combine(right_span),
                    kind: params_kind,
                };

                let ex = self.get_right_angle();
                let right_angle_span = self.expect(ex, "`>`")?.1;

                Ok(Some(Type {
                    span: id.span.combine(right_angle_span),
                    kind: TypeKind::FuncPtr(FuncPtrKind::Detailed(FuncPtrDetailed {
                        flag,
                        return_types,
                        params,
                    })),
                }))
            } else {
                unreachable!()
            }
        } else if let Some((_, s0)) = self.get_punc(&[Punctuation::Dot]) {
            let ex = self.get_dottable_id()?;
            let ty = self.expect(ex, "an identifier")?;

            Ok(Some(Type {
                span: s0.combine(ty.span),
                kind: TypeKind::DottedUserType(ty),
            }))
        } else {
            let p = self.get_punc(&[Punctuation::AtSign]);
            let native = p.is_some();

            let ident = if let Some(id) = self.get_ident() {
                id
            } else {
                return Ok(None);
            };

            let span = match p {
                Some((_, s0)) => s0.combine(ident.span),
                None => ident.span,
            };

            Ok(Some(Type {
                span,
                kind: if native {
                    TypeKind::NativeType(ident)
                } else {
                    TypeKind::SingleUserType(ident)
                },
            }))
        }
    }

    fn get_array_sizes(&mut self) -> Result<Option<ArraySizes>, ParsingError> {
        let mut ret = None;
        while let Some((_, s0)) = self.get_punc(&[Punctuation::LeftSquare]) {
            let expr = self.get_expr()?;

            let ex = self.get_punc(&[Punctuation::RightSquare]);
            let (_, s1) = self.expect(ex, "`]`")?;

            match &mut ret {
                Some(ArraySizes { list: v, span }) => {
                    v.push(expr);
                    *span = span.combine(s1);
                }
                None => {
                    ret = Some(ArraySizes {
                        span: s0.combine(s1),
                        list: vec1![expr],
                    });
                }
            }
        }
        Ok(ret)
    }

    fn get_type_or_array(&mut self) -> Result<Option<TypeOrArray>, ParsingError> {
        let ty = if let Some(t) = self.get_single_type()? {
            t
        } else {
            return Ok(None);
        };

        let sizes = self.get_array_sizes()?;

        Ok(Some(match sizes {
            Some(list) => TypeOrArray {
                span: ty.span.combine(list.span),
                kind: TypeOrArrayKind::Array(ty, list),
            },
            None => TypeOrArray {
                span: ty.span,
                kind: TypeOrArrayKind::Type(ty),
            },
        }))
    }

    fn get_types_or_void(&mut self) -> Result<Option<TypeListOrVoid>, ParsingError> {
        if let Some(id) = self.get_specific_ident(&["void"]) {
            return Ok(Some(TypeListOrVoid {
                span: id.span,
                kind: TypeListOrVoidKind::Void,
            }));
        }

        let t = if let Some(t) = self.get_type_or_array()? {
            t
        } else {
            return Ok(None);
        };
        let mut ret = vec1![t];

        while self.get_punc(&[Punctuation::Comma]).is_some() {
            let ex = self.get_type_or_array()?;
            let t = self.expect(ex, "type")?;
            ret.push(t);
        }

        Ok(Some(TypeListOrVoid {
            span: ret[0].span.combine(ret[ret.len() - 1].span),
            kind: TypeListOrVoidKind::TypeList(ret),
        }))
    }

    fn get_func_param_flags(&mut self) -> Result<(Vec<ParamFlagItem>, Option<Span>), ParsingError> {
        let mut flags = vec![];
        let mut s0 = None;
        loop {
            if let Some((k, s)) = self.get_keyword(&[Keyword::In, Keyword::Out]) {
                s0 = Some(s0.unwrap_or(s));
                match k {
                    Keyword::In => {
                        flags.push(ParamFlagItem {
                            span: s,
                            kind: ParamFlagItemKind::In,
                        });
                    }
                    Keyword::Out => {
                        flags.push(ParamFlagItem {
                            span: s,
                            kind: ParamFlagItemKind::Out,
                        });
                    }
                    _ => unreachable!(),
                }
            } else if let Some(id) = self.get_specific_ident(&["optional"]) {
                s0 = Some(s0.unwrap_or(id.span));
                flags.push(ParamFlagItem {
                    span: id.span,
                    kind: ParamFlagItemKind::Optional,
                });
            } else {
                break;
            }
        }
        Ok((flags, s0))
    }

    fn get_func_param(&mut self) -> Result<Option<FuncParam>, ParsingError> {
        let (mut flags, s0) = self.get_func_param_flags()?;

        let ex = self.get_single_type()?;
        let param_type = if let Some(t) = ex {
            t
        } else {
            return if !flags.is_empty() {
                Err(self.expect(ex, "`in`, `out` or a type").unwrap_err())
            } else {
                Ok(None)
            };
        };
        let s0 = s0.unwrap_or(param_type.span);

        if let Some((_, span)) = self.get_punc(&[Punctuation::BitwiseAnd]) {
            flags.push(ParamFlagItem {
                span,
                kind: ParamFlagItemKind::Out,
            });
        }

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;
        let mut s1 = name.span;

        let init = if self.get_punc(&[Punctuation::Assign]).is_some() {
            let ex = self.get_expr()?;
            let expr = self.expect(ex, "an expression")?;
            s1 = expr.span.unwrap();
            Some(expr)
        } else {
            None
        };

        Ok(Some(FuncParam {
            span: s0.combine(s1),
            flags,
            param_type,
            name,
            init,
        }))
    }

    fn get_func_params(&mut self, s0: Span) -> Result<FuncParams, ParsingError> {
        if self.get_specific_ident(&["void"]).is_some() {
            let ex = self.get_punc(&[Punctuation::RightRound]);
            let (_, s1) = self.expect(ex, "`)`")?;

            return Ok(FuncParams {
                span: s0.combine(s1),
                kind: FuncParamsKind::Void,
            });
        }

        let mut args = vec![];
        let mut variadic = false;

        loop {
            if self.get_punc(&[Punctuation::Ellipsis]).is_some() {
                variadic = true;
                break;
            }

            let ex = self.get_func_param()?;
            let p = if let Some(p) = ex {
                p
            } else {
                break;
            };
            args.push(p);
            if self.get_punc(&[Punctuation::Comma]).is_some() {
                continue;
            }
            break;
        }

        let ex = self.get_punc(&[Punctuation::RightRound]);
        let (_, s1) = self.expect(ex, "`)` or a function argument")?;

        Ok(FuncParams {
            span: s0.combine(s1),
            kind: FuncParamsKind::List { args, variadic },
        })
    }

    fn get_decl_prelude(
        &mut self,
    ) -> Result<Option<(Vec<DeclarationMetadataItem>, Span)>, ParsingError> {
        let mut ret = None;
        let mut top_span = Span {
            start: 0,
            end: 0,
            file: self.file,
        };
        loop {
            if let Some(Token {
                data: TokenData::Keyword(Keyword::ReadOnly),
                ..
            }) = self.tokenizer.peek_no_doc(&mut self.errs)
            {
                if let Some(Token {
                    data: TokenData::Punctuation(Punctuation::LeftAngle),
                    ..
                }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
                {
                    break;
                }
            }
            if let Some((k, s0)) = self.get_keyword(&[
                Keyword::Native,
                Keyword::Static,
                Keyword::Private,
                Keyword::Protected,
                Keyword::Final,
                Keyword::Meta,
                Keyword::Transient,
                Keyword::ReadOnly,
                Keyword::Internal,
                Keyword::Virtual,
                Keyword::Override,
                Keyword::Abstract,
                Keyword::VarArg,
                Keyword::UI,
                Keyword::Play,
                Keyword::ClearScope,
                Keyword::Unsafe,
                Keyword::VirtualScope,
                Keyword::Action,
                Keyword::Deprecated,
                Keyword::Version,
            ]) {
                if ret.is_none() {
                    ret = Some(vec![]);
                    top_span = s0;
                }
                let ret = ret.as_mut().unwrap();
                top_span = top_span.combine(s0);
                match k {
                    Keyword::Native => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Native,
                        });
                    }
                    Keyword::Static => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Static,
                        });
                    }
                    Keyword::Private => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Private,
                        });
                    }
                    Keyword::Protected => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Protected,
                        });
                    }
                    Keyword::Final => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Final,
                        });
                    }
                    Keyword::Meta => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Meta,
                        });
                    }
                    Keyword::Transient => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Transient,
                        });
                    }
                    Keyword::ReadOnly => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::ReadOnly,
                        });
                    }
                    Keyword::Internal => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Internal,
                        });
                    }
                    Keyword::Virtual => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Virtual,
                        });
                    }
                    Keyword::Override => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Override,
                        });
                    }
                    Keyword::Abstract => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Abstract,
                        });
                    }
                    Keyword::VarArg => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::VarArg,
                        });
                    }
                    Keyword::UI => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::UI,
                        });
                    }
                    Keyword::Play => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::Play,
                        });
                    }
                    Keyword::ClearScope => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::ClearScope,
                        });
                    }
                    Keyword::VirtualScope => {
                        ret.push(DeclarationMetadataItem {
                            span: s0,
                            kind: DeclarationMetadataItemKind::VirtualScope,
                        });
                    }

                    Keyword::Unsafe => {
                        let ex = self.get_punc(&[Punctuation::LeftRound]);
                        self.expect(ex, "`(`")?;

                        let ex = self.get_keyword(&[Keyword::ClearScope]);
                        self.expect(ex, "`clearscope`")?;

                        let ex = self.get_punc(&[Punctuation::RightRound]);
                        let (_, s1) = self.expect(ex, "`)`")?;

                        ret.push(DeclarationMetadataItem {
                            span: s0.combine(s1),
                            kind: DeclarationMetadataItemKind::UnsafeClearScope,
                        });
                    }
                    Keyword::Action => {
                        let (types, s) = if self.get_punc(&[Punctuation::LeftRound]).is_some() {
                            let types = self.get_ident_list()?;

                            let ex = self.get_punc(&[Punctuation::RightRound]);
                            let (_, s1) = self.expect(ex, "`,` or `)`")?;
                            top_span = top_span.combine(s1);

                            (Some(types), s0.combine(s1))
                        } else {
                            (None, s0)
                        };

                        ret.push(DeclarationMetadataItem {
                            span: s,
                            kind: DeclarationMetadataItemKind::Action(types),
                        });
                    }
                    Keyword::Deprecated => {
                        let ex = self.get_punc(&[Punctuation::LeftRound]);
                        self.expect(ex, "`(`")?;

                        let ex = self.get_string();
                        let version = self.expect(ex, "a string constant")?;

                        let message = if self.get_punc(&[Punctuation::Comma]).is_some() {
                            let ex = self.get_string();
                            Some(self.expect(ex, "a string constant")?)
                        } else {
                            None
                        };

                        let ex = self.get_punc(&[Punctuation::RightRound]);
                        let (_, sr) = self.expect(
                            ex,
                            if message.is_some() {
                                "`)`"
                            } else {
                                "`,` or `)`"
                            },
                        )?;
                        top_span = top_span.combine(sr);

                        ret.push(DeclarationMetadataItem {
                            span: s0.combine(sr),
                            kind: DeclarationMetadataItemKind::Deprecated { version, message },
                        });
                    }
                    Keyword::Version => {
                        let ex = self.get_punc(&[Punctuation::LeftRound]);
                        self.expect(ex, "`(`")?;

                        let ex = self.get_string();
                        let version = self.expect(ex, "a string constant")?;

                        let ex = self.get_punc(&[Punctuation::RightRound]);
                        let (_, s1) = self.expect(ex, "`)`")?;
                        top_span = top_span.combine(s1);

                        ret.push(DeclarationMetadataItem {
                            span: s0.combine(s1),
                            kind: DeclarationMetadataItemKind::Version(version),
                        });
                    }

                    _ => unreachable!(),
                }
                continue;
            }

            if let Some(id) = self.get_specific_ident(&["latent"]) {
                if ret.is_none() {
                    ret = Some(vec![]);
                    top_span = id.span;
                }
                let ret = ret.as_mut().unwrap();
                ret.push(DeclarationMetadataItem {
                    span: id.span,
                    kind: DeclarationMetadataItemKind::Latent,
                });
                top_span = top_span.combine(id.span);
                continue;
            }

            break;
        }
        Ok(ret.map(|x| (x, top_span)))
    }

    fn get_declaration(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<Declaration>, ParsingError> {
        let metadata = self.get_decl_prelude()?;

        let ex = self.get_types_or_void()?;
        let types = if let Some(t) = ex {
            t
        } else {
            return if metadata.is_none() {
                Ok(None)
            } else {
                Err(self.expect(ex, "a declaration flag or a type").unwrap_err())
            };
        };
        let s0 = metadata.as_ref().map(|(_, x)| *x);
        let metadata = metadata.map(|(x, _)| x).unwrap_or_else(Vec::new);

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;
        let s0 = s0.unwrap_or(name.span);

        if let Some((_, s)) = self.get_punc(&[Punctuation::LeftRound]) {
            let params = self.get_func_params(s)?;

            let constant = if self.get_keyword(&[Keyword::Const]).is_some() {
                FunctionDeclarationConstKind::Const
            } else if self.get_keyword(&[Keyword::Unsafe]).is_some() {
                let ex = self.get_punc(&[Punctuation::LeftRound]);
                self.expect(ex, "`(`")?;

                let ex = self.get_keyword(&[Keyword::Const]);
                self.expect(ex, "`const`")?;

                let ex = self.get_punc(&[Punctuation::RightRound]);
                self.expect(ex, "`)`")?;

                FunctionDeclarationConstKind::UnsafeConst
            } else {
                FunctionDeclarationConstKind::Not
            };

            let (body, s1) = if let Some(c) = self.get_compound_statement()? {
                let s = c.span;
                (Some(c), s)
            } else {
                let ex = self.get_punc(&[Punctuation::Semicolon]);
                let (_, s) = self.expect(ex, "`;` or `{`")?;
                (None, s)
            };

            return Ok(Some(Declaration::Function(FunctionDeclaration {
                doc_comment,
                span: s0.combine(s1),
                name,
                constant,
                metadata,
                return_types: types,
                params,
                body,
            })));
        }

        let sizes0 = self.get_array_sizes()?;

        let mut vars = vec1![(name, sizes0)];
        while self.get_punc(&[Punctuation::Comma]).is_some() {
            let ex = self.get_ident();
            let name = self.expect(ex, "an identifier")?;

            let sizes = self.get_array_sizes()?;

            vars.push((name, sizes));
        }

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        let (_, s1) = self.expect(ex, if vars.len() == 1 { "`;` or `(`" } else { "`;`" })?;

        Ok(Some(Declaration::Member(MemberDeclaration {
            doc_comment,
            span: s0.combine(s1),
            vars,
            metadata,
            member_type: types,
        })))
    }

    pub fn get_class_inner(&mut self) -> Result<Option<ClassInner>, ParsingError> {
        let doc = self.get_doc_comment();
        if let Some(e) = self.get_enum(doc)? {
            return Ok(Some(ClassInner {
                span: e.span,
                kind: ClassInnerKind::Enum(e),
            }));
        }
        if let Some(s) = self.get_struct(doc)? {
            return Ok(Some(ClassInner {
                span: s.span,
                kind: ClassInnerKind::Struct(s),
            }));
        }
        if let Some(c) = self.get_const_def(doc)? {
            return Ok(Some(ClassInner {
                span: c.span,
                kind: ClassInnerKind::Const(c),
            }));
        }
        if let Some(f) = self.get_flag_def(doc)? {
            return Ok(Some(ClassInner {
                span: f.span,
                kind: ClassInnerKind::Flag(f),
            }));
        }
        if let Some(p) = self.get_property_def(doc)? {
            return Ok(Some(ClassInner {
                span: p.span,
                kind: ClassInnerKind::Property(p),
            }));
        }
        if let Some(d) = self.get_default_def()? {
            return Ok(Some(ClassInner {
                span: d.span,
                kind: ClassInnerKind::Default(d),
            }));
        }
        if let Some(s) = self.get_states_def()? {
            return Ok(Some(ClassInner {
                span: s.span,
                kind: ClassInnerKind::States(s),
            }));
        }
        if let Some((_, s0)) = self.get_keyword(&[Keyword::Mixin]) {
            let ex = self.get_ident();
            let i = self.expect(ex, "an identifier")?;

            let ex = self.get_punc(&[Punctuation::Semicolon]);
            let (_, s1) = self.expect(ex, "`;` or `,`")?;

            return Ok(Some(ClassInner {
                span: s0.combine(s1),
                kind: ClassInnerKind::Mixin(i),
            }));
        }
        if let Some(Token {
            data: TokenData::Keyword(Keyword::Static),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            if let Some(Token {
                data: TokenData::Keyword(Keyword::Const),
                ..
            }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
            {
                let s0 = self
                    .tokenizer
                    .next_no_doc(&mut self.errs)
                    .unwrap()
                    .span(self.file, self.text);
                let s = self.get_static_const_body(s0, doc)?.unwrap();
                return Ok(Some(ClassInner {
                    span: s.span,
                    kind: ClassInnerKind::StaticConstArray(s),
                }));
            }
        }
        if let Some(d) = self.get_declaration(doc)? {
            let span = match &d {
                Declaration::Member(r) => r.span,
                Declaration::Function(r) => r.span,
            };
            return Ok(Some(ClassInner {
                span,
                kind: ClassInnerKind::Declaration(d),
            }));
        }
        Ok(None)
    }

    fn get_class_ancestry(&mut self) -> Result<Option<DottableId>, ParsingError> {
        if self.get_punc(&[Punctuation::Colon]).is_none() {
            return Ok(None);
        }

        let ex = self.get_dottable_id()?;
        let ancestor = self.expect(ex, "an identifier")?;

        Ok(Some(ancestor))
    }

    fn get_class_metadata(&mut self) -> Result<Vec<ClassMetadataItem>, ParsingError> {
        let mut ret = vec![];

        while let Some((k, s0)) = self.get_keyword(&[
            Keyword::Abstract,
            Keyword::Native,
            Keyword::UI,
            Keyword::Play,
            Keyword::Replaces,
            Keyword::Version,
            Keyword::Deprecated,
            Keyword::Sealed,
            Keyword::Final,
        ]) {
            match k {
                Keyword::Abstract => {
                    ret.push(ClassMetadataItem {
                        span: s0,
                        kind: ClassMetadataItemKind::Abstract,
                    });
                }
                Keyword::Native => {
                    ret.push(ClassMetadataItem {
                        span: s0,
                        kind: ClassMetadataItemKind::Native,
                    });
                }
                Keyword::UI => {
                    ret.push(ClassMetadataItem {
                        span: s0,
                        kind: ClassMetadataItemKind::UI,
                    });
                }
                Keyword::Play => {
                    ret.push(ClassMetadataItem {
                        span: s0,
                        kind: ClassMetadataItemKind::Play,
                    });
                }
                Keyword::Final => {
                    ret.push(ClassMetadataItem {
                        span: s0,
                        kind: ClassMetadataItemKind::Final,
                    });
                }
                Keyword::Sealed => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ident_list = self.get_ident_list()?;

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, s1) = self.expect(ex, "`)`")?;

                    ret.push(ClassMetadataItem {
                        span: s0.combine(s1),
                        kind: ClassMetadataItemKind::Sealed(ident_list),
                    });
                }
                Keyword::Replaces => {
                    let ex = self.get_dottable_id()?;
                    let replacee = self.expect(ex, "an identifier")?;

                    ret.push(ClassMetadataItem {
                        span: s0.combine(replacee.span),
                        kind: ClassMetadataItemKind::Replaces(replacee),
                    });
                }
                Keyword::Version => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ex = self.get_string();
                    let version = self.expect(ex, "a string constant")?;

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, s1) = self.expect(ex, "`)`")?;

                    ret.push(ClassMetadataItem {
                        span: s0.combine(s1),
                        kind: ClassMetadataItemKind::Version(version),
                    });
                }
                Keyword::Deprecated => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ex = self.get_string();
                    let version = self.expect(ex, "a string constant")?;

                    let message = if self.get_punc(&[Punctuation::Comma]).is_some() {
                        let ex = self.get_string();
                        Some(self.expect(ex, "a string constant")?)
                    } else {
                        None
                    };

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, sr) = self.expect(
                        ex,
                        if message.is_some() {
                            "`)`"
                        } else {
                            "`,` or `)`"
                        },
                    )?;

                    ret.push(ClassMetadataItem {
                        span: s0.combine(sr),
                        kind: ClassMetadataItemKind::Deprecated { version, message },
                    });
                }
                _ => unreachable!(),
            }
        }

        Ok(ret)
    }

    fn get_class_body(&mut self) -> Result<(Vec<ClassInner>, Span), ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        let (_, s0) = self.expect(ex, "`{`")?;

        let mut inners = vec![];

        Ok(loop {
            if let Some(inner) = self.get_class_inner()? {
                inners.push(inner);
            } else if let Some((_, s1)) = self.get_punc(&[Punctuation::RightCurly]) {
                break (inners, s0.combine(s1));
            } else {
                return Err(self
                    .expect::<()>(None, "a class inner element or `}`")
                    .unwrap_err());
            }
        })
    }

    fn get_class(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<ClassDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Class]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let class_name = self.expect(ex, "an identifier")?;

        let ancestor = self.get_class_ancestry()?;
        let metadata = self.get_class_metadata()?;

        let (inners, s1) = self.get_class_body()?;

        Ok(Some(ClassDefinition {
            doc_comment,
            span: s0.combine(s1),
            name: class_name,
            ancestor,
            metadata,
            inners,
        }))
    }

    fn get_mixin_class_inner(&mut self) -> Result<Option<MixinClassInner>, ParsingError> {
        let doc = self.get_doc_comment();
        if let Some(e) = self.get_enum(doc)? {
            return Ok(Some(MixinClassInner {
                span: e.span,
                kind: MixinClassInnerKind::Enum(e),
            }));
        }
        if let Some(s) = self.get_struct(doc)? {
            return Ok(Some(MixinClassInner {
                span: s.span,
                kind: MixinClassInnerKind::Struct(s),
            }));
        }
        if let Some(c) = self.get_const_def(doc)? {
            return Ok(Some(MixinClassInner {
                span: c.span,
                kind: MixinClassInnerKind::Const(c),
            }));
        }
        if let Some(f) = self.get_flag_def(doc)? {
            return Ok(Some(MixinClassInner {
                span: f.span,
                kind: MixinClassInnerKind::Flag(f),
            }));
        }
        if let Some(p) = self.get_property_def(doc)? {
            return Ok(Some(MixinClassInner {
                span: p.span,
                kind: MixinClassInnerKind::Property(p),
            }));
        }
        if let Some(d) = self.get_default_def()? {
            return Ok(Some(MixinClassInner {
                span: d.span,
                kind: MixinClassInnerKind::Default(d),
            }));
        }
        if let Some(s) = self.get_states_def()? {
            return Ok(Some(MixinClassInner {
                span: s.span,
                kind: MixinClassInnerKind::States(s),
            }));
        }
        if let Some(Token {
            data: TokenData::Keyword(Keyword::Static),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            if let Some(Token {
                data: TokenData::Keyword(Keyword::Const),
                ..
            }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
            {
                let s0 = self
                    .tokenizer
                    .next_no_doc(&mut self.errs)
                    .unwrap()
                    .span(self.file, self.text);
                let s = self.get_static_const_body(s0, doc)?.unwrap();
                return Ok(Some(MixinClassInner {
                    span: s.span,
                    kind: MixinClassInnerKind::StaticConstArray(s),
                }));
            }
        }
        if let Some(d) = self.get_declaration(doc)? {
            let span = match &d {
                Declaration::Member(r) => r.span,
                Declaration::Function(r) => r.span,
            };
            return Ok(Some(MixinClassInner {
                span,
                kind: MixinClassInnerKind::Declaration(d),
            }));
        }
        Ok(None)
    }

    fn get_mixin_class_body(&mut self) -> Result<(Vec<MixinClassInner>, Span), ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        let (_, s0) = self.expect(ex, "`{`")?;

        let mut inners = vec![];

        Ok(loop {
            if let Some(inner) = self.get_mixin_class_inner()? {
                inners.push(inner);
            } else if let Some((_, s1)) = self.get_punc(&[Punctuation::RightCurly]) {
                break (inners, s0.combine(s1));
            } else {
                return Err(self
                    .expect::<()>(None, "a mixin class inner element or `}`")
                    .unwrap_err());
            }
        })
    }

    fn get_mixin(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<MixinClassDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Mixin]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_keyword(&[Keyword::Class]);
        self.expect(ex, "`class`")?;

        let ex = self.get_ident();
        let mixin_name = self.expect(ex, "an identifier")?;

        let (inners, s1) = self.get_mixin_class_body()?;

        Ok(Some(MixinClassDefinition {
            doc_comment,
            span: s0.combine(s1),
            name: mixin_name,
            inners,
        }))
    }

    fn get_struct_inner(&mut self) -> Result<Option<StructInner>, ParsingError> {
        let doc = self.get_doc_comment();
        if let Some(e) = self.get_enum(doc)? {
            return Ok(Some(StructInner {
                span: e.span,
                kind: StructInnerKind::Enum(e),
            }));
        }
        if let Some(c) = self.get_const_def(doc)? {
            return Ok(Some(StructInner {
                span: c.span,
                kind: StructInnerKind::Const(c),
            }));
        }
        if let Some(Token {
            data: TokenData::Keyword(Keyword::Static),
            ..
        }) = self.tokenizer.peek_no_doc(&mut self.errs)
        {
            if let Some(Token {
                data: TokenData::Keyword(Keyword::Const),
                ..
            }) = self.tokenizer.peek_twice_no_doc(&mut self.errs)
            {
                let s0 = self
                    .tokenizer
                    .next_no_doc(&mut self.errs)
                    .unwrap()
                    .span(self.file, self.text);
                let s = self.get_static_const_body(s0, doc)?.unwrap();
                return Ok(Some(StructInner {
                    span: s.span,
                    kind: StructInnerKind::StaticConstArray(s),
                }));
            }
        }
        if let Some(d) = self.get_declaration(doc)? {
            let span = match &d {
                Declaration::Member(r) => r.span,
                Declaration::Function(r) => r.span,
            };
            return Ok(Some(StructInner {
                span,
                kind: StructInnerKind::Declaration(d),
            }));
        }
        Ok(None)
    }

    fn get_struct_metadata(&mut self) -> Result<Vec<StructMetadataItem>, ParsingError> {
        let mut ret = vec![];

        while let Some((k, s0)) = self.get_keyword(&[
            Keyword::Native,
            Keyword::UI,
            Keyword::ClearScope,
            Keyword::Play,
            Keyword::Version,
            Keyword::Deprecated,
            Keyword::Unsafe,
        ]) {
            match k {
                Keyword::Native => {
                    ret.push(StructMetadataItem {
                        span: s0,
                        kind: StructMetadataItemKind::Native,
                    });
                }
                Keyword::UI => {
                    ret.push(StructMetadataItem {
                        span: s0,
                        kind: StructMetadataItemKind::UI,
                    });
                }
                Keyword::Play => {
                    ret.push(StructMetadataItem {
                        span: s0,
                        kind: StructMetadataItemKind::Play,
                    });
                }
                Keyword::ClearScope => {
                    ret.push(StructMetadataItem {
                        span: s0,
                        kind: StructMetadataItemKind::ClearScope,
                    });
                }
                Keyword::Version => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ex = self.get_string();
                    let version = self.expect(ex, "a string constant")?;

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, s1) = self.expect(ex, "`)`")?;

                    ret.push(StructMetadataItem {
                        span: s0.combine(s1),
                        kind: StructMetadataItemKind::Version(version),
                    });
                }
                Keyword::Deprecated => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ex = self.get_string();
                    let version = self.expect(ex, "a string constant")?;

                    let message = if self.get_punc(&[Punctuation::Comma]).is_some() {
                        let ex = self.get_string();
                        Some(self.expect(ex, "a string constant")?)
                    } else {
                        None
                    };

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, sr) = self.expect(
                        ex,
                        if message.is_some() {
                            "`)`"
                        } else {
                            "`,` or `)`"
                        },
                    )?;

                    ret.push(StructMetadataItem {
                        span: s0.combine(sr),
                        kind: StructMetadataItemKind::Deprecated { version, message },
                    });
                }
                Keyword::Unsafe => {
                    let ex = self.get_punc(&[Punctuation::LeftRound]);
                    self.expect(ex, "`(`")?;

                    let ex = self.get_keyword(&[Keyword::Internal]);
                    self.expect(ex, "`internal`")?;

                    let ex = self.get_punc(&[Punctuation::RightRound]);
                    let (_, s1) = self.expect(ex, "`)`")?;

                    ret.push(StructMetadataItem {
                        span: s0.combine(s1),
                        kind: StructMetadataItemKind::UnsafeInternal,
                    });
                }
                _ => unreachable!(),
            }
        }

        Ok(ret)
    }

    fn get_struct_body(&mut self) -> Result<(Vec<StructInner>, Span), ParsingError> {
        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        let (_, s0) = self.expect(ex, "'{'")?;

        let mut inners = vec![];

        Ok(loop {
            if let Some(inner) = self.get_struct_inner()? {
                inners.push(inner);
            } else if let Some((_, s1)) = self.get_punc(&[Punctuation::RightCurly]) {
                let s1 = if let Some((_, s)) = self.get_punc(&[Punctuation::Semicolon]) {
                    s
                } else {
                    s1
                };

                break (inners, s0.combine(s1));
            } else {
                return Err(self
                    .expect::<()>(None, "a struct inner element or `}`")
                    .unwrap_err());
            }
        })
    }

    fn get_struct(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<StructDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Struct]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let struct_name = self.expect(ex, "an identifier")?;

        let metadata = self.get_struct_metadata()?;

        let (inners, s1) = self.get_struct_body()?;

        Ok(Some(StructDefinition {
            doc_comment,
            span: s0.combine(s1),
            name: struct_name,
            metadata,
            inners,
        }))
    }

    fn get_extend(&mut self) -> Result<Option<Extend>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Extend]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_keyword(&[Keyword::Class, Keyword::Struct]);
        let (k, _) = self.expect(ex, "`class` or `struct`")?;

        match k {
            Keyword::Class => {
                let ex = self.get_ident();
                let class_name = self.expect(ex, "an identifier")?;

                let (inners, s1) = self.get_class_body()?;

                Ok(Some(Extend::Class(ExtendClass {
                    span: s0.combine(s1),
                    name: class_name,
                    inners,
                })))
            }
            Keyword::Struct => {
                let ex = self.get_ident();
                let struct_name = self.expect(ex, "an identifier")?;

                let (inners, s1) = self.get_struct_body()?;

                Ok(Some(Extend::Struct(ExtendStruct {
                    span: s0.combine(s1),
                    name: struct_name,
                    inners,
                })))
            }
            _ => unreachable!(),
        }
    }

    fn get_enum(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<EnumDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Enum]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;

        let enum_type = if self.get_punc(&[Punctuation::Colon]).is_some() {
            let ex = self.get_specific_ident(&["sbyte", "byte", "short", "ushort", "int", "uint"]);
            let enum_type = self.expect(ex, "an integer type")?;

            let s = enum_type.symbol;
            Some(if s == intern_name("sbyte") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::SByte,
                }
            } else if s == intern_name("byte") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::Byte,
                }
            } else if s == intern_name("short") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::Short,
                }
            } else if s == intern_name("ushort") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::UShort,
                }
            } else if s == intern_name("int") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::Int,
                }
            } else if s == intern_name("uint") {
                IntType {
                    span: enum_type.span,
                    kind: IntTypeKind::UInt,
                }
            } else {
                unreachable!()
            })
        } else {
            None
        };

        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        self.expect(ex, "`{`")?;

        let mut variants = vec![];
        let s1 = loop {
            if let Some((_, s)) = self.get_punc(&[Punctuation::RightCurly]) {
                break s;
            }

            if !variants.is_empty() {
                let ex = self.get_punc(&[Punctuation::Comma]);
                self.expect(ex, "`,` or `}`")?;
                if let Some((_, s)) = self.get_punc(&[Punctuation::RightCurly]) {
                    break s;
                }
            }

            let doc_comment = self.get_doc_comment();

            let ex = self.get_ident();
            let name = self.expect(ex, "`}` or an identifier")?;
            let mut span = name.span;

            let init = if self.get_punc(&[Punctuation::Assign]).is_some() {
                let ex = self.get_expr()?;
                let expr = self.expect(ex, "an expression")?;
                span = span.combine(expr.span.unwrap());
                Some(expr)
            } else {
                None
            };

            variants.push(EnumVariant {
                doc_comment,
                span,
                name,
                init,
            });
        };

        let s1 = if let Some((_, s)) = self.get_punc(&[Punctuation::Semicolon]) {
            s
        } else {
            s1
        };

        Ok(Some(EnumDefinition {
            doc_comment,
            span: s0.combine(s1),
            name,
            enum_type,
            variants,
        }))
    }

    fn get_const_def(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<ConstDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Const]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Assign]);
        self.expect(ex, "`=`")?;

        let ex = self.get_expr()?;
        let expr = self.expect(ex, "an expression")?;

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        let (_, s1) = self.expect(ex, "`;`")?;

        Ok(Some(ConstDefinition {
            doc_comment,
            span: s0.combine(s1),
            name,
            expr,
        }))
    }

    fn get_flag_def(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<FlagDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::FlagDef]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let flag_name = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Colon]);
        self.expect(ex, "`:`")?;

        let ex = self.get_ident();
        let var_name = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Comma]);
        self.expect(ex, "`,`")?;

        let ex = self.get_int();
        let shift = self.expect(ex, "an integer constant")?;

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        let (_, s1) = self.expect(ex, "`;`")?;

        Ok(Some(FlagDefinition {
            doc_comment,
            span: s0.combine(s1),
            flag_name,
            var_name,
            shift,
        }))
    }

    fn get_property_def(
        &mut self,
        doc_comment: Option<StringSymbol>,
    ) -> Result<Option<PropertyDefinition>, ParsingError> {
        let s0 = match self.get_specific_ident(&["property"]) {
            Some(id) => id.span,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_ident();
        let name = self.expect(ex, "an identifier")?;

        let ex = self.get_punc(&[Punctuation::Colon]);
        self.expect(ex, "`:`")?;

        let ex = self.get_ident();
        let var = self.expect(ex, "an identifier")?;
        let mut vars = vec1![var];

        while self.get_punc(&[Punctuation::Comma]).is_some() {
            let ex = self.get_ident();
            let var = self.expect(ex, "an identifier")?;
            vars.push(var);
        }

        let ex = self.get_punc(&[Punctuation::Semicolon]);
        let (_, s1) = self.expect(ex, "`;` or `,`")?;

        Ok(Some(PropertyDefinition {
            doc_comment,
            span: s0.combine(s1),
            name,
            vars,
        }))
    }

    fn get_default_def(&mut self) -> Result<Option<DefaultDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::Default]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        self.expect(ex, "`{`")?;

        let mut statements = vec![];

        let s1 = loop {
            if let Some((_, s)) = self.get_punc(&[Punctuation::RightCurly]) {
                break s;
            }
            if self.get_punc(&[Punctuation::Semicolon]).is_some() {
                continue;
            }
            if let Some(prop) = self.get_dottable_id()? {
                let s0 = prop.span;
                let vals = self.get_expr_list()?;

                let ex = self.get_punc(&[Punctuation::Semicolon]);
                let (_, s1) = self.expect(ex, "`;`")?;

                statements.push(DefaultStatement {
                    span: s0.combine(s1),
                    kind: DefaultStatementKind::Property { prop, vals },
                });

                continue;
            }
            if let Some((p, s0)) = self.get_punc(&[Punctuation::Plus, Punctuation::Minus]) {
                let ex = self.get_dottable_id()?;
                let id = self.expect(ex, "an identifier")?;

                statements.push(match p {
                    Punctuation::Plus => DefaultStatement {
                        span: s0.combine(id.span),
                        kind: DefaultStatementKind::AddFlag(id),
                    },
                    Punctuation::Minus => DefaultStatement {
                        span: s0.combine(id.span),
                        kind: DefaultStatementKind::RemoveFlag(id),
                    },
                    _ => unreachable!(),
                });

                continue;
            }

            return Err(self
                .expect::<()>(None, "`}` or a default item")
                .unwrap_err());
        };

        Ok(Some(DefaultDefinition {
            span: s0.combine(s1),
            statements,
        }))
    }

    fn get_ident_list(&mut self) -> Result<Vec1<Identifier>, ParsingError> {
        let ex = self.get_ident();
        let var = self.expect(ex, "an identifier")?;
        let mut ret = vec1![var];

        while self.get_punc(&[Punctuation::Comma]).is_some() {
            let ex = self.get_ident();
            let var = self.expect(ex, "an identifier")?;
            ret.push(var);
        }

        Ok(ret)
    }

    fn get_states_def(&mut self) -> Result<Option<StatesDefinition>, ParsingError> {
        let s0 = match self.get_keyword(&[Keyword::States]) {
            Some((_, s)) => s,
            None => {
                return Ok(None);
            }
        };

        let opts = if self.get_punc(&[Punctuation::LeftRound]).is_some() {
            let opts = self.get_ident_list()?;

            let ex = self.get_punc(&[Punctuation::RightRound]);
            self.expect(ex, "`)`")?;

            Some(opts)
        } else {
            None
        };

        let ex = self.get_punc(&[Punctuation::LeftCurly]);
        self.expect(ex, if opts.is_some() { "`{`" } else { "`{` or `(`" })?;

        self.tokenizer.set_states_mode(true);

        let mut body = vec![];

        let s1 = loop {
            if let Some((_, s)) = self.get_punc(&[Punctuation::RightCurly]) {
                self.tokenizer.set_states_mode(false);
                break s;
            }

            if let Some(start) = self.get_nws() {
                let sym = start.symbol;
                macro_rules! check {
                    ($s: expr, $r: expr) => {
                        if sym == intern_name($s) {
                            let ex = self.get_punc(&[Punctuation::Semicolon]);
                            let (_, s1) = self.expect(ex, "`;`")?;
                            body.push(StatesBodyItem {
                                span: start.span.combine(s1),
                                kind: $r,
                            });
                            continue;
                        }
                    };
                }
                check!("stop", StatesBodyItemKind::Stop);
                check!("wait", StatesBodyItemKind::Wait);
                check!("fail", StatesBodyItemKind::Fail);
                check!("loop", StatesBodyItemKind::Loop);
                if sym == intern_name("goto") {
                    self.tokenizer.set_states_mode(false);
                    let target = if let Some((_, s0)) = self.get_keyword(&[Keyword::Super]) {
                        let ex = self.get_punc(&[Punctuation::DoubleColon]);
                        self.expect(ex, "`::`")?;

                        let ex = self.get_dottable_id()?;
                        let target = self.expect(ex, "an identifier")?;

                        StateGotoTarget {
                            span: s0.combine(target.span),
                            kind: StateGotoTargetKind::Super(target),
                        }
                    } else {
                        let ex = self.get_dottable_id()?;
                        let target = self.expect(ex, "`super` or an identifier")?;

                        if target.ids.len() == 1
                            && self.get_punc(&[Punctuation::DoubleColon]).is_some()
                        {
                            let scope = target.ids[0];

                            let ex = self.get_dottable_id()?;
                            let target = self.expect(ex, "an identifier")?;

                            StateGotoTarget {
                                span: scope.span.combine(target.span),
                                kind: StateGotoTargetKind::Scoped(scope, target),
                            }
                        } else {
                            StateGotoTarget {
                                span: target.span,
                                kind: StateGotoTargetKind::Unscoped(target),
                            }
                        }
                    };

                    let offset = if self.get_punc(&[Punctuation::Plus]).is_some() {
                        let ex = self.get_expr()?;
                        Some(self.expect(ex, "an expression")?)
                    } else {
                        None
                    };

                    let ex = self.get_punc(&[Punctuation::Semicolon]);
                    let (_, s1) = self.expect(
                        ex,
                        match &target.kind {
                            StateGotoTargetKind::Unscoped(t) if t.ids.len() == 1 => {
                                "`::`, `.`, `+` or `;`"
                            }
                            _ => "`.`, `+` or `;`",
                        },
                    )?;
                    self.tokenizer.set_states_mode(true);

                    body.push(StatesBodyItem {
                        span: start.span.combine(s1),
                        kind: StatesBodyItemKind::Goto { target, offset },
                    });
                    continue;
                }
                if let Some((_, s1)) = self.get_punc(&[Punctuation::Colon]) {
                    body.push(StatesBodyItem {
                        span: start.span.combine(s1),
                        kind: StatesBodyItemKind::Label(start),
                    });
                    continue;
                }

                let sprite = start;
                let ex = self.get_nws();
                let frames = self.expect(ex, "`:` or non-whitespace")?;

                self.tokenizer.set_states_mode(false);

                let ex = self.get_expr()?;
                let duration = self.expect(ex, "an expression")?;

                let mut metadata = vec![];
                while let Some(id) = self.get_specific_ident(&[
                    "bright", "fast", "slow", "nodelay", "canraise", "offset", "light",
                ]) {
                    let s = id.symbol;

                    if s == intern_name("bright") {
                        metadata.push(StateLineMetadataItem {
                            span: id.span,
                            kind: StateLineMetadataItemKind::Bright,
                        });
                    } else if s == intern_name("fast") {
                        metadata.push(StateLineMetadataItem {
                            span: id.span,
                            kind: StateLineMetadataItemKind::Fast,
                        });
                    } else if s == intern_name("slow") {
                        metadata.push(StateLineMetadataItem {
                            span: id.span,
                            kind: StateLineMetadataItemKind::Slow,
                        });
                    } else if s == intern_name("nodelay") {
                        metadata.push(StateLineMetadataItem {
                            span: id.span,
                            kind: StateLineMetadataItemKind::NoDelay,
                        });
                    } else if s == intern_name("canraise") {
                        metadata.push(StateLineMetadataItem {
                            span: id.span,
                            kind: StateLineMetadataItemKind::CanRaise,
                        });
                    } else if s == intern_name("offset") {
                        let ex = self.get_punc(&[Punctuation::LeftRound]);
                        self.expect(ex, "`(`")?;

                        let ex = self.get_expr()?;
                        let expr0 = self.expect(ex, "an expression")?;

                        let ex = self.get_punc(&[Punctuation::Comma]);
                        self.expect(ex, "`,`")?;

                        let ex = self.get_expr()?;
                        let expr1 = self.expect(ex, "an expression")?;

                        let ex = self.get_punc(&[Punctuation::RightRound]);
                        let (_, s1) = self.expect(ex, "`)`")?;

                        metadata.push(StateLineMetadataItem {
                            span: id.span.combine(s1),
                            kind: StateLineMetadataItemKind::Offset(expr0, expr1),
                        });
                    } else if s == intern_name("light") {
                        let ex = self.get_punc(&[Punctuation::LeftRound]);
                        self.expect(ex, "`(`")?;

                        let ex = self.get_string();
                        let var = self.expect(ex, "a string constant")?;
                        let mut list = vec1![var];

                        while self.get_punc(&[Punctuation::Comma]).is_some() {
                            let ex = self.get_string();
                            let var = self.expect(ex, "a string constant")?;
                            list.push(var);
                        }

                        let ex = self.get_punc(&[Punctuation::RightRound]);
                        let (_, s1) = self.expect(ex, "`,` or `)`")?;

                        metadata.push(StateLineMetadataItem {
                            span: id.span.combine(s1),
                            kind: StateLineMetadataItemKind::Light(list),
                        });
                    }
                }

                let (action, s1) = if let Some(c) = self.get_compound_statement()? {
                    let span = c.span;
                    (
                        Some(StateLineAction {
                            span,
                            kind: StateLineActionKind::Anonymous(c),
                        }),
                        span,
                    )
                } else {
                    let action = if let Some(func) = self.get_ident() {
                        let mut span = func.span;
                        let args = if let Some((_, s0)) = self.get_punc(&[Punctuation::LeftRound]) {
                            let (args, s1) = self.get_function_call_args(s0)?;
                            span = span.combine(s1);
                            Some(args)
                        } else {
                            None
                        };

                        Some(StateLineAction {
                            span,
                            kind: StateLineActionKind::Call { func, args },
                        })
                    } else {
                        None
                    };

                    let ex = self.get_punc(&[Punctuation::Semicolon]);
                    let (_, s1) = self.expect(
                        ex,
                        match action {
                            Some(ref a) => match &a.kind {
                                StateLineActionKind::Call { args: Some(_), .. } => "`;`",
                                StateLineActionKind::Call { args: None, .. } => "`(` or `;`",
                                _ => unreachable!(),
                            },
                            None => "a state metadata element, `;` or an identifier",
                        },
                    )?;

                    (action, s1)
                };

                self.tokenizer.set_states_mode(true);

                let span = sprite.span.combine(s1);
                body.push(StatesBodyItem {
                    span,
                    kind: StatesBodyItemKind::Line(StateLine {
                        span,
                        sprite,
                        frames,
                        metadata,
                        duration,
                        action,
                    }),
                });

                continue;
            }

            return Err(self
                .expect::<()>(
                    None,
                    "`stop`, `wait`, `fail`, `loop`, `goto` or non-whitespace",
                )
                .unwrap_err());
        };

        Ok(Some(StatesDefinition {
            span: s0.combine(s1),
            opts: None,
            body,
        }))
    }

    pub fn parse(mut self) -> ParserResult {
        let version = match self.get_lump_version() {
            Ok(v) => v,
            Err(e) => {
                self.err(e);
                return ParserResult {
                    file: self.file,
                    ast: TopLevel {
                        version: None,
                        definitions: vec![],
                    },
                    errs: self.errs,
                };
            }
        };

        let mut definitions = vec![];

        loop {
            let doc = self.get_doc_comment();

            if self.tokenizer.peek_no_doc(&mut self.errs).is_none() {
                self.tokenizer.next_no_doc(&mut self.errs);
                break;
            }

            if let Some(Token {
                data: TokenData::Include,
                ..
            }) = self.tokenizer.peek_no_doc(&mut self.errs)
            {
                let t = self.tokenizer.next_no_doc(&mut self.errs).unwrap();

                let ex = self.get_string_concat();
                let path = match self.expect(ex, "a string constant") {
                    Ok(p) => p,
                    Err(e) => {
                        self.err(e);
                        break;
                    }
                };

                definitions.push(TopLevelDefinition {
                    span: t.span(self.file, self.text).combine(path.span),
                    kind: TopLevelDefinitionKind::Include(path),
                });
                continue;
            }
            match self.get_class(doc) {
                Ok(Some(c)) => {
                    definitions.push(TopLevelDefinition {
                        span: c.span,
                        kind: TopLevelDefinitionKind::Class(c),
                    });
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }
            match self.get_mixin(doc) {
                Ok(Some(c)) => {
                    definitions.push(TopLevelDefinition {
                        span: c.span,
                        kind: TopLevelDefinitionKind::MixinClass(c),
                    });
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }
            match self.get_struct(doc) {
                Ok(Some(s)) => {
                    definitions.push(TopLevelDefinition {
                        span: s.span,
                        kind: TopLevelDefinitionKind::Struct(s),
                    });
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }
            match self.get_extend() {
                Ok(Some(e)) => {
                    match e {
                        Extend::Class(c) => {
                            definitions.push(TopLevelDefinition {
                                span: c.span,
                                kind: TopLevelDefinitionKind::ExtendClass(c),
                            });
                        }
                        Extend::Struct(s) => {
                            definitions.push(TopLevelDefinition {
                                span: s.span,
                                kind: TopLevelDefinitionKind::ExtendStruct(s),
                            });
                        }
                    }
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }
            match self.get_enum(doc) {
                Ok(Some(e)) => {
                    definitions.push(TopLevelDefinition {
                        span: e.span,
                        kind: TopLevelDefinitionKind::Enum(e),
                    });
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }
            match self.get_const_def(doc) {
                Ok(Some(c)) => {
                    definitions.push(TopLevelDefinition {
                        span: c.span,
                        kind: TopLevelDefinitionKind::Const(c),
                    });
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    self.err(e);
                    break;
                }
            }

            let e = self.expect::<()>(None, "a top level element").unwrap_err();
            self.err(e);
            break;
        }

        ParserResult {
            file: self.file,
            ast: TopLevel {
                version,
                definitions,
            },
            errs: self.errs,
        }
    }
}

#[cfg(test)]
mod test {
    use super::filesystem::{File, Files};
    use super::*;

    fn build_parser(text: &str) -> Parser {
        let mut files = Files::default();
        let fndx = files.add(File::new("test.zs".to_string(), text.as_bytes().to_vec()));
        Parser::new(fndx, text)
    }

    fn assert_no_errors(result: ParserResult) {
        if !result.errs.is_empty() {
            let mut errors = String::new();

            for err in result.errs {
                errors.push_str("- ");
                errors.push_str(&err.msg);
                errors.push('\n');
            }

            errors.pop();
            panic!("Encountered errors:\n{}", errors);
        }
    }

    fn assert_errors(result: ParserResult, expected: &[ParsingError]) {
        assert_eq!(result.errs, expected);
    }

    #[test]
    fn rettype_readonly() {
        const SOURCE: &str = r#"
            version "3.7"

            class rettype_readonly_test {
                readOnly<Dictionary> function_returning_readonly() {
                    return null;
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn class_ptr_array() {
        const SOURCE: &str = r#"
            version "3.7"

            class ptrarray_test {
                void function() {
                    Array<class<Object> > types;
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn multidecl() {
        const SOURCE: &str = r#"
            version "3.7"

            class multidecl_test {
                void function() {
                    int i1, i2;
                    Array<int> arr1, arr2;
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn decl_readonly() {
        const SOURCE: &str = r#"
            version "3.7"

            class multidecl_test {
                void function() {
                    readOnly<int> i = 0;
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn struct_unsafe_internal() {
        const SOURCE: &str = r#"
            version "3.7"

            struct TheThing unsafe(internal) {}
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn decl_const() {
        const SOURCE: &str = r#"
            version "3.7"

            class the_test {
                void function() const {
                    readOnly<int> i = 0;
                }
                void functionAgain() unsafe(const) {
                    readOnly<int> i = 0;
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn decl_clearscope() {
        const SOURCE: &str = r#"
            version "3.7"

            class the_test {
                clearscope void function() {}
                unsafe(clearscope) void functionAgain() {}
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn func_param_ampersand() {
        const SOURCE: &str = r#"
            version "3.7"

            struct TheThing {
                void test(int& argument) {}
                void anotherTest(Function<clearscope int (AType&, NotByReference)>& harderArgument) {}
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn sealed_final() {
        const SOURCE: &str = r#"
            version "3.7"

            class MyClass sealed(AnotherClass) {}
            class MyOtherClass final {}
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn foreach() {
        const SOURCE: &str = r#"
            version "3.7"

            class MyClass {
                void test() {
                    foreach (x : v) {}
                    foreach (x, y : v) {}
                    foreach (x, y, z : v) {}
                    foreach (t x : v) {}
                }
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());
    }

    #[test]
    fn double_right_angle_generic() {
        const SOURCE: &str = r#"
            version "3.7"

            struct TheThing {
                Array<ReadOnly<int>> stuff;
                Array<Array<Map<int, int>>> stuffAgain;
                Array<Array<Array<Array<Map<MapIterator<int, int>, Map<int, int>>>>>> evil;
                Array<Array<Array<Array<Map<MapIterator<int, int>, Map<int, int
                    /// doc comment!
                >>>>>> worse;
            }
        "#;

        assert_no_errors(build_parser(SOURCE).parse());

        const SOURCE2: &str = r#"
            version "3.7"

            struct TheThing {
                Array<Array<int>>= hi;
            }
        "#;

        let parser = build_parser(SOURCE2);
        let file = parser.file;

        assert_errors(
            parser.parse(),
            &[ParsingError {
                level: ParsingErrorLevel::Error,
                msg: "expected an identifier, got `=`".to_string(),
                main_spans: vec1![Span {
                    file,
                    start: 91,
                    end: 92
                }],
                info_spans: vec![],
            }],
        );

        const SOURCE3: &str = r#"
            version "3.7"

            struct TheThing {
                Array<Array<int>>== hi;
            }
        "#;

        let parser = build_parser(SOURCE3);
        let file = parser.file;

        assert_errors(
            parser.parse(),
            &[ParsingError {
                level: ParsingErrorLevel::Error,
                msg: "expected an identifier, got `==`".to_string(),
                main_spans: vec1![Span {
                    file,
                    start: 91,
                    end: 93
                }],
                info_spans: vec![],
            }],
        );
    }

    #[test]
    fn multiline_comment_noterm_eof() {
        const SOURCE: &str = r#"
            version "3.7"

            /*
            class multiline_comment_noterm_eof_test {
                void function() {}
            }
        "#;

        let parser = build_parser(SOURCE);
        let file = parser.file;

        assert_errors(
            parser.parse(),
            &[ParsingError {
                level: ParsingErrorLevel::Error,
                msg: "unterminated block comment".to_string(),
                main_spans: vec1![Span {
                    file,
                    start: 40,
                    end: 154
                }],
                info_spans: vec![],
            }],
        );
    }
}
