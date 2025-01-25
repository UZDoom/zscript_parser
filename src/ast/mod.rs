use vec1::Vec1;

use crate::interner::StringSymbol;
use crate::ir_common::*;
use crate::Span;

#[cfg(feature = "serialize")]
use serde::Serialize;

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
    Class(ClassDefinition),
    Struct(StructDefinition),
    ExtendClass(ExtendClass),
    ExtendStruct(ExtendStruct),
    MixinClass(MixinClassDefinition),
    Enum(EnumDefinition),
    Const(ConstDefinition),
    Include(StringConst),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TopLevelDefinitionKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevel {
    pub version: Option<VersionInfo>,
    pub definitions: Vec<TopLevelDefinition>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassMetadataItemKind {
    Abstract,
    Native,
    UI,
    Play,
    Version(StringConst),
    Replaces(DottableId),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ClassMetadataItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: ClassMetadataItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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
    Mixin(Identifier),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ClassInner {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: ClassInnerKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub ancestor: Option<DottableId>,
    pub metadata: Vec<ClassMetadataItem>,
    pub inners: Vec<ClassInner>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendClass {
    pub span: Span,
    pub name: Identifier,
    pub inners: Vec<ClassInner>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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
    StaticConstArray(StaticConstArray),
}

impl MixinClassInnerKind {
    pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind {
        match self {
            MixinClassInnerKind::Declaration(d) => ClassInnerKind::Declaration(d),
            MixinClassInnerKind::Enum(e) => ClassInnerKind::Enum(e),
            MixinClassInnerKind::Struct(s) => ClassInnerKind::Struct(s),
            MixinClassInnerKind::States(s) => ClassInnerKind::States(s),
            MixinClassInnerKind::Default(d) => ClassInnerKind::Default(d),
            MixinClassInnerKind::Const(c) => ClassInnerKind::Const(c),
            MixinClassInnerKind::Property(p) => ClassInnerKind::Property(p),
            MixinClassInnerKind::Flag(f) => ClassInnerKind::Flag(f),
            MixinClassInnerKind::StaticConstArray(sca) => ClassInnerKind::StaticConstArray(sca),
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MixinClassInner {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: MixinClassInnerKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MixinClassDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub inners: Vec<MixinClassInner>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StructMetadataItemKind {
    ClearScope,
    Native,
    UI,
    Play,
    Version(StringConst),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StructMetadataItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StructMetadataItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StructInnerKind {
    Declaration(Declaration),
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub metadata: Vec<StructMetadataItem>,
    pub inners: Vec<StructInner>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendStruct {
    pub span: Span,
    pub name: Identifier,
    pub inners: Vec<StructInner>,
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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
    Deprecated {
        version: StringConst,
        message: Option<StringConst>,
    },
    Version(StringConst),
    Action(Option<Vec1<Identifier>>),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationMetadataItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: DeclarationMetadataItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum FuncPtrFlag {
    Ui,
    Play,
    ClearScope,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncPtrParam {
    pub span: Span,
    pub flags: Vec<ParamFlagItem>,
    pub ty: Type,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum FuncPtrParamsKind {
    Void,
    List(Vec<FuncPtrParam>),
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncPtrParams {
    pub span: Span,
    pub kind: FuncPtrParamsKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncPtrDetailed {
    pub flag: FuncPtrFlag,
    pub return_types: TypeListOrVoid,
    pub params: FuncPtrParams,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum FuncPtrKind {
    Void,
    Detailed(FuncPtrDetailed),
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    SingleUserType(Identifier),
    DottedUserType(DottableId),
    NativeType(Identifier),
    ReadonlyType(Identifier),
    ReadonlyNativeType(Identifier),
    Class(Option<DottableId>),
    Map(Box<(TypeOrArray, TypeOrArray)>),
    MapIterator(Box<(TypeOrArray, TypeOrArray)>),
    FuncPtr(FuncPtrKind),
    DynArray(Box<TypeOrArray>),
    Let,

    Error,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TypeKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeOrArrayKind {
    Type(Type),
    Array(Type, ArraySizes),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeOrArray {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TypeOrArrayKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoidKind {
    TypeList(Vec1<TypeOrArray>),
    Void,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeListOrVoid {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: TypeListOrVoidKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum ParamFlagItemKind {
    In,
    Out,
    Optional,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ParamFlagItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: ParamFlagItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncParam {
    pub span: Span,
    pub flags: Vec<ParamFlagItem>,
    pub param_type: Type,
    pub name: Identifier,
    pub init: Option<Expression>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum FuncParamsKind {
    Void,
    List {
        args: Vec<FuncParam>,
        variadic: bool,
    },
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncParams {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: FuncParamsKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub name: Identifier,
    pub constant: bool,
    pub metadata: Vec<DeclarationMetadataItem>,
    pub return_types: TypeListOrVoid,
    pub params: FuncParams,
    pub body: Option<CompoundStatement>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
    pub doc_comment: Option<StringSymbol>,
    pub span: Span,
    pub metadata: Vec<DeclarationMetadataItem>,
    pub member_type: TypeListOrVoid,
    pub vars: Vec1<(Identifier, Option<ArraySizes>)>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Function(FunctionDeclaration),
    Member(MemberDeclaration),
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum CondIterType {
    While,
    Until,
    DoWhile,
    DoUntil,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum VarInitKind {
    Single {
        name: Identifier,
        val: Option<Expression>,
    },
    Multi {
        names: Vec1<Identifier>,
        val: Expression,
    },
    Array {
        name: Identifier,
        sizes: Option<ArraySizes>,
        vals: Option<ExprList>,
    },
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct VarInit {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: VarInitKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
    pub span: Span,
    pub var_type: Type,
    pub inits: Vec1<VarInit>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum ForInitKind {
    VarDef(LocalVariableDefinition),
    ExprList(ExprList),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ForInit {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: ForInitKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundStatement {
    pub span: Span,
    pub statements: Vec<Statement>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Labeled(LabeledStatement),
    Compound(CompoundStatement),
    Expression(Expression),
    If {
        cond: Expression,
        body: Box<Statement>,
        else_body: Option<Box<Statement>>,
    },
    Switch {
        val: Expression,
        body: Box<Statement>,
    },
    CondIter {
        cond: Expression,
        body: Box<Statement>,
        iter_type: CondIterType,
    },
    For {
        init: Option<ForInit>,
        cond: Option<Expression>,
        update: Option<ExprList>,
        body: Box<Statement>,
    },
    Break,
    Continue,
    Return(Option<ExprList>),
    LocalVariableDefinition(LocalVariableDefinition),
    MultiAssign {
        assignees: ExprList,
        rhs: Expression,
    },
    StaticConstArray(StaticConstArray),
    Empty,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StatementKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
#[derive(Debug, Clone, PartialEq)]
pub enum StateLineMetadataItemKind {
    Bright,
    Fast,
    Slow,
    NoDelay,
    CanRaise,
    Offset(Expression, Expression),
    Light(Vec1<StringConst>),
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StateLineMetadataItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StateLineMetadataItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StateLine {
    pub span: Span,
    pub sprite: NonWhitespace,
    pub frames: NonWhitespace,
    pub duration: Expression,
    pub metadata: Vec<StateLineMetadataItem>,
    pub action: Option<StateLineAction>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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
#[cfg_attr(feature = "serialize", serde(tag = "kind", content = "def"))]
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
        offset: Option<Expression>,
    },
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StatesBodyItem {
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub kind: StatesBodyItemKind,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StatesDefinition {
    pub span: Span,
    pub opts: Option<Vec1<Identifier>>,
    pub body: Vec<StatesBodyItem>,
}
