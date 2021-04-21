use std::borrow::Cow;
use regex::Regex;
use lazy_static::lazy_static;

use super::{ParsingError, Span};

macro_rules! define_token_rule {
	($reg: expr => $s: ident, $c: ident, $action: expr) => {
		|text, remaining| {
			lazy_static! {
				static ref RE: Regex = Regex::new(concat!("^(", $reg, ")")).unwrap();
			}
			let $c = RE.captures(remaining)?;
			let $s = $c.get(0).unwrap().as_str();
			let a: Result<TokenData, String> = $action;
			let ret = match a {
				Ok(data) => Ok(Token {
					original: &remaining[..$c.get(0).unwrap().end()],
					data
				}),
				Err(msg) => Err(ParsingError {
					msg: String::from(msg),
					spans: vec1::vec1![span(text, remaining)]
				})
			};
			Some((
				ret,
				&remaining[$c.get(0).unwrap().end()..]
			))
		}
	}
}

macro_rules! define_token_rule_func {
	($rule: ident: $reg: literal => $s: ident, $c: ident, $action: expr) => {
		fn $rule<'src>(text: &'src str, remaining: &'src str) -> Option<(Result<Token<'src>, ParsingError>, &'src str)> {
			define_token_rule!($reg => $s, $c, $action)(text, remaining)
		}
	};
}

macro_rules! first_of {
	($rules: expr) => {
		|text, remaining| {
			for r in $rules {
				if let Some(t) = r(text, remaining) {
					return Some(t);
				}
			}
			None
		}
	}
}

macro_rules! rule_grouping_type {
	($name: ident, $reg_func: ident, $match_func: ident {
		$( $rule: literal => $res: ident ),*
	}) => {
		#[derive(Debug, PartialEq)]
		pub enum $name {
			$(
				$res,
			)*
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				match *self {
					$(
						Self::$res => write!(f, "{}", $rule),
					)*
				}
			}
		}

		#[allow(unused)]
		fn $reg_func<'src>(text: &'src str, remaining: &'src str) -> Option<(Result<Token<'src>, ParsingError>, &'src str)> {
			first_of!(&[
				$(
					|text, remaining| {
						lazy_static! {
							static ref RE: Regex = {
								let mut rule = "^(".to_string();
								rule.push_str(&regex::escape($rule));
								rule.push_str(")");
								Regex::new(&rule).unwrap()
							};
						}
						let m = RE.find(remaining)?;
						let data = TokenData::$name($name::$res);

						let original = &remaining[..m.end()];
						let new = &remaining[m.end()..];

						Some((
							Ok(Token {
								original, data
							}),
							new
						))
					},
				)*
			])(text, remaining)
		}

		#[allow(unused)]
		fn $match_func<'src>(remaining: &'src str) -> Option<$name> {
			match &*remaining.to_lowercase() {
				$(
					$rule => Some($name::$res),
				)*
				_ => None
			}
		}
	};
}

#[derive(Debug, PartialEq)]
pub struct Token<'src> {
	pub original: &'src str,
	pub data: TokenData<'src>
}

fn span(text: &str, original: &str) -> Span {
	let text_ptr = text.as_ptr() as usize;
	let self_ptr = original.as_ptr() as usize;
	let start = self_ptr - text_ptr;
	let end = start + original.len();
	Span { start, end }
}

impl<'src> Token<'src> {
	pub fn span(&self, text: &'src str) -> Span {
		span(text, &self.original)
	}
}

#[derive(Debug, PartialEq)]
pub enum TokenData<'src> {
	Identifier(&'src str),
	Name(&'src str),
	String(Cow<'src, str>),
	Int { val: u64, long: bool, unsigned: bool },
	Float { val: f64, double: bool },
	Punctuation(Punctuation),
	Keyword(Keyword),
	Include,

	NonWhitespace(Cow<'src, str>),
}

impl<'src> std::fmt::Display for TokenData<'src> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Identifier(_)    => write!(f, "an identifier"),
			Self::Name(_)          => write!(f, "a name literal"),
			Self::String(_)        => write!(f, "a string literal"),
			Self::Int { .. }       => write!(f, "an integer constant"),
			Self::Float { .. }     => write!(f, "a float constant"),
			Self::Punctuation(p)   => write!(f, "`{}`", p),
			Self::Keyword(k)       => write!(f, "`{}`", k),
			Self::Include          => write!(f, "#include"),

			Self::NonWhitespace(_) => write!(f, "non-whitespace")
		}
	}
}

rule_grouping_type!(Punctuation, punc, match_punc {
	r">>>=" => UnsignedRightShiftAssign,

	r">>>"  => UnsignedRightShift,
	r">>="  => RightShiftAssign,
	r"<<="  => LeftShiftAssign,
	r"~=="  => ApproxEquals,
	r"<>="  => ThreeWayComp,
	r"..."  => Ellipsis,

	r".."   => DotDot,
	r"+="   => PlusAssign,
	r"-="   => MinusAssign,
	r"*="   => TimesAssign,
	r"/="   => DivideAssign,
	r"%="   => ModuloAssign,
	r"&="   => BitwiseAndAssign,
	r"^="   => BitwiseXorAssign,
	r"|="   => BitwiseOrAssign,
	r">>"   => RightShift,
	r"<<"   => LeftShift,
	r"=="   => Equals,
	r"!="   => NotEquals,
	r">="   => GreaterThanEquals,
	r"<="   => LessThanEquals,
	r"&&"   => LogicalAnd,
	r"||"   => LogicalOr,
	r"++"   => Increment,
	r"--"   => Decrement,
	r"::"   => DoubleColon,
	r"->"   => Arrow,
	r"**"   => Raise,

	r"+"    => Plus,
	r"-"    => Minus,
	r"*"    => Times,
	r"/"    => Divide,
	r"%"    => Modulo,
	r"&"    => BitwiseAnd,
	r"^"    => BitwiseXor,
	r"|"    => BitwiseOr,
	r"~"    => BitwiseNot,
	r"!"    => LogicalNot,
	r"("    => LeftRound,
	r")"    => RightRound,
	r"["    => LeftSquare,
	r"]"    => RightSquare,
	r"{"    => LeftCurly,
	r"}"    => RightCurly,
	r"<"    => LeftAngle,
	r">"    => RightAngle,
	r"="    => Assign,
	r":"    => Colon,
	r";"    => Semicolon,
	r"."    => Dot,
	r","    => Comma,
	r"?"    => QuestionMark,
	r"#"    => Hash,
	r"@"    => AtSign
});

fn states_punc<'src>(text: &'src str, remaining: &'src str) -> Option<(Result<Token<'src>, ParsingError>, &'src str)> {
	first_of!(&[
		define_token_rule!(r":"  => _s, c, Ok(TokenData::Punctuation(Punctuation::Colon))),
		define_token_rule!(r";"  => _s, c, Ok(TokenData::Punctuation(Punctuation::Semicolon))),
		define_token_rule!(r"\}" => _s, c, Ok(TokenData::Punctuation(Punctuation::RightCurly))),
	])(text, remaining)
}

rule_grouping_type!(Keyword, keyword, match_keyword {
	r#"break"#        => Break,
	r#"case"#         => Case,
	r#"const"#        => Const,
	r#"continue"#     => Continue,
	r#"default"#      => Default,
	r#"do"#           => Do,
	r#"else"#         => Else,
	r#"for"#          => For,
	r#"goto"#         => Goto,
	r#"if"#           => If,
	r#"return"#       => Return,
	r#"switch"#       => Switch,
	r#"until"#        => Until,
	r#"volatile"#     => Volatile,
	r#"while"#        => While,

	r#"char"#         => Char,
	r#"long"#         => Long,
	r#"ulong"#        => ULong,
	r#"struct"#       => Struct,
	r#"class"#        => Class,
	r#"mixin"#        => Mixin,
	r#"enum"#         => Enum,
	r#"in"#           => In,
	r#"sizeof"#       => SizeOf,
	r#"alignof"#      => AlignOf,

	r#"abstract"#     => Abstract,
	r#"foreach"#      => ForEach,
	r#"true"#         => True,
	r#"false"#        => False,
	r#"auto"#         => Auto,
	r#"flagdef"#      => FlagDef,
	r#"native"#       => Native,
	r#"var"#          => Var,
	r#"out"#          => Out,
	r#"static"#       => Static,
	r#"transient"#    => Transient,
	r#"final"#        => Final,
	r#"extend"#       => Extend,
	r#"protected"#    => Protected,
	r#"private"#      => Private,
	r#"dot"#          => Dot,
	r#"cross"#        => Cross,
	r#"virtual"#      => Virtual,
	r#"override"#     => Override,
	r#"vararg"#       => VarArg,
	r#"ui"#           => UI,
	r#"play"#         => Play,
	r#"clearscope"#   => ClearScope,
	r#"virtualscope"# => VirtualScope,
	r#"super"#        => Super,
	r#"stop"#         => Stop,
	r#"null"#         => Null,

	r#"is"#           => Is,
	r#"replaces"#     => Replaces,
	r#"states"#       => States,
	r#"meta"#         => Meta,
	r#"deprecated"#   => Deprecated,
	r#"version"#      => Version,
	r#"action"#       => Action,
	r#"readonly"#     => ReadOnly,
	r#"internal"#     => Internal,
	r#"let"#          => Let
});

define_token_rule_func!(ident_or_keyword: r"[_a-zA-Z][_a-zA-Z0-9]*" => s, c, {
	if let Some(k) = match_keyword(s) {
		Ok(TokenData::Keyword(k))
	} else {
		Ok(TokenData::Identifier(s))
	}
});

fn unescape<'src>(s: &'src str) -> Cow<'src, str> {
	if s.contains("\\") {
		let s = s.replace(r#"\""#, "\"");
		let s = s.replace(r"\\", "\\");
		let s = s.replace("\\\n", "\n");
		let s = s.replace(r"\a", "\x07");
		let s = s.replace(r"\b", "\x08");
		let s = s.replace(r"\c", "\x1C");
		let s = s.replace(r"\f", "\x0C");
		let s = s.replace(r"\n", "\n");
		let s = s.replace(r"\t", "\t");
		let s = s.replace(r"\r", "\r");
		let s = s.replace(r"\v", "\x0B");
		let s = s.replace(r"\?", "?");
		Cow::Owned(s)
	} else {
		Cow::Borrowed(s)
	}
}

define_token_rule_func!(
	name: r"'[^\n']*(?P<EndQuote>')?" => s, c, {
		if c.name("EndQuote").is_some() {
			Ok(TokenData::Name(&s[1..s.len() - 1]))
		} else {
			Err("unterminated name literal".to_string())
		}
	}
);

define_token_rule_func!(
	string: r#""(\\"|[^"])*(?P<EndQuote>")?"# => s, c, {
		if c.name("EndQuote").is_some() {
			Ok(TokenData::String(unescape(&s[1..s.len() - 1])))
		} else {
			Err("unterminated string literal".to_string())
		}
	}
);

define_token_rule_func!(
	int: r#"(?x)
		(
			(?P<Oct>0(?P<OctBody>[0-9]+))|
			(?P<Hex>(0[xX])(?P<HexBody>[0-9a-fA-F]+))|
			(?P<Dec>(?P<DecBody>[0-9]+))
		)
		(?P<Suff>[uUlL]{0,2})
	"# => _s, c, {
		let val = if c.name("Oct").is_some() {
			u64::from_str_radix(c.name("OctBody").unwrap().as_str(), 8).unwrap()
		} else if c.name("Hex").is_some() {
			u64::from_str_radix(c.name("HexBody").unwrap().as_str(), 16).unwrap()
		} else {
			u64::from_str_radix(c.name("DecBody").unwrap().as_str(), 10).unwrap()
		};
		let long = c.name("Suff").map(|x| x.as_str().contains("l") | x.as_str().contains("L")).unwrap_or(false);
		let unsigned = c.name("Suff").map(|x| x.as_str().contains("u") | x.as_str().contains("U")).unwrap_or(false);
		Ok(TokenData::Int { val, long, unsigned })
	}
);

define_token_rule_func!(
	float: r#"(?x)
		(?P<Body>
			([0-9]+([eE][+-]?[0-9]+))          |
			([0-9]*\.[0-9]+([eE][+-]?[0-9]+)?) |
			([0-9]+\.[0-9]*([eE][+-]?[0-9]+)?)
		)
		(?P<Suff>[fF])?
	"# => _s, c, {
		let val = c.name("Body").unwrap().as_str().parse::<f64>().unwrap();
		let double = !c.name("Suff").is_some();
		Ok(TokenData::Float { val, double })
	}
);

define_token_rule_func!(
	include: r#"#include"# => _s, c, {
		Ok(TokenData::Include)
	}
);

define_token_rule_func!(
	nws: r#"[^\u0000- /":;]*(/[^\u0000- /":;*][^\u0000- /":;]*)*"# => s, c, {
		Ok(TokenData::NonWhitespace(Cow::from(s)))
	}
);


fn skip_whitespace<'src>(remaining: &'src str) -> &'src str {
	lazy_static! {
		static ref RE: Regex = Regex::new(
			r"^((#region.*\n)|(#endregion.*\n)|([\u0000- ]+)|(//.*\n)|(?s:/\*.*\*/))"
		).unwrap();
	}
	let mut cur = remaining;
	while let Some(s) = RE.shortest_match(cur) {
		cur = &cur[s..]
	}
	cur
}

#[derive(Debug)]
pub struct Tokenizer<'src> {
	text: &'src str,
	remaining: &'src str,
	states_mode: bool,
	peeked: Option<Option<Token<'src>>>
}

impl<'src> Tokenizer<'src> {
	pub fn new(text: &'src str) -> Self {
		Self { text, remaining: text, states_mode: false, peeked: None }
	}

	pub fn set_states_mode(&mut self, mode: bool) {
		if self.peeked.is_some() { panic!("set_states_mode called while in a peeked state"); }
		self.states_mode = mode;
	}

	fn next_internal(&mut self) -> Option<Result<Token<'src>, ParsingError>> {
		self.remaining = skip_whitespace(self.remaining);
		if self.remaining.is_empty() { return None; }

		if self.states_mode {
			let result = first_of!(
				&[
					string, states_punc, nws
				]
			)(self.text, self.remaining);
			if let Some((token, new_remaining)) = result {
				self.remaining = new_remaining;
				Some(
					if let Ok(Token { original, data: TokenData::String(s) }) = token {
						Ok(Token {
							original,
							data: TokenData::NonWhitespace(s)
						})
					} else { token }
				)
			} else {
				dbg!(self.remaining);
				unimplemented!()
			}
		} else {
			let result = first_of!(
				&[include, ident_or_keyword, name, string, float, int, punc]
			)(self.text, self.remaining);
			if let Some((token, new_remaining)) = result {
				self.remaining = new_remaining;
				Some(token)
			} else {
				dbg!(self.remaining);
				unimplemented!()
			}
		}
	}

	pub fn next(&mut self, errs: &mut Vec<ParsingError>) -> Option<Token<'src>> {
		if self.peeked.is_some() {
			return self.peeked.take().unwrap();
		}
		loop {
			let tok = self.next_internal();
			if let Some(Err(e)) = tok {
				errs.push(e);
				continue;
			}
			return tok.map(|x| x.unwrap());
		}
	}

	pub fn peek(&mut self, errs: &mut Vec<ParsingError>) -> &Option<Token<'src>> {
		if let None = self.peeked {
			self.peeked = Some(self.next(errs));
		}

		match self.peeked.as_ref() {
			Some(p) => &p,
			None => unreachable!()
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_ident() {
		let s = r"_something5 hi";
		let (tok, rest) = ident_or_keyword(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: "_something5",
				data: TokenData::Identifier("_something5")
			}
		);
		assert_eq!(rest, " hi");
	}

	#[test]
	fn test_name() {
		let s = r"'Hello \n hi me!' crazy";
		let (tok, rest) = name(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r"'Hello \n hi me!'",
				data: TokenData::Name(r"Hello \n hi me!")
			}
		);
		assert_eq!(rest, " crazy");

		let s = r"'Hello \n hi' me!' crazy";
		let (tok, rest) = name(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r"'Hello \n hi'",
				data: TokenData::Name(r"Hello \n hi")
			}
		);
		assert_eq!(rest, " me!' crazy");

		let s = "'Hello \\n hi me!\n crazy";
		let (tok, rest) = name(s, s).unwrap();
		assert_eq!(
			tok.unwrap_err(),
			ParsingError {
				msg: "unterminated name literal".to_string(),
				spans: vec1::vec1![Span { start: 0, end: s.len() }]
			}
		);
		assert_eq!(rest, "\n crazy");
	}

	#[test]
	fn test_string() {
		let s = r#""Hello \" ' hi me!" crazy"#;
		let (tok, rest) = string(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r#""Hello \" ' hi me!""#,
				data: TokenData::String(Cow::from(r#"Hello " ' hi me!"#))
			}
		);
		assert_eq!(rest, " crazy");

		let s = concat!(r#""Hello \" "#, "\n", r#"' hi me!" crazy"#);
		let (tok, rest) = string(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: concat!(r#""Hello \" "#, "\n", r#"' hi me!""#),
				data: TokenData::String(Cow::from(concat!(r#"Hello " "#, "\n", r#"' hi me!"#)))
			}
		);
		assert_eq!(rest, " crazy");

		let s = r#""Hello \" ' hi me! crazy"#;
		let (tok, rest) = string(s, s).unwrap();
		assert_eq!(
			tok.unwrap_err(),
			ParsingError {
				msg: "unterminated string literal".to_string(),
				spans: vec1::vec1![Span { start: 0, end: s.len() }]
			}
		);
		assert_eq!(rest, "");
	}

	#[test]
	fn test_int() {
		let s = r#"0x45A2u some other stuff"#;
		let (tok, rest) = int(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r#"0x45A2u"#,
				data: TokenData::Int { val: 0x45A2, long: false, unsigned: true }
			}
		);
		assert_eq!(rest, " some other stuff");
	}

	#[test]
	fn test_float() {
		let s = r#".245e-42f some other stuff"#;
		let (tok, rest) = float(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r#".245e-42f"#,
				data: TokenData::Float { val: 0.245e-42, double: false }
			}
		);
		assert_eq!(rest, " some other stuff");
	}

	#[test]
	fn test_nws() {
		let s = r#"Test#24[/]#£2" other things"#;
		let (tok, rest) = nws(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r#"Test#24[/]#£2"#,
				data: TokenData::NonWhitespace(Cow::from(r#"Test#24[/]#£2"#))
			}
		);
		assert_eq!(rest, r#"" other things"#);
	}

	#[test]
	fn test_punc() {
		let s = r#">>= hi"#;
		let (tok, rest) = punc(s, s).unwrap();
		assert_eq!(
			tok.unwrap(),
			Token {
				original: r#">>="#,
				data: TokenData::Punctuation(Punctuation::RightShiftAssign)
			}
		);
		assert_eq!(rest, " hi");
	}

	#[test]
	fn test_tokenizer() {
		let s = r#"
			something _s "Hi!" // hello!
			/* hi
			 * end of comment */
			#region Some Stuff
			'Hello...'424l += 0x7453 0.42 @
			#endregion
		"#;
		let mut errs = vec![];
		let mut tok = Tokenizer::new(s);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: "something",
				data: TokenData::Identifier("something")
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: "_s",
				data: TokenData::Identifier("_s")
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#""Hi!""#,
				data: TokenData::String(Cow::from("Hi!"))
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"'Hello...'"#,
				data: TokenData::Name("Hello...")
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"424l"#,
				data: TokenData::Int { val: 424, long: true, unsigned: false } 
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"+="#,
				data: TokenData::Punctuation(Punctuation::PlusAssign)
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"0x7453"#,
				data: TokenData::Int { val: 0x7453, long: false, unsigned: false } 
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"0.42"#,
				data: TokenData::Float { val: 0.42, double: true } 
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"@"#,
				data: TokenData::Punctuation(Punctuation::AtSign)
			}
		);
		assert!(tok.next(&mut errs).is_none());
	}

	#[test]
	fn test_tokenizer_states() {
		let s = r#"HI#24 AA##A ; "Hello\t...""#;
		let mut tok = Tokenizer::new(s);
		tok.set_states_mode(true);
		let mut errs = vec![];
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"HI#24"#,
				data: TokenData::NonWhitespace(Cow::from("HI#24"))
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#"AA##A"#,
				data: TokenData::NonWhitespace(Cow::from("AA##A"))
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#";"#,
				data: TokenData::Punctuation(Punctuation::Semicolon)
			}
		);
		assert_eq!(
			tok.next(&mut errs).unwrap(),
			Token {
				original: r#""Hello\t...""#,
				data: TokenData::NonWhitespace(Cow::from("Hello\t..."))
			}
		);
		assert!(tok.next(&mut errs).is_none());
	}
}
