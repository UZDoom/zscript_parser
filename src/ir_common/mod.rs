use vec1::Vec1;

use crate::Span;
use crate::interner::{NameSymbol, StringSymbol};

pub(crate) mod print;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Identifier {
	pub span: Span,
	pub symbol: NameSymbol
}
#[derive(Debug, Clone, PartialEq)]
pub struct DottableId {
	pub span: Span,
	pub ids: Vec1<Identifier>
}
#[derive(Debug, Clone, PartialEq)]
pub struct StringConst {
	pub span: Span,
	pub symbol: StringSymbol
}
#[derive(Debug, Clone, PartialEq)]
pub struct NameConst {
	pub span: Span,
	pub symbol: NameSymbol
}
#[derive(Debug, Clone, PartialEq)]
pub struct IntConst {
	pub span: Span,
	pub val: u64,
	pub long: bool,
	pub unsigned: bool
}
#[derive(Debug, Clone, PartialEq)]
pub struct FloatConst {
	pub span: Span,
	pub val: f64,
	pub double: bool
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstKind {
	String(StringConst),
	Name(NameConst),
	Int(IntConst),
	Float(FloatConst),
	Bool(bool),
	Null
}
#[derive(Debug, Clone, PartialEq)]
pub struct Const {
	pub span: Span,
	pub kind: ConstKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
	SingleUserType(Identifier),
	DottedUserType(DottableId),
	NativeType(Identifier),
	ReadonlyType(Identifier),
	ReadonlyNativeType(Identifier),
	Class(Option<DottableId>),
	Map(Box<(TypeOrArray, TypeOrArray)>),
	DynArray(Box<TypeOrArray>),
	Let
}
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
	pub span: Span,
	pub kind: TypeKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeOrArrayKind {
	Type(Type),
	Array(Type, ExprList)
}
#[derive(Debug, Clone, PartialEq)]
pub struct TypeOrArray {
	pub span: Span,
	pub kind: TypeOrArrayKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoidKind {
	TypeList(Vec1<TypeOrArray>),
	Void
}
#[derive(Debug, Clone, PartialEq)]
pub struct TypeListOrVoid {
	pub span: Span,
	pub kind: TypeListOrVoidKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionInfo {
	pub major: u16,
	pub minor: u16,
	pub revision: u16
}

impl VersionInfo {
	pub fn new(major: u16, minor: u16, revision: u16) -> Self {
		Self {
			major, minor, revision
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
	Ident(Identifier),
	Const(Const),
	Binary { op: BinaryOp, exprs: Box<BinaryOpExprs> },
	Prefix { op: PrefixOp, expr: Box<Expression> },
	Postfix { op: PostfixOp, expr: Box<Expression> },
	Ternary(Box<TernaryOpExprs>),
	ArrayIndex(Box<ArrayIndexExprs>),
	FunctionCall { lhs: Box<Expression>, exprs: Vec<FunctionCallArg> },
	Vector2(Box<(Expression, Expression)>),
	Vector3(Box<(Expression, Expression, Expression)>),
	ClassCast(Identifier, Vec<FunctionCallArg>),
	Super
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
	pub span: Span,
	pub kind: ExpressionKind
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
	Add,
	Subtract,
	Times,
	Divide,
	Modulo,
	Raise,

	LeftShift,
	RightShift,
	UnsignedRightShift,

	CrossProd,
	DotProd,
	Concat,

	LessThan,
	LessThanEquals,
	GreaterThan,
	GreaterThanEquals,
	Equals,
	NotEquals,
	ApproxEquals,
	ThreeWayComp,

	LogicalAnd,
	BitwiseAnd,
	LogicalOr,
	BitwiseOr,
	BitwiseXor,

	Is,
	Scope,
	MemberAccess,

	Assign,
	PlusAssign,
	MinusAssign,
	TimesAssign,
	DivideAssign,
	ModuloAssign,
	LeftShiftAssign,
	RightShiftAssign,
	UnsignedRightShiftAssign,
	BitwiseOrAssign,
	BitwiseAndAssign,
	BitwiseXorAssign
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefixOp {
	Plus,
	Minus,
	Increment,
	Decrement,
	LogicalNot,
	BitwiseNot,
	SizeOf,
	AlignOf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostfixOp {
	Increment,
	Decrement
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabeledStatement {
	Default,
	Case(Box<Expression>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprList {
	pub span: Span,
	pub list: Vec1<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOpExprs {
	pub lhs: Expression,
	pub rhs: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct TernaryOpExprs {
	pub cond: Expression,
	pub if_true: Expression,
	pub if_false: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayIndexExprs {
	pub lhs: Expression,
	pub index: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallArgKind {
	Unnamed(Expression),
	Named(Identifier, Expression)
}
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallArg {
	pub span: Span,
	pub kind: FunctionCallArgKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntTypeKind {
	SByte,
	Byte,
	Short,
	UShort,
	Int,
	UInt
}
#[derive(Debug, Clone, PartialEq)]
pub struct IntType {
	pub span: Span,
	pub kind: IntTypeKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub init: Option<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub enum_type: Option<IntType>,
	pub variants: Vec<EnumVariant>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub expr: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlagDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub flag_name: Identifier,
	pub var_name: Identifier,
	pub shift: IntConst
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub vars: Vec1<Identifier>
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticConstArray {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub arr_type: Type,
	pub name: Identifier,
	pub exprs: ExprList
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultStatementKind {
	Property { prop: DottableId, vals: Option<ExprList> },
	AddFlag(DottableId),
	RemoveFlag(DottableId)
}
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultStatement {
	pub span: Span,
	pub kind: DefaultStatementKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDefinition {
	pub span: Span,
	pub statements: Vec<DefaultStatement>
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonWhitespace {
	pub span: Span,
	pub symbol: NameSymbol
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarInitKind {
	Single { name: Identifier, val: Option<Expression> },
	Array {
		name: Identifier,
		sizes: Option<ExprList>,
		vals: Option<ExprList>
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct VarInit {
	pub span: Span,
	pub kind: VarInitKind
}
