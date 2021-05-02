use std::collections::HashMap;

use vec1::Vec1;
use flagset::*;

use super::interner::{NameSymbol, StringSymbol};
use crate::ir_common::*;
use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
	Class(ClassDefinition),
	Struct(StructDefinition),
	Enum(EnumDefinition),
	Const(ConstDefinition),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
	pub span: Span,
	pub kind: TopLevelDefinitionKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevel {
	pub version: Option<VersionInfo>,
	pub definitions: HashMap<NameSymbol, TopLevelDefinition>
}

flags! {
	pub enum ClassDefinitionFlags: u8 {
		Abstract,
		Native,
		UI,
		Play
	}
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
	pub flags: FlagSet<ClassDefinitionFlags>,
	pub version: VersionInfo,
	pub replaces: Option<DottableId>,
	pub inners: HashMap<NameSymbol, ClassInner>,
}

flags! {
	pub enum StructDefinitionFlags: u8 {
		Abstract,
		Native,
		UI,
		Play
	}
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
	pub flags: FlagSet<StructDefinitionFlags>,
	pub version: VersionInfo,
	pub inners: HashMap<NameSymbol, StructInner>,
}

flags! {
	pub enum FunctionFlags: u8 {
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

		Const
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deprecated {
	version: VersionInfo, message: Option<StringConst>
}

flags! {
	pub enum ActionFlags: u8 {
		Actor,
		Overlay,
		Weapon,
		Item
	}
}

flags! {
	pub enum FuncParamFlags: u8 {
		In,
		Out,
		Optional
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncParam {
	pub span: Span,
	pub flags: FlagSet<FuncParamFlags>,
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
	pub flags: FlagSet<FunctionFlags>,
	pub deprecated: Option<Deprecated>,
	pub version: VersionInfo,
	pub action: Option<FlagSet<ActionFlags>>,
	pub return_types: TypeListOrVoid,
	pub params: FuncParams,
	pub body: Option<CompoundStatement>
}

flags! {
	pub enum MemberFlags: u8 {
		Native,
		Private,
		Protected,
		Transient,
		ReadOnly,
		Internal,
		VarArg,
		UI,
		Play,

		// only allowed in classes, not structs
		Meta,

		// only allowed within the base archive
		ClearScope
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub flags: FlagSet<MemberFlags>,
	pub deprecated: Option<Deprecated>,
	pub version: VersionInfo,
	pub member_type: TypeOrArray,
	pub vars: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
	Function(FunctionDeclaration),
	Member(MemberDeclaration)
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
	If { cond: Expression, body: CompoundStatement, else_body: Option<CompoundStatement> },
	Switch { val: Expression, body: CompoundStatement },
	Loop(CompoundStatement),
	Break,
	Continue,
	Return(Option<ExprList>),
	LocalVariableDefinition(LocalVariableDefinition),
	MultiAssign { assignees: ExprList, rhs: Expression },
	StaticConstArray(StaticConstArray)
}
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
	pub span: Span,
	pub kind: StatementKind
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
	pub span: Span,
	pub var_type: Type,
	pub init: VarInit
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

flags! {
	pub enum StateLineFlags: u8 {
		Bright,
		Fast,
		Slow,
		NoDelay,
		CanRaise,
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateLine {
	pub span: Span,
	pub sprite: NonWhitespace,
	pub frames: NonWhitespace,
	pub duration: Expression,
	pub flags: FlagSet<StateLineFlags>,
	pub offset: Option<(Expression, Expression)>,
	pub light: Option<Vec1<StringConst>>,
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
