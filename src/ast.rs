use super::interner::{NameSymbol, StringSymbol};

use flagset::{FlagSet, flags};
use vec1::Vec1;

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinition {
	Class(ClassDefinition),
	Struct(StructDefinition),
	ExtendClass(ExtendClass),
	ExtendStruct(ExtendStruct),
	MixinClass(MixinClassDefinition),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	Include(StringSymbol)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevel {
	pub version: VersionInfo,
	pub definitions: Vec<TopLevelDefinition>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Identifier(pub NameSymbol);
#[derive(Debug, Clone, PartialEq)]
pub struct DottableId(pub Vec1<Identifier>);

#[derive(Debug, Clone, PartialEq)]
pub struct IntConst {
	pub val: u64,
	pub long: bool,
	pub unsigned: bool
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatConst {
	pub val: f64,
	pub double: bool
}

#[derive(Debug, Clone, PartialEq)]
pub enum Const {
	String(StringSymbol),
	Name(NameSymbol),
	Int(IntConst),
	Float(FloatConst),
	Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	SingleUserType(Identifier),
	DottedUserType(DottableId),
	NativeType(Identifier),
	ReadonlyType(Identifier),
	ReadonlyNativeType(Identifier),
	Class(Option<DottableId>),
	Map(Box<(TypeOrArray, TypeOrArray)>),
	DynArray(Box<TypeOrArray>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeOrArray {
	Type(Type),
	Array(Type, ExprList)
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

flags! {
	pub enum ClassFlags: u32 {
		Abstract,
		Native,
		UI,
		Play
	}
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ClassMetadata {
	pub flags: FlagSet<ClassFlags>,
	pub version: Option<StringSymbol>,
	pub replaces: Option<DottableId>
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassInner {
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
pub struct ClassDefinition {
	pub class_name: Identifier,
	pub ancestor: Option<DottableId>,
	pub metadata: ClassMetadata,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendClass {
	pub class_name: Identifier,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixinClassInner {
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
pub struct MixinClassDefinition {
	pub mixin_name: Identifier,
	pub inners: Vec<MixinClassInner>,
}

flags! {
	pub enum StructFlags: u32 {
		ClearScope,
		Native,
		UI,
		Play
	}
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructMetadata {
	pub flags: FlagSet<StructFlags>,
	pub version: Option<StringSymbol>
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructInner {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	StaticConstArray(StaticConstArray),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
	pub struct_name: Identifier,
	pub metadata: StructMetadata,
	pub inners: Vec<StructInner>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendStruct {
	pub struct_name: Identifier,
	pub inners: Vec<StructInner>,
}

flags! {
	pub enum DeclarationFlags: u32 {
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
		VirtualScope
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deprecated {
	pub version: StringSymbol,
	pub message: Option<StringSymbol>
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DeclarationMetadata {
	pub flags: FlagSet<DeclarationFlags>,
	pub deprecated: Option<Deprecated>,
	pub version: Option<StringSymbol>,
	pub action: Option<Vec1<Identifier>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
	pub metadata: DeclarationMetadata,
	pub member_type: TypeListOrVoid,
	pub var_names: Vec1<Identifier>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
	Function(FunctionDeclaration),
	Member(MemberDeclaration)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoid {
	TypeList(Vec<TypeOrArray>),
	Void
}

flags! {
	pub enum ParamFlags: u32 {
		In,
		Out,
		Optional
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncParam {
	pub flags: FlagSet<ParamFlags>,
	pub param_type: Type,
	pub name: Identifier,
	pub init: Option<Expression>
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncParams {
	Void,
	List {
		args: Vec<FuncParam>,
		variadic: bool
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
	pub name: Identifier,
	pub constant: bool,
	pub metadata: DeclarationMetadata,
	pub return_types: TypeListOrVoid,
	pub params: FuncParams,
	pub body: Option<CompoundStatement>
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticConstArray {
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
pub enum FunctionCallArg {
	Unnamed(Expression),
	Named(Identifier, Expression)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
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
	ClassCast(Identifier, Vec<FunctionCallArg>)
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
pub struct ExprList(pub Vec1<Expression>);

#[derive(Debug, Clone, PartialEq)]
pub enum VarInit {
	Single { name: Identifier, val: Option<Expression> },
	Array {
		name: Identifier,
		sizes: Option<ExprList>,
		vals: Option<ExprList>
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
	pub var_type: Type,
	pub inits: Vec1<VarInit>
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
	VarDef(LocalVariableDefinition),
	ExprList(ExprList)
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundStatement(pub Vec<Statement>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
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
pub enum IntType {
	SByte,
	Byte,
	Short,
	UShort,
	Int,
	UInt
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDefinition {
	pub name: Identifier,
	pub enum_type: Option<IntType>,
	pub variants: Vec<(Identifier, Option<Expression>)>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDefinition {
	pub name: Identifier,
	pub expr: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlagDefinition {
	pub flag_name: Identifier,
	pub var_name: Identifier,
	pub shift: IntConst
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDefinition {
	pub name: Identifier,
	pub vars: Vec1<Identifier>
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultStatement {
	Property { prop: DottableId, vals: Option<ExprList> },
	AddFlag(DottableId),
	RemoveFlag(DottableId)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDefinition(pub Vec<DefaultStatement>);

#[derive(Debug, Clone, PartialEq)]
pub struct NonWhitespace(pub NameSymbol);

#[derive(Debug, Clone, PartialEq)]
pub enum StateLineAction {
	Call {
		func: Identifier,
		args: Option<Vec<FunctionCallArg>>
	},
	Anonymous(CompoundStatement)
}

flags! {
	pub enum StateLineFlags: u32 {
		Bright,
		Fast,
		Slow,
		NoDelay,
		CanRaise
	}
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StateLineMetadata {
	pub flags: FlagSet<StateLineFlags>,
	pub offset: Option<(Expression, Expression)>,
	pub light: Option<Vec1<StringSymbol>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateLine {
	pub sprite: NonWhitespace,
	pub frames: NonWhitespace,
	pub duration: Expression,
	pub metadata: StateLineMetadata,
	pub action: Option<StateLineAction>
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateGotoTarget {
	Unscoped(DottableId),
	Scoped(Identifier, DottableId),
	Super(DottableId)
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatesBodyItem {
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
pub struct StatesDefinition {
	pub opts: Option<Vec1<Identifier>>,
	pub body: Vec<StatesBodyItem>
}
