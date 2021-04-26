use crate::Span;
use super::interner::{NameSymbol, StringSymbol};

use vec1::Vec1;

pub(crate) mod print;

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
	Class(ClassDefinition),
	Struct(StructDefinition),
	ExtendClass(ExtendClass),
	ExtendStruct(ExtendStruct),
	MixinClass(MixinClassDefinition),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	Include(StringConst)
}
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
	pub span: Span,
	pub kind: TopLevelDefinitionKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevel {
	pub version: Option<VersionInfo>,
	pub definitions: Vec<TopLevelDefinition>
}

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
pub struct VersionInfo {
	major: u16,
	minor: u16,
	revision: u16
}

impl VersionInfo {
	pub fn new(major: u16, minor: u16, revision: u16) -> Self {
		Self {
			major, minor, revision
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMetadataItemKind {
	Abstract,
	Native,
	UI,
	Play,
	Version(StringConst),
	Replaces(DottableId),
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClassMetadataItem {
	pub span: Span,
	pub kind: ClassMetadataItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Struct(StructDefinition),
	States(StatesDefinition),
	Default(DefaultDefinition),
	Const(ConstDefinition),
	Property(PropertyDefinition),
	Flag(FlagDefinition),
	StaticConstArray(StaticConstArray),
	Mixin(Identifier)
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClassInner {
	pub span: Span,
	pub kind: ClassInnerKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub ancestor: Option<DottableId>,
	pub metadata: Vec<ClassMetadataItem>,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendClass {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixinClassInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Struct(StructDefinition),
	States(StatesDefinition),
	Default(DefaultDefinition),
	Const(ConstDefinition),
	Property(PropertyDefinition),
	Flag(FlagDefinition),
	StaticConstArray(StaticConstArray)
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixinClassInner {
	pub span: Span,
	pub kind: MixinClassInnerKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixinClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<MixinClassInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructMetadataItemKind {
	ClearScope,
	Native,
	UI,
	Play,
	Version(StringConst)
}
#[derive(Debug, Clone, PartialEq)]
pub struct StructMetadataItem {
	pub span: Span,
	pub kind: StructMetadataItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	StaticConstArray(StaticConstArray),
}
#[derive(Debug, Clone, PartialEq)]
pub struct StructInner {
	pub span: Span,
	pub kind: StructInnerKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub metadata: Vec<StructMetadataItem>,
	pub inners: Vec<StructInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendStruct {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<StructInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationMetadataItemKind {
	Native,
	Static,
	Private,
	Protected,
	Latent,
	Final,
	Meta,
	Transient,
	ReadOnly,
	Internal,
	Virtual,
	Override,
	Abstract,
	VarArg,
	UI,
	Play,
	ClearScope,
	VirtualScope,
	Deprecated { version: StringConst, message: Option<StringConst> },
	Version(StringConst),
	Action(Option<Vec1<Identifier>>)
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationMetadataItem {
	pub span: Span,
	pub kind: DeclarationMetadataItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub metadata: Vec<DeclarationMetadataItem>,
	pub member_type: TypeListOrVoid,
	pub vars: Vec1<(Identifier, Option<ExprList>)>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
	Function(FunctionDeclaration),
	Member(MemberDeclaration)
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
pub enum ParamFlagItemKind {
	In,
	Out,
	Optional
}
#[derive(Debug, Clone, PartialEq)]
pub struct ParamFlagItem {
	pub span: Span,
	pub kind: ParamFlagItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncParam {
	pub span: Span,
	pub flags: Vec<ParamFlagItem>,
	pub param_type: Type,
	pub name: Identifier,
	pub init: Option<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncParamsKind {
	Void,
	List {
		args: Vec<FuncParam>,
		variadic: bool
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct FuncParams {
	pub span: Span,
	pub kind: FuncParamsKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub constant: bool,
	pub metadata: Vec<DeclarationMetadataItem>,
	pub return_types: TypeListOrVoid,
	pub params: FuncParams,
	pub body: Option<CompoundStatement>
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
pub enum CondIterType {
	While,
	Until,
	DoWhile,
	DoUntil
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprList {
	pub span: Span,
	pub list: Vec1<Expression>
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

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
	pub span: Span,
	pub var_type: Type,
	pub inits: Vec1<VarInit>
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInitKind {
	VarDef(LocalVariableDefinition),
	ExprList(ExprList)
}
#[derive(Debug, Clone, PartialEq)]
pub struct ForInit {
	pub span: Span,
	pub kind: ForInitKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundStatement {
	pub span: Span,
	pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
	Labeled(LabeledStatement),
	Compound(CompoundStatement),
	Expression(Expression),
	If { cond: Expression, body: Box<Statement>, else_body: Option<Box<Statement>> },
	Switch { val: Expression, body: Box<Statement> },
	CondIter { cond: Expression, body: Box<Statement>, iter_type: CondIterType },
	For {
		init: Option<ForInit>,
		cond: Option<Expression>,
		update: Option<ExprList>,
		body: Box<Statement>
	},
	Break,
	Continue,
	Return(Option<ExprList>),
	LocalVariableDefinition(LocalVariableDefinition),
	MultiAssign { assignees: ExprList, rhs: Expression },
	StaticConstArray(StaticConstArray),
	Empty
}
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
	pub span: Span,
	pub kind: StatementKind
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
pub enum StateLineActionKind {
	Call {
		func: Identifier,
		args: Option<Vec<FunctionCallArg>>
	},
	Anonymous(CompoundStatement)
}
#[derive(Debug, Clone, PartialEq)]
pub struct StateLineAction {
	pub span: Span,
	pub kind: StateLineActionKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateLineMetadataItemKind {
	Bright,
	Fast,
	Slow,
	NoDelay,
	CanRaise,
	Offset(Expression, Expression),
	Light(Vec1<StringConst>)
}
#[derive(Debug, Clone, PartialEq)]
pub struct StateLineMetadataItem {
	pub span: Span,
	pub kind: StateLineMetadataItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateLine {
	pub span: Span,
	pub sprite: NonWhitespace,
	pub frames: NonWhitespace,
	pub duration: Expression,
	pub metadata: Vec<StateLineMetadataItem>,
	pub action: Option<StateLineAction>
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateGotoTargetKind {
	Unscoped(DottableId),
	Scoped(Identifier, DottableId),
	Super(DottableId)
}
#[derive(Debug, Clone, PartialEq)]
pub struct StateGotoTarget {
	pub span: Span,
	pub kind: StateGotoTargetKind
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatesBodyItemKind {
	Label(NonWhitespace),
	Line(StateLine),
	Stop,
	Wait,
	Fail,
	Loop,
	Goto {
		target: StateGotoTarget,
		offset: Option<Expression>
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct StatesBodyItem {
	pub span: Span,
	pub kind: StatesBodyItemKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatesDefinition {
	pub span: Span,
	pub opts: Option<Vec1<Identifier>>,
	pub body: Vec<StatesBodyItem>
}
