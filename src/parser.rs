use regex::Regex;
use lazy_static::*;
use flagset::FlagSet;
use vec1::{Vec1, vec1};

use super::*;
use super::ast::*;
use super::tokenizer::*;
use super::interner::*;

#[derive(Debug)]
pub struct Parser<'src> {
	tokenizer: Tokenizer<'src>,
	errs: Vec<ParsingError>,
	name_interner: NameInterner,
	string_interner: StringInterner
}

#[derive(Debug)]
pub struct ParserResult {
	ast: Option<TopLevel>,
	errs: Vec<ParsingError>,
	name_interner: NameInterner,
	string_interner: StringInterner
}

macro_rules! define_simple_getter {
	($name: ident -> $ret: ty: $p: pat, $v: expr) => {
		#[inline(always)]
		fn $name(&mut self) -> Option<$ret> {
			#[allow(unused)]
			if let Some($p) = self.tokenizer.peek(&mut self.errs) {
				if let Some($p) = self.tokenizer.next(&mut self.errs) {
					Some($v)
				} else { unreachable!() }
			} else { None }
		}
	};
}

fn get_prefix_op(t: &Option<Token>) -> Option<PrefixOp> {
	if t.is_none() { return None; }
	let t = t.as_ref().unwrap();

	match t {
		Token { data: TokenData::Punctuation(p), .. } => match p {
			Punctuation::Plus => Some(PrefixOp::Plus),
			Punctuation::Minus => Some(PrefixOp::Minus),
			Punctuation::Increment => Some(PrefixOp::Increment),
			Punctuation::Decrement => Some(PrefixOp::Decrement),
			Punctuation::LogicalNot => Some(PrefixOp::LogicalNot),
			Punctuation::BitwiseNot => Some(PrefixOp::BitwiseNot),
			_ => None
		}
		Token { data: TokenData::Keyword(k), .. } => match k {
			Keyword::SizeOf  => Some(PrefixOp::SizeOf),
			Keyword::AlignOf => Some(PrefixOp::AlignOf),
			_ => None
		}
		_ => None
	}
}

fn get_postfix_op(t: &Option<Token>) -> Option<PostfixOp> {
	if t.is_none() { return None; }
	let t = t.as_ref().unwrap();

	if let Token { data: TokenData::Punctuation(p), .. } = t {
		match p {
			Punctuation::Increment => Some(PostfixOp::Increment),
			Punctuation::Decrement => Some(PostfixOp::Decrement),
			_ => None
		}
	} else { None }
}

#[derive(Debug, Clone, Copy)]
enum InfixOp {
	Binary(BinaryOp),
	LeftRound,
	LeftSquare,
	Ternary,
}

fn get_infix_op(t: &Option<Token>) -> Option<InfixOp> {
	if t.is_none() { return None; }
	let t = t.as_ref().unwrap();

	match t {
		Token { data: TokenData::Punctuation(p), .. } => match p {
			Punctuation::Plus                     => Some(InfixOp::Binary(BinaryOp::Add)),
			Punctuation::Minus                    => Some(InfixOp::Binary(BinaryOp::Subtract)),
			Punctuation::Times                    => Some(InfixOp::Binary(BinaryOp::Times)),
			Punctuation::Divide                   => Some(InfixOp::Binary(BinaryOp::Divide)),
			Punctuation::Modulo                   => Some(InfixOp::Binary(BinaryOp::Modulo)),
			Punctuation::Raise                    => Some(InfixOp::Binary(BinaryOp::Raise)),

			Punctuation::LeftShift                => Some(InfixOp::Binary(BinaryOp::LeftShift)),
			Punctuation::RightShift               => Some(InfixOp::Binary(BinaryOp::LeftShift)),
			Punctuation::UnsignedRightShift       => Some(InfixOp::Binary(BinaryOp::LeftShift)),

			Punctuation::DotDot                   => Some(InfixOp::Binary(BinaryOp::Concat)),

			Punctuation::LeftAngle                => Some(InfixOp::Binary(BinaryOp::LessThan)),
			Punctuation::LessThanEquals           => Some(InfixOp::Binary(BinaryOp::LessThanEquals)),
			Punctuation::RightAngle               => Some(InfixOp::Binary(BinaryOp::LessThan)),
			Punctuation::GreaterThanEquals        => Some(InfixOp::Binary(BinaryOp::LessThanEquals)),
			Punctuation::Equals                   => Some(InfixOp::Binary(BinaryOp::Equals)),
			Punctuation::NotEquals                => Some(InfixOp::Binary(BinaryOp::NotEquals)),
			Punctuation::ApproxEquals             => Some(InfixOp::Binary(BinaryOp::ApproxEquals)),
			Punctuation::ThreeWayComp             => Some(InfixOp::Binary(BinaryOp::ThreeWayComp)),

			Punctuation::LogicalAnd               => Some(InfixOp::Binary(BinaryOp::LogicalAnd)),
			Punctuation::BitwiseAnd               => Some(InfixOp::Binary(BinaryOp::BitwiseAnd)),
			Punctuation::LogicalOr                => Some(InfixOp::Binary(BinaryOp::LogicalOr)),
			Punctuation::BitwiseOr                => Some(InfixOp::Binary(BinaryOp::BitwiseOr)),
			Punctuation::BitwiseXor               => Some(InfixOp::Binary(BinaryOp::BitwiseXor)),

			Punctuation::DoubleColon              => Some(InfixOp::Binary(BinaryOp::Scope)),
			Punctuation::Dot                      => Some(InfixOp::Binary(BinaryOp::MemberAccess)),

			Punctuation::Assign                   => Some(InfixOp::Binary(BinaryOp::Assign)),
			Punctuation::PlusAssign               => Some(InfixOp::Binary(BinaryOp::PlusAssign)),
			Punctuation::MinusAssign              => Some(InfixOp::Binary(BinaryOp::MinusAssign)),
			Punctuation::TimesAssign              => Some(InfixOp::Binary(BinaryOp::TimesAssign)),
			Punctuation::DivideAssign             => Some(InfixOp::Binary(BinaryOp::DivideAssign)),
			Punctuation::ModuloAssign             => Some(InfixOp::Binary(BinaryOp::ModuloAssign)),
			Punctuation::LeftShiftAssign          => Some(InfixOp::Binary(BinaryOp::LeftShiftAssign)),
			Punctuation::RightShiftAssign         => Some(InfixOp::Binary(BinaryOp::RightShiftAssign)),
			Punctuation::UnsignedRightShiftAssign => Some(InfixOp::Binary(BinaryOp::UnsignedRightShiftAssign)),
			Punctuation::BitwiseOrAssign          => Some(InfixOp::Binary(BinaryOp::BitwiseOrAssign)),
			Punctuation::BitwiseAndAssign         => Some(InfixOp::Binary(BinaryOp::BitwiseAndAssign)),
			Punctuation::BitwiseXorAssign         => Some(InfixOp::Binary(BinaryOp::BitwiseXorAssign)),

			Punctuation::QuestionMark             => Some(InfixOp::Ternary),
			Punctuation::LeftSquare               => Some(InfixOp::LeftSquare),
			Punctuation::LeftRound                => Some(InfixOp::LeftRound),

			_ => None
		}
		Token { data: TokenData::Keyword(k), .. } => match k {
			Keyword::Dot   => Some(InfixOp::Binary(BinaryOp::DotProd)),
			Keyword::Cross => Some(InfixOp::Binary(BinaryOp::CrossProd)),
			Keyword::Is    => Some(InfixOp::Binary(BinaryOp::Is)),
			_ => None
		}
		_ => None
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
		InfixOp::Ternary    => (3, 2),

		InfixOp::LeftRound  => (30, 31),
		InfixOp::LeftSquare => (30, 31),

		InfixOp::Binary(b) => match b {
			Assign                   => (1, 0),
			PlusAssign               => (1, 0),
			MinusAssign              => (1, 0),
			TimesAssign              => (1, 0),
			DivideAssign             => (1, 0),
			ModuloAssign             => (1, 0),
			LeftShiftAssign          => (1, 0),
			RightShiftAssign         => (1, 0),
			UnsignedRightShiftAssign => (1, 0),
			BitwiseOrAssign          => (1, 0),
			BitwiseAndAssign         => (1, 0),
			BitwiseXorAssign         => (1, 0),

			LogicalOr                => (4, 5),

			LogicalAnd               => (6, 7),

			Equals                   => (8, 9),
			NotEquals                => (8, 9),
			ApproxEquals             => (8, 9),

			LessThan                 => (10, 11),
			LessThanEquals           => (10, 11),
			GreaterThan              => (10, 11),
			GreaterThanEquals        => (10, 11),
			ThreeWayComp             => (10, 11),
			Is                       => (10, 11),

			Concat                   => (12, 13),

			BitwiseOr                => (14, 15),

			BitwiseXor               => (16, 17),

			BitwiseAnd               => (18, 19),

			LeftShift                => (20, 21),
			RightShift               => (20, 21),
			UnsignedRightShift       => (20, 21),

			Add                      => (22, 23),
			Subtract                 => (22, 23),

			Times                    => (24, 25),
			Divide                   => (24, 25),
			Modulo                   => (24, 25),
			CrossProd                => (24, 25),
			DotProd                  => (24, 25),

			Raise                    => (26, 27),

			MemberAccess             => (30, 31),

			Scope                    => (32, 33),
		}
	}
}

enum Extend {
	Class(ExtendClass),
	Struct(ExtendStruct)
}

impl<'src> Parser<'src> {
	pub fn new(input: &'src str) -> Self {
		Self {
			tokenizer: Tokenizer::new(input),
			errs: vec![],
			name_interner: NameInterner::default(),
			string_interner: StringInterner::default(),
		}
	}

	#[inline(always)]
	fn get_keyword(&mut self, keywords: &[Keyword]) -> Option<Keyword> {
		if let Some(Token { data: TokenData::Keyword(k), .. }) = self.tokenizer.peek(&mut self.errs) {
			for key in keywords {
				if *key == *k {
					if let Some(Token { data: TokenData::Keyword(k), .. }) = self.tokenizer.next(&mut self.errs) {
						return Some(k);
					} else { unreachable!() }
				}
			}
			None
		} else {
			None
		}
	}
	#[inline(always)]
	fn get_punc(&mut self, puncs: &[Punctuation]) -> Option<Punctuation> {
		if let Some(Token { data: TokenData::Punctuation(p), .. }) = self.tokenizer.peek(&mut self.errs) {
			for punc in puncs {
				if *punc == *p {
					if let Some(Token { data: TokenData::Punctuation(p), .. }) = self.tokenizer.next(&mut self.errs) {
						return Some(p);
					} else { unreachable!() }
				}
			}
			None
		} else {
			None
		}
	}
	#[inline(always)]
	fn get_specific_ident(&mut self, idents: &[&str]) -> Option<Identifier> {
		if let Some(Token { data: TokenData::Identifier(i), .. }) = self.tokenizer.peek(&mut self.errs) {
			let i_sym = self.name_interner.interned(i);
			for id in idents {
				let id_sym = self.name_interner.interned(id);
				if id_sym == i_sym {
					self.tokenizer.next(&mut self.errs);
					return Some(Identifier(i_sym));
				}
			}
			None
		} else {
			None
		}
	}
	#[inline(always)]
	fn get_ident(&mut self) -> Option<Identifier> {
		if let Some(Token { data: TokenData::Identifier(s), .. }) = self.tokenizer.peek(&mut self.errs) {
			let sym = self.name_interner.interned(s);
			self.tokenizer.next(&mut self.errs);
			Some(Identifier(sym))
		} else { None }
	}
	#[inline(always)]
	fn get_nws(&mut self) -> Option<NonWhitespace> {
		if let Some(Token { data: TokenData::NonWhitespace(s), .. }) = self.tokenizer.peek(&mut self.errs) {
			let sym = self.name_interner.interned(s);
			self.tokenizer.next(&mut self.errs);
			Some(NonWhitespace(sym))
		} else { None }
	}
	#[inline(always)]
	fn get_string(&mut self) -> Option<StringSymbol> {
		if let Some(Token { data: TokenData::String(s), .. }) = self.tokenizer.peek(&mut self.errs) {
			let sym = self.string_interner.interned(s);
			self.tokenizer.next(&mut self.errs);
			Some(sym)
		} else { None }
	}
	#[inline(always)]
	fn get_name(&mut self) -> Option<NameSymbol> {
		if let Some(Token { data: TokenData::Name(s), .. }) = self.tokenizer.peek(&mut self.errs) {
			let sym = self.name_interner.interned(s);
			self.tokenizer.next(&mut self.errs);
			Some(sym)
		} else { None }
	}

	define_simple_getter!(get_int -> IntConst:
		Token { data: TokenData::Int { val, long, unsigned }, .. }, IntConst { val, long, unsigned }
	);
	define_simple_getter!(get_float -> FloatConst:
		Token { data: TokenData::Float { val, double }, .. }, FloatConst { val, double }
	);
	#[inline(always)]
	fn get_bool(&mut self) -> Option<bool> {
		if let Some(x) = self.get_keyword(&[Keyword::True, Keyword::False]) {
			Some(x == Keyword::True)
		} else { None }
	}

	#[inline(always)]
	fn get_const(&mut self) -> Option<Const> {
		if let Some(s) = self.get_string() {
			return Some(Const::String(s));
		}
		if let Some(s) = self.get_name() {
			return Some(Const::Name(s));
		}
		if let Some(c) = self.get_int() {
			return Some(Const::Int(c));
		}
		if let Some(c) = self.get_float() {
			return Some(Const::Float(c));
		}
		if let Some(b) = self.get_bool() {
			return Some(Const::Bool(b));
		}
		None
	}

	#[inline(always)]
	fn expect<T>(&mut self, val: Option<T>, msg: &str) -> Result<T, ParsingError> {
		if let Some(v) = val {
			Ok(v)
		} else {
			let next = self.tokenizer.peek(&mut self.errs);
			let err_msg = if let Some(n) = next {
				format!("expected {}, got {}", msg, n.data)
			} else {
				format!("expected {}, got `EOF`", msg)
			};
			Err(ParsingError {
				msg: err_msg
			})
		}
	}

	fn get_lump_version(&mut self) -> Result<VersionInfo, ParsingError> {
		if self.get_keyword(&[Keyword::Version][..]).is_some() {
			let ex = self.get_string();
			let s = self.expect(ex, "string literal")?;
			lazy_static! {
				static ref RE: Regex = Regex::new(
					r"^(?P<Major>[0-9]+)\. *(?P<Minor>[0-9]+)(\. *(?P<Revision>[0-9]+))?$"
				).unwrap();
			}
			let c = RE.captures(&self.string_interner.string(s));
			if let Some(c) = c {
				// TODO: once int_error_matching is stabilized, redo this
				let major = u16::from_str_radix(c.name("Major").unwrap().as_str(), 10).unwrap_or(0);
				let minor = u16::from_str_radix(c.name("Minor").unwrap().as_str(), 10).unwrap_or(0);
				let revision = u16::from_str_radix(c.name("Revision").map(|x| x.as_str()).unwrap_or("0"), 10).unwrap_or(0);

				Ok(VersionInfo::new(major, minor, revision))
			} else {
				Err(ParsingError { msg: "invalid version directive".to_string() })
			}
		} else {
			Ok(VersionInfo::new(2, 3, 0))
		}
	}

	fn get_dottable_id(&mut self) -> Result<Option<DottableId>, ParsingError> {
		let id = match self.get_ident() {
			None => return Ok(None),
			Some(id) => id
		};
		let mut ret = DottableId(vec1![id]);

		while let Some(_) = self.get_punc(&[Punctuation::Dot]) {
			let ex = self.get_ident();
			let id = self.expect(ex, "identifier")?;
			ret.0.push(id);
		}

		Ok(Some(ret))
	}

	fn get_primary_expr(&mut self) -> Result<Option<Expression>, ParsingError> {
		if let Some(c) = self.get_const() {
			return Ok(Some(Expression::Const(c)));
		}

		if self.get_punc(&[Punctuation::LeftRound]).is_some() {
			return if self.get_keyword(&[Keyword::Class]).is_some() {
				let ex = self.get_punc(&[Punctuation::LeftAngle]);
				self.expect(ex, "`<`")?;

				let ex = self.get_ident();
				let cls = self.expect(ex, "an identifier")?;

				let ex = self.get_punc(&[Punctuation::RightAngle]);
				self.expect(ex, "`>`")?;

				let ex = self.get_punc(&[Punctuation::RightRound]);
				self.expect(ex, "`)`")?;

				let ex = self.get_punc(&[Punctuation::LeftRound]);
				self.expect(ex, "`(`")?;

				let params = self.get_function_call_args()?;

				Ok(Some(Expression::ClassCast(cls, params)))
			} else {
				let ex = self.get_expr()?;
				let expr0 = self.expect(ex, "an expression")?;

				if self.get_punc(&[Punctuation::RightRound]).is_some() {
					return Ok(Some(expr0));
				}

				let ex = self.get_punc(&[Punctuation::Comma]);
				self.expect(ex, "`,`")?;

				let ex = self.get_expr()?;
				let expr1 = self.expect(ex, "an expression")?;

				if self.get_punc(&[Punctuation::RightRound]).is_some() {
					return Ok(Some(Expression::Vector2(Box::new((expr0, expr1)))));
				}

				let ex = self.get_punc(&[Punctuation::Comma]);
				self.expect(ex, "`,`")?;

				let ex = self.get_expr()?;
				let expr2 = self.expect(ex, "an expression")?;

				let ex = self.get_punc(&[Punctuation::RightRound]);
				self.expect(ex, "`)`")?;

				Ok(Some(Expression::Vector3(Box::new((expr0, expr1, expr2)))))
			}
		}

		let op_tok = self.tokenizer.peek(&mut self.errs);
		if let Some(op) = get_prefix_op(op_tok) {
			self.tokenizer.next(&mut self.errs);
			let (_, r_prec) = get_prefix_precedence(op);

			let ex = self.get_primary_expr()?;
			let p = self.expect(ex, "an expression")?;
			let rhs = self.get_expr_inner(p, r_prec)?;

			return Ok(Some(Expression::Prefix { op, expr: Box::new(rhs) }));
		}

		let id = if let Some(id) = self.get_ident() { id } else { return Ok(None); };
		Ok(Some(Expression::Ident(id)))
	}

	fn get_expr_inner(
		&mut self, lhs: Expression, min_precedence: usize
	) -> Result<Expression, ParsingError> {
		let mut lhs = lhs;
		loop {
			let op_tok = self.tokenizer.peek(&mut self.errs);

			if let Some(op) = get_postfix_op(op_tok) {
				let (l_prec, _) = get_postfix_precedence(op);
				if l_prec < min_precedence { break; }
				self.tokenizer.next(&mut self.errs);

				lhs = Expression::Postfix {
					op,
					expr: Box::new(lhs)
				};

				continue;
			}

			if let Some(op) = get_infix_op(op_tok) {
				let (l_prec, r_prec) = get_infix_precedence(op);
				if l_prec < min_precedence { break; }
				self.tokenizer.next(&mut self.errs);

				lhs = match op {
					InfixOp::Binary(op) => {
						let ex = self.get_primary_expr()?;
						let p = self.expect(ex, "an expression")?;
						let rhs = self.get_expr_inner(p, r_prec)?;

						Expression::Binary {
							op,
							exprs: Box::new(BinaryOpExprs {
								lhs, rhs
							})
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

						Expression::Ternary(Box::new(TernaryOpExprs {
							cond: lhs,
							if_true: mhs,
							if_false: rhs
						}))
					}
					InfixOp::LeftSquare => {
						let ex = self.get_expr()?;
						let rhs = self.expect(ex, "an expression")?;

						let ex = self.get_punc(&[Punctuation::RightSquare]);
						self.expect(ex, "`]`")?;

						Expression::ArrayIndex(Box::new(ArrayIndexExprs {
							lhs, index: rhs
						}))
					}
					InfixOp::LeftRound => {
						Expression::FunctionCall {
							lhs: Box::new(lhs),
							exprs: self.get_function_call_args()?
						}
					}
				};

				continue;
			}

			break;
		}
		Ok(lhs)
	}

	fn get_function_call_args(&mut self) -> Result<Vec<FunctionCallArg>, ParsingError> {
		if self.get_punc(&[Punctuation::RightRound]).is_some() {
			Ok(vec![])
		} else {
			let mut exprs = vec![];
			loop {
				let ex = self.get_expr()?;
				let expr = self.expect(ex, "an identifier or expression")?;

				if let Expression::Ident(id) = expr {
					if self.get_punc(&[Punctuation::Colon]).is_some() {
						let ex = self.get_expr()?;
						let expr = self.expect(ex, "an expression")?;
						exprs.push(FunctionCallArg::Named(id, expr));
					} else {
						exprs.push(FunctionCallArg::Unnamed(Expression::Ident(id)));
					}
				} else {
					exprs.push(FunctionCallArg::Unnamed(expr));
				}

				if self.get_punc(&[Punctuation::RightRound]).is_some() {
					break Ok(exprs);
				}

				let ex = self.get_punc(&[Punctuation::Comma]);
				self.expect(ex, "`,`")?;
			}
		}
	}

	fn get_expr(&mut self) -> Result<Option<Expression>, ParsingError> {
		let p = if let Some(p) = self.get_primary_expr()? { p } else { return Ok(None); };
		self.get_expr_inner(p, 0).map(|x| Some(x))
	}

	fn get_expr_list(&mut self) -> Result<Option<ExprList>, ParsingError> {
		let expr = if let Some(e) = self.get_expr()? { e } else { return Ok(None); };
		let mut ret = vec1![expr];

		while self.get_punc(&[Punctuation::Comma]).is_some() {
			let ex = self.get_expr()?;
			let expr = self.expect(ex, "an expression")?;

			ret.push(expr);
		}

		Ok(Some(ExprList(ret)))
	}

	fn get_compound_statement(
		&mut self
	) -> Result<Option<CompoundStatement>, ParsingError> {
		if self.get_punc(&[Punctuation::LeftCurly]).is_none() { return Ok(None); }

		let mut ret = vec![];

		loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() { break; }

			let ex = self.get_statement()?;
			let stmt = self.expect(ex, "a statement")?;

			match stmt {
				Statement::Empty => {}
				_ => { ret.push(stmt); }
			}
		}

		Ok(Some(CompoundStatement(ret)))
	}

	fn get_variable_declaration(&mut self) -> Result<Option<Statement>, ParsingError> {
		let var_type = if let Some(var_type) = self.get_single_type()? { var_type } else { return Ok(None); };

		let mut ret: Option<Vec1<VarInit>> = None;

		loop {
			let ex = self.get_ident();
			if ex.is_none() {
				if ret.is_some() {
					return Err(self.expect(ex, "an identifier").unwrap_err());
				}
				return match var_type {
					Type::SingleUserType(id) => {
						let lhs = Expression::Ident(id);
						let expr = self.get_expr_inner(lhs, 0)?;

						Ok(Some(Statement::Expression(expr)))
					}
					_ => {
						Err(self.expect(ex, "an identifier").unwrap_err())
					}
				};
			}
			let name = ex.unwrap();

			let sizes = self.get_array_sizes()?;
			let init = match sizes {
				Some(sizes) => {
					let vals = 
						if self.get_punc(&[Punctuation::Assign]).is_some() {
							let ex = self.get_punc(&[Punctuation::LeftCurly]);
							self.expect(ex, "`{`")?;

							let ex = self.get_expr_list()?;
							let vals = self.expect(ex, "an expression")?;

							let ex = self.get_punc(&[Punctuation::RightCurly]);
							self.expect(ex, "`}`")?;

							Some(vals)
						} else { None };

					VarInit::Array {
						name, sizes: Some(sizes), vals
					}
				}
				None => {
					if self.get_punc(&[Punctuation::Assign]).is_some() {
						if self.get_punc(&[Punctuation::LeftCurly]).is_some() {
							let ex = self.get_expr_list()?;
							let vals = self.expect(ex, "an expression")?;

							let ex = self.get_punc(&[Punctuation::RightCurly]);
							self.expect(ex, "`}`")?;

							VarInit::Array {
								name, sizes: None, vals: Some(vals)
							}
						} else {
							let ex = self.get_expr()?;
							let expr = self.expect(ex, "an expression or `{`")?;
							VarInit::Single {
								name, val: Some(expr)
							}
						}
					} else {
						VarInit::Single {
							name, val: None
						}
					}
				}
			};

			match &mut ret {
				Some(v) => { v.push(init); }
				None => { ret = Some(vec1![init]); }
			};

			if self.get_punc(&[Punctuation::Comma]).is_some() {
				continue;
			}

			break;
		}

		let inits = ret.unwrap();

		Ok(Some(Statement::LocalVariableDefinition(LocalVariableDefinition {
			var_type, inits
		})))
	}

	fn get_if_body(&mut self) -> Result<Option<Statement>, ParsingError> {
		let ex = self.get_punc(&[Punctuation::LeftRound]);
		self.expect(ex, "`(`")?;

		let ex = self.get_expr()?;
		let cond = self.expect(ex, "an expression")?;

		let ex = self.get_punc(&[Punctuation::RightRound]);
		self.expect(ex, "`)`")?;

		let ex = self.get_statement()?;
		let body = self.expect(ex, "a statement")?;

		let else_body = if self.get_keyword(&[Keyword::Else]).is_some() {
			let ex = self.get_statement()?;
			let else_body = self.expect(ex, "a statement")?;
			Some(else_body)
		} else {
			None
		}.map(|x| Box::new(x));
		let body = Box::new(body);

		Ok(Some(Statement::If {
			cond, body, else_body
		}))
	}
	
	fn get_switch_body(&mut self) -> Result<Option<Statement>, ParsingError> {
		let ex = self.get_punc(&[Punctuation::LeftRound]);
		self.expect(ex, "`(`")?;

		let ex = self.get_expr()?;
		let val = self.expect(ex, "an expression")?;

		let ex = self.get_punc(&[Punctuation::RightRound]);
		self.expect(ex, "`)`")?;

		let ex = self.get_statement()?;
		let body = self.expect(ex, "a statement")?;
		let body = Box::new(body);

		Ok(Some(Statement::Switch {
			val, body
		}))
	}

	fn get_while_until_body(&mut self, start: Keyword) -> Result<Option<Statement>, ParsingError> {
		let iter_type = match start {
			Keyword::While => CondIterType::While,
			Keyword::Until => CondIterType::Until,
			_ => panic!("get_while_until_body called with invalid starting keyword")
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

		Ok(Some(Statement::CondIter {
			cond, body, iter_type
		}))
	}

	fn get_do_body(&mut self) -> Result<Option<Statement>, ParsingError> {
		let ex = self.get_statement()?;
		let body = self.expect(ex, "a statement")?;

		let ex = self.get_keyword(&[Keyword::While, Keyword::Until]);
		let iter_type = self.expect(ex, "`while` or `until`")?;
		let iter_type = match iter_type {
			Keyword::While => CondIterType::DoWhile,
			Keyword::Until => CondIterType::DoUntil,
			_ => unreachable!()
		};

		let ex = self.get_punc(&[Punctuation::LeftRound]);
		self.expect(ex, "`(`")?;

		let ex = self.get_expr()?;
		let cond = self.expect(ex, "an expression")?;

		let ex = self.get_punc(&[Punctuation::RightRound]);
		self.expect(ex, "`)`")?;

		let body = Box::new(body);
		Ok(Some(Statement::CondIter {
			cond, body, iter_type
		}))
	}

	fn get_for_init(&mut self) -> Result<Option<ForInit>, ParsingError> {
		if let Some(var_def) = self.get_variable_declaration()? {
			match var_def {
				Statement::LocalVariableDefinition(var_dec) => {
					Ok(Some(ForInit::VarDef(var_dec)))
				}
				Statement::Expression(expr) => {
					let mut ret = vec1![expr];
					if self.get_punc(&[Punctuation::Comma]).is_some() {
						let ex = self.get_expr_list()?;
						let list = self.expect(ex, "an expression")?;
						ret.append(&mut list.0.into_vec());
					}
					Ok(Some(ForInit::ExprList(ExprList(ret))))
				}
				_ => unreachable!()
			}
		} else if let Some(expr_list) = self.get_expr_list()? {
			Ok(Some(ForInit::ExprList(expr_list)))
		} else {
			Ok(None)
		}
	}

	fn get_for_body(&mut self) -> Result<Option<Statement>, ParsingError> {
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

		Ok(Some(Statement::For { init, cond, update, body }))
	}

	fn get_static_const_body(&mut self) -> Result<Option<StaticConstArray>, ParsingError> {
		if self.get_keyword(&[Keyword::Const]).is_none() { return Ok(None); }

		let ex = self.get_single_type()?;
		let arr_type = self.expect(ex, "a type")?;

		let ex = self.get_punc(&[Punctuation::LeftSquare]);
		self.expect(ex, "`[`")?;

		let ex = self.get_punc(&[Punctuation::RightSquare]);
		self.expect(ex, "`]`")?;

		let ex = self.get_ident();
		let name = self.expect(ex, "an identifier")?;

		let ex = self.get_punc(&[Punctuation::Assign]);
		self.expect(ex, "`=`")?;

		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, "`{`")?;

		let ex = self.get_expr_list()?;
		let exprs = self.expect(ex, "an expression")?;

		let ex = self.get_punc(&[Punctuation::RightCurly]);
		self.expect(ex, "`}`")?;

		let ex = self.get_punc(&[Punctuation::Semicolon]);
		self.expect(ex, "`;`")?;

		Ok(Some(StaticConstArray {
			arr_type, name, exprs
		}))
	}

	fn get_statement(&mut self) -> Result<Option<Statement>, ParsingError> {
		if let Some(c) = self.get_compound_statement()? {
			Ok(Some(Statement::Compound(c)))
		}
		else if let Some(stmt) = self.get_variable_declaration()? {
			let ex = self.get_punc(&[Punctuation::Semicolon]);
			self.expect(ex, "`;`")?;

			Ok(Some(stmt))
		} else if let Some(k) = self.get_keyword(&[
			Keyword::If, Keyword::Switch, Keyword::For, Keyword::While,
			Keyword::Until, Keyword::Do, Keyword::Case, Keyword::Default,
			Keyword::Static, Keyword::Continue, Keyword::Break,
			Keyword::Return
		]) {
			match k {
				Keyword::If => {
					self.get_if_body()
				}
				Keyword::Switch => {
					self.get_switch_body()
				}
				Keyword::For => {
					self.get_for_body()
				}
				Keyword::While | Keyword::Until => {
					self.get_while_until_body(k)
				}
				Keyword::Do => {
					self.get_do_body()
				}
				Keyword::Static => {
					let ex = self.get_static_const_body()?;
					Ok(Some(Statement::StaticConstArray(self.expect(ex, "`const`")?)))
				}
				Keyword::Case => {
					let ex = self.get_expr()?;
					let expr = self.expect(ex, "an expression")?;

					let ex = self.get_punc(&[Punctuation::Colon]);
					self.expect(ex, "`:`")?;

					let expr = Box::new(expr);
					Ok(Some(Statement::Labeled(LabeledStatement::Case(expr))))
				}
				Keyword::Default => {
					let ex = self.get_punc(&[Punctuation::Colon]);
					self.expect(ex, "`:`")?;

					Ok(Some(Statement::Labeled(LabeledStatement::Default)))
				}
				Keyword::Continue => {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;

					Ok(Some(Statement::Continue))
				}
				Keyword::Break => {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;

					Ok(Some(Statement::Break))
				}
				Keyword::Return => {
					let ret = if let Some(l) = self.get_expr_list()? { Some(l) } else { None };

					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;

					Ok(Some(Statement::Return(ret)))
				}
				_ => unreachable!()
			}
		} else if let Some(expr) = self.get_expr()? {
			let ex = self.get_punc(&[Punctuation::Semicolon]);
			self.expect(ex, "`;`")?;

			Ok(Some(Statement::Expression(expr)))
		} else if self.get_punc(&[Punctuation::LeftSquare]).is_some() {
			let ex = self.get_expr_list()?;
			let assignees = self.expect(ex, "an expression")?;

			let ex = self.get_punc(&[Punctuation::RightSquare]);
			self.expect(ex, "`]` or `,`")?;

			let ex = self.get_punc(&[Punctuation::Assign]);
			self.expect(ex, "`=`")?;

			let ex = self.get_expr()?;
			let rhs = self.expect(ex, "an expression")?;

			let ex = self.get_punc(&[Punctuation::Semicolon]);
			self.expect(ex, "`;`")?;

			Ok(Some(Statement::MultiAssign {
				assignees, rhs
			}))

		} else if self.get_punc(&[Punctuation::Semicolon]).is_some() {
			Ok(Some(Statement::Empty))
		} else {
			Ok(None)
		}
	}

	fn get_single_type(&mut self) -> Result<Option<Type>, ParsingError> {
		if self.get_keyword(&[Keyword::Class]).is_some() {
			if self.get_punc(&[Punctuation::LeftAngle]).is_none() {
				return Ok(Some(Type::Class(None)));
			}
			
			let ex = self.get_dottable_id()?;
			let inner = self.expect(ex, "an identifier")?;

			let ex = self.get_punc(&[Punctuation::RightAngle]);
			self.expect(ex, "`>`")?;

			return Ok(Some(Type::Class(Some(inner))));
		}

		if self.get_keyword(&[Keyword::ReadOnly]).is_some() {
			let ex = self.get_punc(&[Punctuation::LeftAngle]);
			self.expect(ex, "`<`")?;

			let native = self.get_punc(&[Punctuation::AtSign]).is_some();
			
			let ex = self.get_ident();
			let inner = self.expect(ex, "an identifier")?;

			let ex = self.get_punc(&[Punctuation::RightAngle]);
			self.expect(ex, "`>`")?;

			return Ok(Some(if native { Type::ReadonlyNativeType(inner) } else { Type::ReadonlyType(inner) }));
		}

		if let Some(id) = self.get_specific_ident(&["array", "map"]) {
			if self.name_interner.interned("array") == id.0 {
				let ex = self.get_punc(&[Punctuation::LeftAngle]);
				self.expect(ex, "`<`")?;

				let ex = self.get_type_or_array()?;
				let inner = self.expect(ex, "a type or array")?;

				let ex = self.get_punc(&[Punctuation::RightAngle]);
				self.expect(ex, "`>`")?;

				return Ok(Some(Type::DynArray(Box::new(inner))));
			}
			if self.name_interner.interned("map") == id.0 {
				let ex = self.get_punc(&[Punctuation::LeftAngle]);
				self.expect(ex, "`<`")?;

				let ex = self.get_type_or_array()?;
				let key = self.expect(ex, "a type or array")?;

				let ex = self.get_punc(&[Punctuation::Comma]);
				self.expect(ex, "`,`")?;

				let ex = self.get_type_or_array()?;
				let value = self.expect(ex, "a type or array")?;

				let ex = self.get_punc(&[Punctuation::RightAngle]);
				self.expect(ex, "`>`")?;

				return Ok(Some(Type::Map(Box::new((key, value)))));
			}
		}

		if self.get_punc(&[Punctuation::Dot]).is_some() {
			let ex = self.get_dottable_id()?;
			let ty = self.expect(ex, "an identifier")?;

			return Ok(Some(Type::DottedUserType(ty)));
		}

		let native = self.get_punc(&[Punctuation::AtSign]).is_some();

		let ident = if let Some(id) = self.get_ident() { id } else { return Ok(None); };

		Ok(Some(
			if native {
				Type::NativeType(ident)
			} else {
				Type::SingleUserType(ident)
			}
		))
	}

	fn get_array_sizes(&mut self) -> Result<Option<ExprList>, ParsingError> {
		let mut ret = None;
		while let Some(_) = self.get_punc(&[Punctuation::LeftSquare]) {
			let ex = self.get_expr()?;
			let expr = self.expect(ex, "an expression")?;

			match &mut ret {
				Some(ExprList(v)) => { v.push(expr); }
				None => { ret = Some(ExprList(vec1![expr])); }
			}

			let ex = self.get_punc(&[Punctuation::RightSquare]);
			self.expect(ex, "`]`")?;
		}
		Ok(ret)
	}

	fn get_type_or_array(&mut self) -> Result<Option<TypeOrArray>, ParsingError> {
		let ty = if let Some(t) = self.get_single_type()? { t } else { return Ok(None); };

		let sizes = self.get_array_sizes()?;

		Ok(Some(match sizes {
			Some(list) => TypeOrArray::Array(ty, list),
			None       => TypeOrArray::Type(ty)
		}))
	}

	fn get_types_or_void(&mut self) -> Result<Option<TypeListOrVoid>, ParsingError> {
		if let Some(Token { data: TokenData::Identifier(id), .. }) = self.tokenizer.peek(&mut self.errs) {
			if id.to_lowercase() == "void" {
				self.tokenizer.next(&mut self.errs);
				return Ok(Some(TypeListOrVoid::Void));
			}
		}

		let t = if let Some(t) = self.get_type_or_array()? { t } else { return Ok(None); };
		let mut ret = vec![t];

		while let Some(_) = self.get_punc(&[Punctuation::Comma]) {
			let ex = self.get_type_or_array()?;
			let t = self.expect(ex, "type")?;
			ret.push(t);
		}

		Ok(Some(TypeListOrVoid::TypeList(ret)))
	}

	fn get_func_param(&mut self) -> Result<Option<FuncParam>, ParsingError> {
		let mut flags = FlagSet::default();
		loop {
			if let Some(k) = self.get_keyword(&[Keyword::In, Keyword::Out]) {
				match k {
					Keyword::In       => { flags |= ParamFlags::In       }
					Keyword::Out      => { flags |= ParamFlags::Out      }
					_ => unreachable!()
				}
			} else if self.get_specific_ident(&["optional"]).is_some() {
				flags |= ParamFlags::Optional;
			} else { break; }
		}

		let ex = self.get_single_type()?;
		let param_type = if let Some(t) = ex { t } else {
			return if !flags.is_empty() {
				Err(self.expect(ex, "`in`, `out` or a type").unwrap_err())
			} else {
				Ok(None)
			}
		};

		let ex = self.get_ident();
		dbg!(&param_type);
		if let Type::SingleUserType(Identifier(i)) = param_type {
			dbg!(self.name_interner.string(i));
		}
		dbg!(&self.tokenizer);
		let name = self.expect(ex, "an identifier")?;

		let init = if self.get_punc(&[Punctuation::Assign]).is_some() {
			let ex = self.get_expr()?;
			let expr = self.expect(ex, "an expression")?;
			Some(expr)
		} else { None };

		Ok(Some(FuncParam {
			flags, param_type, name, init
		}))
	}

	fn get_func_params(&mut self) -> Result<FuncParams, ParsingError> {
		if self.get_specific_ident(&["void"]).is_some() {
			return Ok(FuncParams::Void);
		}

		let mut args = vec![];
		let mut variadic = false;

		loop {
			if self.get_punc(&[Punctuation::Ellipsis]).is_some() {
				variadic = true;
				break;
			}

			let ex = self.get_func_param()?;
			let p = if let Some(p) = ex { p } else { break; };
			args.push(p);
			if self.get_punc(&[Punctuation::Comma]).is_some() {
				continue;
			}
			break;
		}

		Ok(FuncParams::List {
			args, variadic
		})
	}

	fn get_decl_prelude(&mut self) -> Result<Option<DeclarationMetadata>, ParsingError> {
		let mut ret = None;
		loop {
			if let Some(k) = self.get_keyword(&[
				Keyword::Native, Keyword::Static, Keyword::Private, Keyword::Protected,
				Keyword::Final, Keyword::Meta, Keyword::Transient, Keyword::ReadOnly,
				Keyword::Internal, Keyword::Virtual, Keyword::Override, Keyword::Abstract,
				Keyword::VarArg, Keyword::UI, Keyword::Play, Keyword::ClearScope, Keyword::VirtualScope,
				Keyword::Action, Keyword::Deprecated, Keyword::Version
			]) {
				if ret.is_none() { ret = Some(DeclarationMetadata::default()); }
				let ret = ret.as_mut().unwrap();
				match k {
					Keyword::Native       => { ret.flags |= DeclarationFlags::Native;       }
					Keyword::Static       => { ret.flags |= DeclarationFlags::Static;       }
					Keyword::Private      => { ret.flags |= DeclarationFlags::Private;      }
					Keyword::Protected    => { ret.flags |= DeclarationFlags::Protected;    }
					Keyword::Final        => { ret.flags |= DeclarationFlags::Final;        }
					Keyword::Meta         => { ret.flags |= DeclarationFlags::Meta;         }
					Keyword::Transient    => { ret.flags |= DeclarationFlags::Transient;    }
					Keyword::ReadOnly     => { ret.flags |= DeclarationFlags::ReadOnly;     }
					Keyword::Internal     => { ret.flags |= DeclarationFlags::Internal;     }
					Keyword::Virtual      => { ret.flags |= DeclarationFlags::Virtual;      }
					Keyword::Override     => { ret.flags |= DeclarationFlags::Override;     }
					Keyword::Abstract     => { ret.flags |= DeclarationFlags::Abstract;     }
					Keyword::VarArg       => { ret.flags |= DeclarationFlags::VarArg;       }
					Keyword::UI           => { ret.flags |= DeclarationFlags::UI;           }
					Keyword::Play         => { ret.flags |= DeclarationFlags::Play;         }
					Keyword::ClearScope   => { ret.flags |= DeclarationFlags::ClearScope;   }
					Keyword::VirtualScope => { ret.flags |= DeclarationFlags::VirtualScope; }

					Keyword::Action => {
						let ex = self.get_punc(&[Punctuation::LeftRound]);
						self.expect(ex, "`(`")?;

						let types = self.get_ident_list()?;

						let ex = self.get_punc(&[Punctuation::RightRound]);
						self.expect(ex, "`,` or `)`")?;

						ret.action = Some(types);
					}
					Keyword::Deprecated => {
						let ex = self.get_punc(&[Punctuation::LeftRound]);
						self.expect(ex, "`(`")?;

						let ex = self.get_string();
						let version = self.expect(ex, "a string constant")?;

						let message = if self.get_punc(&[Punctuation::Comma]).is_some() {
							let ex = self.get_string();
							Some(self.expect(ex, "a string constant")?)
						} else { None };

						let ex = self.get_punc(&[Punctuation::RightRound]);
						self.expect(ex, if message.is_some() { "`)`" } else { "`,` or `)`" })?;
						
						ret.deprecated = Some(Deprecated { version, message });
					}
					Keyword::Version => {
						let ex = self.get_punc(&[Punctuation::LeftRound]);
						self.expect(ex, "`(`")?;

						let ex = self.get_string();
						let version = self.expect(ex, "a string constant")?;

						let ex = self.get_punc(&[Punctuation::RightRound]);
						self.expect(ex, "`)`")?;

						ret.version = Some(version);
					}

					_ => unreachable!()
				}
				continue;
			}

			if self.get_specific_ident(&["latent"]).is_some() {
				if ret.is_none() { ret = Some(DeclarationMetadata::default()); }
				ret.as_mut().unwrap().flags |= DeclarationFlags::Latent;
				continue;
			}

			break;
		}
		Ok(ret)
	}

	fn get_declaration(&mut self) -> Result<Option<Declaration>, ParsingError> {
		let metadata = self.get_decl_prelude()?;

		let ex = self.get_types_or_void()?;
		let types = if let Some(t) = ex { t } else {
			return if metadata.is_none() {
				Ok(None)
			} else {
				Err(self.expect(ex, "a declaration flag or a type").unwrap_err())
			};
		};
		let metadata = metadata.unwrap_or_else(DeclarationMetadata::default);

		let ex = self.get_ident();
		let name = self.expect(ex, "an identifier")?;

		if self.get_punc(&[Punctuation::LeftRound]).is_some() {
			let params = self.get_func_params()?;

			let ex = self.get_punc(&[Punctuation::RightRound]);
			self.expect(
				ex,
				match &params {
					FuncParams::List { args, .. } => {
						if args.len() == 0 {
							"`)`, `...`, `void` or a type"
						} else if args[args.len() - 1].init.is_none() {
							"`)`, `,` or `=`"
						} else {
							"`)` or `,`"
						}
					}
					FuncParams::Void => "`)`"
				}
			)?;

			let constant = self.get_keyword(&[Keyword::Const]).is_some();

			let body = if let Some(c) = self.get_compound_statement()? { Some(c) } else {
				let ex = self.get_punc(&[Punctuation::Semicolon]);
				self.expect(ex, "`;` or `{`")?;
				None
			};

			return Ok(Some(Declaration::Function(FunctionDeclaration {
				name, constant, metadata, return_types: types, params, body
			})));
		}

		let mut var_names = vec1![name];
		while self.get_punc(&[Punctuation::Comma]).is_some() {
			let ex = self.get_ident();
			let name = self.expect(ex, "an identifier")?;

			var_names.push(name);
		}

		let ex = self.get_punc(&[Punctuation::Semicolon]);
		self.expect(ex, if var_names.len() == 1 { "`;` or `(`" } else { "`;`" })?;

		Ok(Some(Declaration::Member(MemberDeclaration {
			var_names, metadata, member_type: types
		})))
	}

	fn get_class_inner(&mut self) -> Result<Option<ClassInner>, ParsingError> {
		if let Some(e) = self.get_enum()? {
			Ok(Some(ClassInner::Enum(e)))
		} else if let Some(s) = self.get_struct()? {
			Ok(Some(ClassInner::Struct(s)))
		} else if let Some(c) = self.get_const_def()? {
			Ok(Some(ClassInner::Const(c)))
		} else if let Some(f) = self.get_flag_def()? {
			Ok(Some(ClassInner::Flag(f)))
		} else if let Some(p) = self.get_property_def()? {
			Ok(Some(ClassInner::Property(p)))
		} else if let Some(d) = self.get_default_def()? {
			Ok(Some(ClassInner::Default(d)))
		} else if let Some(s) = self.get_states_def()? {
			Ok(Some(ClassInner::States(s)))
		} else if self.get_keyword(&[Keyword::Mixin]).is_some() {
			let ex = self.get_ident();
			let i = self.expect(ex, "an identifier")?;

			let ex = self.get_punc(&[Punctuation::Semicolon]);
			self.expect(ex, "`;` or `,`")?;

			Ok(Some(ClassInner::Mixin(i)))
		} else if self.get_keyword(&[Keyword::Static]).is_some() {
			let ex = self.get_static_const_body()?;
			if let Some(s) = ex {
				Ok(Some(ClassInner::StaticConstArray(s)))
			} else {
				let ex = self.get_declaration()?;
				let mut ret = self.expect(ex, "a declaration flag or a type")?;
				match &mut ret {
					Declaration::Member(r)   => { r.metadata.flags |= DeclarationFlags::Static; }
					Declaration::Function(r) => { r.metadata.flags |= DeclarationFlags::Static; }
				}
				Ok(Some(ClassInner::Declaration(ret)))
			}
		} else if let Some(d) = self.get_declaration()? {
			Ok(Some(ClassInner::Declaration(d)))
		} else {
			Ok(None)
		}
	}

	fn get_class_ancestry(&mut self) -> Result<Option<DottableId>, ParsingError> {
		if self.get_punc(&[Punctuation::Colon]).is_none() { return Ok(None); }

		let ex = self.get_dottable_id()?;
		let ancestor = self.expect(ex, "an identifier")?;

		Ok(Some(ancestor))
	}

	fn get_class_metadata(&mut self) -> Result<ClassMetadata, ParsingError> {
		let mut ret = ClassMetadata::default();

		while let Some(k) = self.get_keyword(&[
			Keyword::Abstract, Keyword::Native, Keyword::UI,
			Keyword::Play, Keyword::Replaces, Keyword::Version
		]) {
			match k {
				Keyword::Abstract => { ret.flags |= ClassFlags::Abstract }
				Keyword::Native   => { ret.flags |= ClassFlags::Native   }
				Keyword::UI       => { ret.flags |= ClassFlags::UI       }
				Keyword::Play     => { ret.flags |= ClassFlags::Play     }
				Keyword::Replaces => {
					let ex = self.get_dottable_id()?;
					let replacee = self.expect(ex, "an identifier")?;

					ret.replaces = Some(replacee);
				}
				Keyword::Version  => {
					let ex = self.get_punc(&[Punctuation::LeftRound]);
					self.expect(ex, "`(`")?;

					let ex = self.get_string();
					let version = self.expect(ex, "a string constant")?;

					let ex = self.get_punc(&[Punctuation::RightRound]);
					self.expect(ex, "`)`")?;

					ret.version = Some(version);
				}
				_ => unreachable!()
			}
		}

		Ok(ret)
	}

	fn get_class_body(&mut self) -> Result<Vec<ClassInner>, ParsingError> {
		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, "`{`")?;

		let mut inners = vec![];

		Ok(loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() {
				break inners;
			}

			let ex = self.get_class_inner()?;
			let inner = self.expect(ex, "a class inner element")?;
			inners.push(inner);
		})
	}

	fn get_class(&mut self) -> Result<Option<ClassDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Class]).is_none() { return Ok(None); }

		let ex = self.get_ident();
		let class_name = self.expect(ex, "an identifier")?;

		let ancestor = self.get_class_ancestry()?;
		let metadata = self.get_class_metadata()?;

		let inners = self.get_class_body()?;

		Ok(Some(ClassDefinition {
			class_name,
			ancestor,
			metadata,
			inners
		}))
	}

	fn get_mixin_class_inner(&mut self) -> Result<Option<MixinClassInner>, ParsingError> {
		if let Some(e) = self.get_enum()? {
			Ok(Some(MixinClassInner::Enum(e)))
		} else if let Some(s) = self.get_struct()? {
			Ok(Some(MixinClassInner::Struct(s)))
		} else if let Some(c) = self.get_const_def()? {
			Ok(Some(MixinClassInner::Const(c)))
		} else if let Some(f) = self.get_flag_def()? {
			Ok(Some(MixinClassInner::Flag(f)))
		} else if let Some(p) = self.get_property_def()? {
			Ok(Some(MixinClassInner::Property(p)))
		} else if let Some(d) = self.get_default_def()? {
			Ok(Some(MixinClassInner::Default(d)))
		} else if let Some(s) = self.get_states_def()? {
			Ok(Some(MixinClassInner::States(s)))
		} else if self.get_keyword(&[Keyword::Static]).is_some() {
			let ex = self.get_static_const_body()?;
			if let Some(s) = ex {
				Ok(Some(MixinClassInner::StaticConstArray(s)))
			} else {
				let ex = self.get_declaration()?;
				let mut ret = self.expect(ex, "a declaration flag or a type")?;
				match &mut ret {
					Declaration::Member(r)   => { r.metadata.flags |= DeclarationFlags::Static; }
					Declaration::Function(r) => { r.metadata.flags |= DeclarationFlags::Static; }
				}
				Ok(Some(MixinClassInner::Declaration(ret)))
			}
		} else if let Some(d) = self.get_declaration()? {
			Ok(Some(MixinClassInner::Declaration(d)))
		} else {
			Ok(None)
		}
	}

	fn get_mixin_class_body(&mut self) -> Result<Vec<MixinClassInner>, ParsingError> {
		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, "`{`")?;

		let mut inners = vec![];

		Ok(loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() {
				break inners;
			}

			let ex = self.get_mixin_class_inner()?;
			let inner = self.expect(ex, "a mixin class inner element")?;
			inners.push(inner);
		})
	}

	fn get_mixin(&mut self) -> Result<Option<MixinClassDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Mixin]).is_none() { return Ok(None); }

		let ex = self.get_keyword(&[Keyword::Class]);
		self.expect(ex, "`class`")?;

		let ex = self.get_ident();
		let mixin_name = self.expect(ex, "an identifier")?;

		let inners = self.get_mixin_class_body()?;

		Ok(Some(MixinClassDefinition {
			mixin_name,
			inners
		}))
	}

	fn get_struct_inner(&mut self) -> Result<Option<StructInner>, ParsingError> {
		if let Some(e) = self.get_enum()? {
			Ok(Some(StructInner::Enum(e)))
		} else if let Some(c) = self.get_const_def()? {
			Ok(Some(StructInner::Const(c)))
		} else if self.get_keyword(&[Keyword::Static]).is_some() {
			let ex = self.get_static_const_body()?;
			if let Some(s) = ex {
				Ok(Some(StructInner::StaticConstArray(s)))
			} else {
				let ex = self.get_declaration()?;
				let mut ret = self.expect(ex, "a declaration flag or a type")?;
				match &mut ret {
					Declaration::Member(r)   => { r.metadata.flags |= DeclarationFlags::Static; }
					Declaration::Function(r) => { r.metadata.flags |= DeclarationFlags::Static; }
				}
				Ok(Some(StructInner::Declaration(ret)))
			}
		} else if let Some(d) = self.get_declaration()? {
			Ok(Some(StructInner::Declaration(d)))
		} else {
			Ok(None)
		}
	}

	fn get_struct_metadata(&mut self) -> Result<StructMetadata, ParsingError> {
		let mut ret = StructMetadata::default();

		while let Some(k) = self.get_keyword(&[
			Keyword::Native, Keyword::UI, Keyword::ClearScope,
			Keyword::Play, Keyword::Version
		]) {
			match k {
				Keyword::Native     => { ret.flags |= StructFlags::Native     }
				Keyword::UI         => { ret.flags |= StructFlags::UI         }
				Keyword::Play       => { ret.flags |= StructFlags::Play       }
				Keyword::ClearScope => { ret.flags |= StructFlags::ClearScope }
				Keyword::Version  => {
					let ex = self.get_punc(&[Punctuation::LeftRound]);
					self.expect(ex, "`(`")?;

					let ex = self.get_string();
					let version = self.expect(ex, "a string constant")?;

					let ex = self.get_punc(&[Punctuation::RightRound]);
					self.expect(ex, "`)`")?;

					ret.version = Some(version);
				}
				_ => unreachable!()
			}
		}

		Ok(ret)
	}

	fn get_struct_body(&mut self) -> Result<Vec<StructInner>, ParsingError> {
		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, "`{`")?;

		let mut inners = vec![];

		Ok(loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() {
				self.get_punc(&[Punctuation::Semicolon]);

				break inners;
			}

			let ex = self.get_struct_inner()?;
			let inner = self.expect(ex, "a class inner element")?;
			inners.push(inner);
		})
	}

	fn get_struct(&mut self) -> Result<Option<StructDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Struct]).is_none() { return Ok(None); }

		let ex = self.get_ident();
		let struct_name = self.expect(ex, "an identifier")?;

		let metadata = self.get_struct_metadata()?;

		let inners = self.get_struct_body()?;

		Ok(Some(StructDefinition {
			struct_name,
			metadata,
			inners
		}))
	}

	fn get_extend(&mut self) -> Result<Option<Extend>, ParsingError> {
		if self.get_keyword(&[Keyword::Extend]).is_none() { return Ok(None); }

		let ex = self.get_keyword(&[Keyword::Class, Keyword::Struct]);
		let k = self.expect(ex, "`class` or `struct`")?;

		match k {
			Keyword::Class => {
				let ex = self.get_ident();
				let class_name = self.expect(ex, "an identifier")?;

				let inners = self.get_class_body()?;

				Ok(Some(Extend::Class(ExtendClass {
					class_name, inners
				})))
			}
			Keyword::Struct => {
				let ex = self.get_ident();
				let struct_name = self.expect(ex, "an identifier")?;

				let inners = self.get_struct_body()?;

				Ok(Some(Extend::Struct(ExtendStruct {
					struct_name, inners
				})))
			}
			_ => unreachable!()
		}
	}

	fn get_enum(&mut self) -> Result<Option<EnumDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Enum]).is_none() { return Ok(None); }

		let ex = self.get_ident();
		let name = self.expect(ex, "an identifier")?;

		let enum_type = if self.get_punc(&[Punctuation::Colon]).is_some() {
			let ex = self.get_specific_ident(&["sbyte", "byte", "short", "ushort", "int", "uint"]);
			let enum_type = self.expect(ex, "`sbyte`, `byte`, `short`, `ushort`, `int` or `uint`")?;

			let s = enum_type.0;
			Some(
				     if s == self.name_interner.interned("sbyte")  { IntType::SByte  }
				else if s == self.name_interner.interned("byte")   { IntType::Byte   }
				else if s == self.name_interner.interned("short")  { IntType::Short  }
				else if s == self.name_interner.interned("ushort") { IntType::UShort }
				else if s == self.name_interner.interned("int")    { IntType::Int    }
				else if s == self.name_interner.interned("uint")   { IntType::UInt   }
				else { unreachable!() }
			)
		} else { None };

		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, if enum_type.is_some() { "`{`" } else { "`{` or `:`" })?;

		let mut variants = vec![];
		loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() { break; }

			if variants.len() > 0 {
				let ex = self.get_punc(&[Punctuation::Comma]);
				self.expect(ex, "`,` or `}`")?;
				if self.get_punc(&[Punctuation::RightCurly]).is_some() { break; }
			}

			let ex = self.get_ident();
			let variant_name = self.expect(
				ex, if variants.len() == 0 { "`}` or an identifier" } else { "`}`, `,` or an identifier" }
			)?;

			let expr = if self.get_punc(&[Punctuation::Assign]).is_some() {
				let ex = self.get_expr()?;
				let expr = self.expect(ex, "an expression")?;
				Some(expr)
			} else { None };

			variants.push((variant_name, expr));
		}

		self.get_punc(&[Punctuation::Semicolon]);

		Ok(Some(EnumDefinition {
			name, enum_type, variants
		}))
	}

	fn get_const_def(&mut self) -> Result<Option<ConstDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Const]).is_none() { return Ok(None); }

		let ex = self.get_ident();
		let name = self.expect(ex, "an identifier")?;

		let ex = self.get_punc(&[Punctuation::Assign]);
		self.expect(ex, "`=`")?;

		let ex = self.get_expr()?;
		let expr = self.expect(ex, "an expression")?;

		let ex = self.get_punc(&[Punctuation::Semicolon]);
		self.expect(ex, "`;`")?;

		Ok(Some(ConstDefinition {
			name, expr
		}))
	}

	fn get_flag_def(&mut self) -> Result<Option<FlagDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::FlagDef]).is_none() { return Ok(None); }

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
		self.expect(ex, "`;`")?;

		Ok(Some(FlagDefinition {
			flag_name, var_name, shift
		}))
	}

	fn get_property_def(&mut self) -> Result<Option<PropertyDefinition>, ParsingError> {
		if self.get_specific_ident(&["property"]).is_none() { return Ok(None); }

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
		self.expect(ex, "`;` or `,`")?;

		Ok(Some(PropertyDefinition {
			name, vars
		}))
	}

	fn get_default_def(&mut self) -> Result<Option<DefaultDefinition>, ParsingError> {
		if self.get_keyword(&[Keyword::Default]).is_none() { return Ok(None); }
		
		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, "`{`")?;

		let mut ret = vec![];

		loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() {
				break;
			}
			if self.get_punc(&[Punctuation::Semicolon]).is_some() {
				continue;
			}
			if let Some(prop) = self.get_dottable_id()? {
				let vals = self.get_expr_list()?;

				let ex = self.get_punc(&[Punctuation::Semicolon]);
				self.expect(ex, "`;`")?;

				ret.push(DefaultStatement::Property { prop, vals });
				
				continue;
			}
			if let Some(p) = self.get_punc(&[Punctuation::Plus, Punctuation::Minus]) {
				let ex = self.get_dottable_id()?;
				let id = self.expect(ex, "an identifier")?;

				ret.push(
					match p {
						Punctuation::Plus  => DefaultStatement::AddFlag(id),
						Punctuation::Minus => DefaultStatement::RemoveFlag(id),
						_ => unreachable!()
					}
				);

				continue;
			}

			return Err(self.expect::<()>(None, "`+`, `-`, `;` or an identifier").unwrap_err());
		}

		Ok(Some(DefaultDefinition(ret)))
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
		if self.get_keyword(&[Keyword::States]).is_none() { return Ok(None); }

		let opts = if self.get_punc(&[Punctuation::LeftRound]).is_some() {
			let opts = self.get_ident_list()?;

			let ex = self.get_punc(&[Punctuation::RightRound]);
			self.expect(ex, "`)`")?;

			Some(opts)
		} else { None };

		let ex = self.get_punc(&[Punctuation::LeftCurly]);
		self.expect(ex, if opts.is_some() { "`{`" } else { "`{` or `(`" })?;

		self.tokenizer.set_states_mode(true);

		let mut body = vec![];

		loop {
			if self.get_punc(&[Punctuation::RightCurly]).is_some() {
				self.tokenizer.set_states_mode(false);
				break;
			}

			if let Some(start) = self.get_nws() {
				if start.0 == self.name_interner.interned("stop") {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;
					body.push(StatesBodyItem::Stop);
					continue;
				}
				if start.0 == self.name_interner.interned("wait") {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;
					body.push(StatesBodyItem::Wait);
					continue;
				}
				if start.0 == self.name_interner.interned("fail") {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;
					body.push(StatesBodyItem::Fail);
					continue;
				}
				if start.0 == self.name_interner.interned("loop") {
					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, "`;`")?;
					body.push(StatesBodyItem::Loop);
					continue;
				}
				if start.0 == self.name_interner.interned("goto") {
					self.tokenizer.set_states_mode(false);
					let target = if self.get_keyword(&[Keyword::Super]).is_some() {
						let ex = self.get_punc(&[Punctuation::DoubleColon]);
						self.expect(ex, "`::`")?;

						let ex = self.get_dottable_id()?;
						let target = self.expect(ex, "an identifier")?;

						StateGotoTarget::Super(target)
					} else {
						let ex = self.get_dottable_id()?;
						let target = self.expect(ex, "`super` or an identifier")?;

						if target.0.len() == 1 && self.get_punc(&[Punctuation::DoubleColon]).is_some() {
							let scope = target.0[0];

							let ex = self.get_dottable_id()?;
							let target = self.expect(ex, "an identifier")?;

							StateGotoTarget::Scoped(scope, target)
						} else {
							StateGotoTarget::Unscoped(target)
						}
					};

					let offset = if self.get_punc(&[Punctuation::Plus]).is_some() {
						let ex = self.get_expr()?;
						Some(self.expect(ex, "an expression")?)
					} else { None };

					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex, match &target {
						StateGotoTarget::Unscoped(t) if t.0.len() == 1 => {
							"`::`, `.`, `+` or `;`"
						}
						_ => "`.`, `+` or `;`"
					})?;
					self.tokenizer.set_states_mode(true);

					body.push(StatesBodyItem::Goto { target, offset });
					continue;
				}
				if self.get_punc(&[Punctuation::Colon]).is_some() {
					body.push(StatesBodyItem::Label(start));
					continue;
				}

				let sprite = start;
				let ex = self.get_nws();
				let frames = self.expect(ex, "`:` or non-whitespace")?;

				self.tokenizer.set_states_mode(false);

				let ex = self.get_expr()?;
				let duration = self.expect(ex, "an expression")?;

				let mut metadata = StateLineMetadata::default();
				while let Some(id) = self.get_specific_ident(
					&["bright", "fast", "slow", "nodelay", "canraise", "offset", "light"]
				) {
					let s = id.0;

						 if s == self.name_interner.interned("bright")   { metadata.flags |= StateLineFlags::Bright;   }
					else if s == self.name_interner.interned("fast")     { metadata.flags |= StateLineFlags::Fast;     }
					else if s == self.name_interner.interned("slow")     { metadata.flags |= StateLineFlags::Slow;     }
					else if s == self.name_interner.interned("nodelay")  { metadata.flags |= StateLineFlags::NoDelay;  }
					else if s == self.name_interner.interned("canraise") { metadata.flags |= StateLineFlags::CanRaise; }
					else if s == self.name_interner.interned("offset")   {
						let ex = self.get_punc(&[Punctuation::LeftRound]);
						self.expect(ex, "`(`")?;

						let ex = self.get_expr()?;
						let expr0 = self.expect(ex, "an expression")?;

						let ex = self.get_punc(&[Punctuation::Comma]);
						self.expect(ex, "`,`")?;

						let ex = self.get_expr()?;
						let expr1 = self.expect(ex, "an expression")?;

						let ex = self.get_punc(&[Punctuation::RightRound]);
						self.expect(ex, "`)`")?;

						metadata.offset = Some((expr0, expr1));
					}
					else if s == self.name_interner.interned("light")    {
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
						self.expect(ex, "`,` or `)`")?;

						metadata.light = Some(list);
					}
				}
				
				let action = if let Some(c) = self.get_compound_statement()? {
					Some(StateLineAction::Anonymous(c))
				} else {
					let action = if let Some(func) = self.get_ident() {
						let args = if self.get_punc(&[Punctuation::LeftRound]).is_some() {
							Some(self.get_function_call_args()?)
						} else { None };

						Some(StateLineAction::Call { func, args })
					} else { None };

					let ex = self.get_punc(&[Punctuation::Semicolon]);
					self.expect(ex,
						match action {
							Some(StateLineAction::Call { args: Some(_), .. }) => "`;`",
							Some(StateLineAction::Call { args: None,    .. }) => "`(` or `;`",
							_ => "a state metadata element, `;` or an identifier"
						}
					)?;

					action
				};

				self.tokenizer.set_states_mode(true);

				body.push(StatesBodyItem::Line(StateLine {
					sprite, frames, metadata, duration, action
				}));
				
				continue;
			}

			return Err(self.expect::<()>(None, "`stop`, `wait`, `fail`, `loop`, `goto` or non-whitespace").unwrap_err());
		}
		
		Ok(Some(StatesDefinition {
			opts: None, body
		}))
	}

	pub fn parse(mut self) -> ParserResult {
		let version = match self.get_lump_version() {
			Ok(v) => v,
			Err(e) => {
				self.errs.push(e);
				return ParserResult {
					ast: None,
					errs: self.errs,
					name_interner: self.name_interner,
					string_interner: self.string_interner
				};
			}
		};

		let mut definitions = vec![];

		loop {
			macro_rules! err_ret {
				($e: expr) => {
					{
						self.errs.push($e);
						ParserResult {
							ast: None,
							errs: self.errs,
							name_interner: self.name_interner,
							string_interner: self.string_interner
						}
					}
				}
			}

			if self.tokenizer.peek(&mut self.errs).is_none() {
				self.tokenizer.next(&mut self.errs);
				break;
			}

			if let Some(Token { data: TokenData::Include, .. }) = self.tokenizer.peek(&mut self.errs) {
				self.tokenizer.next(&mut self.errs);

				let ex = self.get_string();
				let path = match self.expect(ex, "a string constant") {
					Ok(p) => p,
					Err(e) => { return err_ret!(e); }
				};

				definitions.push(TopLevelDefinition::Include(path));
				continue;
			}
			match self.get_class() {
				Ok(Some(c)) => {
					definitions.push(TopLevelDefinition::Class(c));
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}
			match self.get_mixin() {
				Ok(Some(c)) => {
					definitions.push(TopLevelDefinition::MixinClass(c));
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}
			match self.get_struct() {
				Ok(Some(s)) => {
					definitions.push(TopLevelDefinition::Struct(s));
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}
			match self.get_extend() {
				Ok(Some(e)) => {
					match e {
						Extend::Class(c)  => { definitions.push(TopLevelDefinition::ExtendClass(c));  }
						Extend::Struct(s) => { definitions.push(TopLevelDefinition::ExtendStruct(s)); }
					}
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}
			match self.get_enum() {
				Ok(Some(e)) => {
					definitions.push(TopLevelDefinition::Enum(e));
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}
			match self.get_const_def() {
				Ok(Some(c)) => {
					definitions.push(TopLevelDefinition::Const(c));
					continue;
				}
				Ok(None) => {}
				Err(e) => { return err_ret!(e); }
			}

			let e = self.expect::<()>(None, "a top level element").unwrap_err();
			self.errs.push(e);
			return ParserResult {
				ast: None,
				errs: self.errs,
				name_interner: self.name_interner,
				string_interner: self.string_interner
			};
		}

		ParserResult {
			ast: Some(TopLevel {
				version,
				definitions
			}),
			errs: self.errs,
			name_interner: self.name_interner,
			string_interner: self.string_interner
		}
	}
}
