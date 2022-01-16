use std::collections::HashMap;

use flagset::*;
use vec1::Vec1;

use super::interner::{NameSymbol, StringSymbol};
use crate::ir_common::*;
use crate::Span;

#[cfg(feature = "serialize")]
use serde::Serialize;

pub mod lower;

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
    Class(ClassDefinition),
    Struct(StructDefinition),
    MixinClass(MixinClassDefinition),
    Enum(EnumDefinition),
    Const(ConstDefinition),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
    pub span: Span,
    pub archive_num: usize,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TopLevelDefinitionKind,
}

impl TopLevelDefinition {
    pub(crate) fn name(&self) -> &Identifier {
        match &self.kind {
            TopLevelDefinitionKind::Class(c) => &c.name,
            TopLevelDefinitionKind::Struct(s) => &s.name,
            TopLevelDefinitionKind::MixinClass(m) => &m.name,
            TopLevelDefinitionKind::Enum(e) => &e.name,
            TopLevelDefinitionKind::Const(c) => &c.name,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevel {
    pub definitions: HashMap<NameSymbol, Vec1<TopLevelDefinition>>,
}

flags! {
    pub enum ClassDefinitionFlags: u8 {
        Abstract,
        Native,
        UI,
        Play
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassInnerKind {
    FunctionDeclaration(FunctionDeclaration),
    MemberDeclaration(MemberDeclaration),
    Enum(EnumDefinition),
    Struct(StructDefinition),
    Const(ConstDefinition),
    Property(PropertyDefinition),
    Flag(FlagDefinition),
    StaticConstArray(StaticConstArray),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ClassInner {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: ClassInnerKind,
}

impl ClassInner {
    pub(crate) fn name(&self) -> &Identifier {
        match &self.kind {
            ClassInnerKind::FunctionDeclaration(x) => &x.name,
            ClassInnerKind::MemberDeclaration(x) => &x.name,
            ClassInnerKind::Enum(x) => &x.name,
            ClassInnerKind::Struct(x) => &x.name,
            ClassInnerKind::Const(x) => &x.name,
            ClassInnerKind::Property(x) => &x.name,
            ClassInnerKind::StaticConstArray(x) => &x.name,
            ClassInnerKind::Flag(x) => &x.flag_name,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub ancestor: Option<DottableId>,
    pub flags: FlagSet<ClassDefinitionFlags>,
    pub states: Vec<StatesItem>,
    pub defaults: Vec<DefaultStatement>,
    pub version: Option<VersionInfo>,
    pub replaces: Option<DottableId>,
    pub inners: HashMap<NameSymbol, Vec1<ClassInner>>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MixinClassDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub states: Vec<StatesItem>,
    pub defaults: Vec<DefaultStatement>,
    pub inners: HashMap<NameSymbol, Vec1<ClassInner>>,
}

flags! {
    pub enum StructDefinitionFlags: u8 {
        ClearScope,
        Abstract,
        Native,
        UI,
        Play
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StructInnerKind {
    FunctionDeclaration(FunctionDeclaration),
    MemberDeclaration(MemberDeclaration),
    Enum(EnumDefinition),
    Const(ConstDefinition),
    StaticConstArray(StaticConstArray),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StructInner {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StructInnerKind,
}

impl StructInner {
    pub(crate) fn name(&self) -> &Identifier {
        match &self.kind {
            StructInnerKind::FunctionDeclaration(x) => &x.name,
            StructInnerKind::MemberDeclaration(x) => &x.name,
            StructInnerKind::Enum(x) => &x.name,
            StructInnerKind::Const(x) => &x.name,
            StructInnerKind::StaticConstArray(x) => &x.name,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub flags: FlagSet<StructDefinitionFlags>,
    pub version: Option<VersionInfo>,
    pub inners: HashMap<NameSymbol, Vec1<StructInner>>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StaticConstArray {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub arr_type: Type,
    pub name: Identifier,
    pub exprs: ExprList,
}

flags! {
    pub enum FunctionFlags: u8 {
        Native,
        Static,
        Private,
        Protected,
        Final,
        Transient,
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deprecated {
    version: VersionInfo,
    message: Option<StringConst>,
}

flags! {
    pub enum ActionFlags: u8 {
        Actor,
        Overlay,
        Weapon,
        Item
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    SingleUserType(Identifier),
    DottedUserType(DottableId),
    NativeType(Identifier),
    ReadonlyType(Identifier),
    ReadonlyNativeType(Identifier),
    Class(Option<DottableId>),
    Map(Box<(Type, Type)>),
    Array(Box<Type>, Option<Expression>),
    DynArray(Box<Type>),
    Let,

    Error,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoidKind {
    TypeList(Vec1<Type>),
    Void,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeListOrVoid {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TypeListOrVoidKind,
}

flags! {
    pub enum FuncParamFlags: u8 {
        In,
        Out,
        Optional
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncParam {
    pub span: Span,
    pub flags: FlagSet<FuncParamFlags>,
    pub param_type: Type,
    pub name: Identifier,
    pub init: Option<Expression>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncParams {
    pub span: Span,
    pub args: Vec<FuncParam>,
    pub variadic: bool,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
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
    pub body: Option<CompoundStatement>,
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub flags: FlagSet<MemberFlags>,
    pub deprecated: Option<Deprecated>,
    pub version: VersionInfo,
    pub member_type: Type,
    pub name: Identifier,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundStatement {
    pub span: Option<Span>,
    pub statements: Vec<Statement>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Labeled(LabeledStatement),
    Compound(CompoundStatement),
    Expression(Expression),
    If {
        cond: Expression,
        body: CompoundStatement,
        else_body: Option<CompoundStatement>,
    },
    Switch {
        val: Expression,
        body: CompoundStatement,
    },
    Loop(CompoundStatement),
    Break,
    Continue,
    Return(Option<ExprList>),
    LocalVariableDefinition(LocalVariableDefinition),
    MultiAssign {
        assignees: ExprList,
        rhs: Expression,
    },
    StaticConstArray(StaticConstArray),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub span: Option<Span>,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StatementKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum VarInit {
    Single(Expression),
    Compound(ExprList),
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
    pub span: Span,
    pub var_type: Type,
    pub name: Identifier,
    pub init: Option<VarInit>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StateLineActionKind {
    Call {
        func: Identifier,
        args: Option<Vec<FunctionCallArg>>,
    },
    Anonymous(CompoundStatement),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StateLineAction {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StateLineActionKind,
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StateLine {
    pub span: Span,
    pub sprite: NonWhitespace,
    pub frames: NonWhitespace,
    pub duration: Expression,
    pub flags: FlagSet<StateLineFlags>,
    pub action_flags: Option<FlagSet<ActionFlags>>,
    pub offset: Option<(Expression, Expression)>,
    pub light: Option<Vec1<StringConst>>,
    pub action: Option<StateLineAction>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StateGotoTargetKind {
    Unscoped(DottableId),
    Scoped(Identifier, DottableId),
    Super(DottableId),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StateGotoTarget {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StateGotoTargetKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "data"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StatesItemKind {
    Label(NonWhitespace),
    Line(Box<StateLine>),
    Stop,
    Wait,
    Fail,
    Loop,
    Goto {
        target: StateGotoTarget,
        offset: Option<Expression>,
    },
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StatesItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StatesItemKind,
}
