use ptree::TreeBuilder;

use super::*;
use crate::interner::{NameInterner, StringInterner};

impl std::fmt::Display for VersionInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
	}
}

impl std::fmt::Display for IntConst {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.val)?;
		if self.unsigned { write!(f, "u")?; }
		if self.long     { write!(f, "l")?; }
		Ok(())
	}
}

impl std::fmt::Display for FloatConst {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.val)?;
		if !self.double { write!(f, "f")?; }
		Ok(())
	}
}

type Interners<'a> = (&'a NameInterner, &'a StringInterner);

trait Repr {
	fn repr(&self, i: &Interners) -> String;
}

impl<T: Repr> Repr for Option<T> {
	fn repr(&self, i: &Interners) -> String {
		match self {
			Some(x) => format!("Some({})", x.repr(i)),
			None    => "None".to_string()
		}
	}
}

impl<T: Repr> Repr for &T {
	fn repr(&self, i: &Interners) -> String {
		(*self).repr(i)
	}
}

impl Repr for StringSymbol {
	fn repr(&self, (_, s): &Interners) -> String {
		format!("\"{}\"", s.string(*self))
	}
}

impl Repr for NameSymbol {
	fn repr(&self, (n, _): &Interners) -> String {
		format!("'{}'", n.string(*self))
	}
}

impl Repr for Identifier {
	fn repr(&self, (n, _): &Interners) -> String {
		n.string(self.0).to_string()
	}
}

impl Repr for NonWhitespace {
	fn repr(&self, (n, _): &Interners) -> String {
		format!("\"{}\"", n.string(self.0))
	}
}

impl Repr for DottableId {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.0.iter().map(|x| x.repr(i)).collect();
		v.join(".")
	}
}

impl<T: Repr> Repr for Vec<T> {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.iter().map(|x| x.repr(i)).collect();
		v.join(", ")
	}
}

impl<T: Repr> Repr for Vec1<T> {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.iter().map(|x| x.repr(i)).collect();
		v.join(", ")
	}
}

impl Repr for Const {
	fn repr(&self, i: &Interners) -> String {
		match self {
			Const::String(s) => s.repr(i),
			Const::Name(n)   => n.repr(i),
			Const::Int(i)    => format!("{}", i),
			Const::Float(f)  => format!("{}", f),
			Const::Bool(b)   => format!("{}", b)
		}
	}
}

impl Repr for Expression {
	fn repr(&self, i: &Interners) -> String {
		match self {
			Self::Ident(id) => id.repr(i),
			Self::Const(c) => c.repr(i),
			Self::Binary { op, exprs } => {
				let op_str = match op {
					BinaryOp::Add                      => " + ",
					BinaryOp::Subtract                 => " - ",
					BinaryOp::Times                    => " * ",
					BinaryOp::Divide                   => " / ",
					BinaryOp::Modulo                   => " % ",
					BinaryOp::Raise                    => " ** ",
					BinaryOp::LeftShift                => " << ",
					BinaryOp::RightShift               => " >> ",
					BinaryOp::UnsignedRightShift       => " >>> ",
					BinaryOp::CrossProd                => " cross ",
					BinaryOp::DotProd                  => " dot ",
					BinaryOp::Concat                   => " .. ",
					BinaryOp::LessThan                 => " < ",
					BinaryOp::LessThanEquals           => " <= ",
					BinaryOp::GreaterThan              => " > ",
					BinaryOp::GreaterThanEquals        => " >= ",
					BinaryOp::Equals                   => " == ",
					BinaryOp::NotEquals                => " != ",
					BinaryOp::ApproxEquals             => " ~== ",
					BinaryOp::ThreeWayComp             => " <>= ",
					BinaryOp::LogicalAnd               => " && ",
					BinaryOp::BitwiseAnd               => " & ",
					BinaryOp::LogicalOr                => " || ",
					BinaryOp::BitwiseOr                => " | ",
					BinaryOp::BitwiseXor               => " ^ ",
					BinaryOp::Is                       => " is ",
					BinaryOp::Scope                    => "::",
					BinaryOp::MemberAccess             => ".",
					BinaryOp::Assign                   => " = ",
					BinaryOp::PlusAssign               => " += ",
					BinaryOp::MinusAssign              => " -= ",
					BinaryOp::TimesAssign              => " *= ",
					BinaryOp::DivideAssign             => " /= ",
					BinaryOp::ModuloAssign             => " %= ",
					BinaryOp::LeftShiftAssign          => " <<= ",
					BinaryOp::RightShiftAssign         => " >>= ",
					BinaryOp::UnsignedRightShiftAssign => " >>>= ",
					BinaryOp::BitwiseOrAssign          => " |= ",
					BinaryOp::BitwiseAndAssign         => " &= ",
					BinaryOp::BitwiseXorAssign         => " ^= "
				};
				let exprs = exprs.as_ref();
				format!("({}{}{})", exprs.lhs.repr(i), op_str, exprs.rhs.repr(i))
			}
			Self::Prefix { op, expr } => {
				let op_str = match op {
					PrefixOp::Plus       => "+",
					PrefixOp::Minus      => "-",
					PrefixOp::Increment  => "++",
					PrefixOp::Decrement  => "--",
					PrefixOp::LogicalNot => "!",
					PrefixOp::BitwiseNot => "~",
					PrefixOp::SizeOf     => "sizeof ",
					PrefixOp::AlignOf    => "alignof "
				};
				let expr = expr.as_ref();
				format!("({}{})", op_str, expr.repr(i))
			}
			Self::Postfix { op, expr } => {
				let op_str = match op {
					PostfixOp::Increment => "++",
					PostfixOp::Decrement => "--"
				};
				let expr = expr.as_ref();
				format!("({}{})", expr.repr(i), op_str)
			}
			Self::Ternary(t) => {
				let t = t.as_ref();
				format!("({} ? {} : {})", t.cond.repr(i), t.if_true.repr(i), t.if_false.repr(i))
			}
			Self::ArrayIndex(a) => {
				let a = a.as_ref();
				format!("({}[{}])", a.lhs.repr(i), a.index.repr(i))
			}
			Self::FunctionCall { lhs, exprs } => {
				let v: Vec<String> = exprs.iter().map(|x| match x {
					FunctionCallArg::Unnamed(e) => e.repr(i),
					FunctionCallArg::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
				}).collect();
				format!("({}({}))", lhs.as_ref().repr(i), v.join(", "))
			}
			Self::Vector2(v) => {
				let (x, y) = v.as_ref();
				format!("({}, {})", x.repr(i), y.repr(i))
			}
			Self::Vector3(v) => {
				let (x, y, z) = v.as_ref();
				format!("({}, {}, {})", x.repr(i), y.repr(i), z.repr(i))
			}
			Self::ClassCast(id, exprs) => {
				let v: Vec<String> = exprs.iter().map(|x| match x {
					FunctionCallArg::Unnamed(e) => e.repr(i),
					FunctionCallArg::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
				}).collect();
				format!("((class<{}>) ({}))", id.repr(i), v.join(", "))
			}
		}
	}
}

impl Repr for Type {
	fn repr(&self, i: &Interners) -> String {
		match self {
		    Type::SingleUserType(id)     => id.repr(i),
		    Type::DottedUserType(ids)    => format!(".{}", ids.repr(i)),
		    Type::NativeType(id)         => format!("@{}", id.repr(i)),
		    Type::ReadonlyType(id)       => format!("ReadOnly<{}>", id.repr(i)),
		    Type::ReadonlyNativeType(id) => format!("ReadOnly<@{}>", id.repr(i)),
		    Type::Class(ids) => {
				format!("Class{}", if let Some(ids) = ids { format!("<{}>", ids.repr(i)) } else { "".to_string() })
			}
		    Type::Map(tys) => {
				let tys = tys.as_ref();
				format!("Map<{}, {}>", tys.0.repr(i), tys.1.repr(i))
			}
		    Type::DynArray(ty) => {
				let ty = ty.as_ref();
				format!("Array<{}>", ty.repr(i))
			}
			Type::Let => { "let".to_string() }
		}
	}
}

impl Repr for TypeOrArray {
	fn repr(&self, i: &Interners) -> String {
		match self {
		    TypeOrArray::Type(t) => t.repr(i),
		    TypeOrArray::Array(t, e) => {
				let v: Vec<String> = e.0.iter().map(|x| format!("[{}]", x.repr(i))).collect();
				format!("{}{}", t.repr(i), v.join(""))
			}
		}
	}
}

impl Repr for TypeListOrVoid {
	fn repr(&self, i: &Interners) -> String {
		match self {
		    TypeListOrVoid::TypeList(tys) => {
				let v: Vec<String> = tys.iter().map(|x| x.repr(i)).collect();
				v.join(", ")
			}
		    TypeListOrVoid::Void => "void".to_string()
		}
	}
}

impl TopLevel {
	pub fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.add_empty_child(format!("Version {}", self.version));
		for d in self.definitions.iter() {
			match d {
				TopLevelDefinition::Class(x)        => { x.tree(i, builder); }
				TopLevelDefinition::Struct(x)       => { x.tree(i, builder); }
				TopLevelDefinition::ExtendClass(x)  => { x.tree(i, builder); }
				TopLevelDefinition::ExtendStruct(x) => { x.tree(i, builder); }
				TopLevelDefinition::MixinClass(x)   => { x.tree(i, builder); }
				TopLevelDefinition::Enum(x)         => { x.tree(i, builder); }
				TopLevelDefinition::Const(x)        => { x.tree(i, builder); }
				TopLevelDefinition::Include(x)      => { builder.add_empty_child(format!("Include: {}", x.repr(i))); }
			}
		}
	}
}

impl ClassDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Class Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Ancestor: {}", self.ancestor.repr(i)));
		builder.add_empty_child(format!("Flags: {:?}", self.metadata.flags));
		builder.add_empty_child(format!("Version: {}", self.metadata.version.repr(i)));
		builder.add_empty_child(format!("Replaces: {}", self.metadata.replaces.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match inner {
				ClassInner::Declaration(x)      => { x.tree(i, builder); }
				ClassInner::Enum(x)             => { x.tree(i, builder); }
				ClassInner::Struct(x)           => { x.tree(i, builder); }
				ClassInner::States(x)           => { x.tree(i, builder); }
				ClassInner::Default(x)          => { x.tree(i, builder); }
				ClassInner::Const(x)            => { x.tree(i, builder); }
				ClassInner::Property(x)         => { x.tree(i, builder); }
				ClassInner::Flag(x)             => { x.tree(i, builder); }
				ClassInner::StaticConstArray(x) => { x.tree(i, builder); }
				ClassInner::Mixin(x)            => { builder.add_empty_child(format!("Mixin: {}", x.repr(i))); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl StructDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Struct Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Flags: {:?}", self.metadata.flags));
		builder.add_empty_child(format!("Version: {}", self.metadata.version.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match inner {
				StructInner::Declaration(x)      => { x.tree(i, builder); }
				StructInner::Enum(x)             => { x.tree(i, builder); }
				StructInner::Const(x)            => { x.tree(i, builder); }
				StructInner::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl ExtendClass {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Class Extension"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match inner {
				ClassInner::Declaration(x)      => { x.tree(i, builder); }
				ClassInner::Enum(x)             => { x.tree(i, builder); }
				ClassInner::Struct(x)           => { x.tree(i, builder); }
				ClassInner::States(x)           => { x.tree(i, builder); }
				ClassInner::Default(x)          => { x.tree(i, builder); }
				ClassInner::Const(x)            => { x.tree(i, builder); }
				ClassInner::Property(x)         => { x.tree(i, builder); }
				ClassInner::Flag(x)             => { x.tree(i, builder); }
				ClassInner::StaticConstArray(x) => { x.tree(i, builder); }
				ClassInner::Mixin(x)            => { builder.add_empty_child(format!("Mixin: {}", x.repr(i))); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl ExtendStruct {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Struct Extension"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match inner {
				StructInner::Declaration(x)      => { x.tree(i, builder); }
				StructInner::Enum(x)             => { x.tree(i, builder); }
				StructInner::Const(x)            => { x.tree(i, builder); }
				StructInner::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl MixinClassDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Mixin Class Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match inner {
				MixinClassInner::Declaration(x)      => { x.tree(i, builder); }
				MixinClassInner::Enum(x)             => { x.tree(i, builder); }
				MixinClassInner::Struct(x)           => { x.tree(i, builder); }
				MixinClassInner::States(x)           => { x.tree(i, builder); }
				MixinClassInner::Default(x)          => { x.tree(i, builder); }
				MixinClassInner::Const(x)            => { x.tree(i, builder); }
				MixinClassInner::Property(x)         => { x.tree(i, builder); }
				MixinClassInner::Flag(x)             => { x.tree(i, builder); }
				MixinClassInner::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl EnumDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Enum Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Type: {:?}", self.enum_type));
		builder.begin_child(format!("Body"));
		for (id, expr) in self.variants.iter() {
			builder.begin_child(format!("Variant: {}", id.repr(i)));
			if let Some(e) = expr {
				builder.add_empty_child(format!("Init: {}", e.repr(i)));
			}
			builder.end_child();
		}
		builder.end_child();
		builder.end_child();
	}
}

impl ConstDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Const Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Expression: {}", self.expr.repr(i)));
		builder.end_child();
	}
}

impl DefaultDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Default Definition"));
		for d in self.0.iter() {
			match d {
			    DefaultStatement::Property { prop, vals } => {
					builder.begin_child(format!("Property"));
					builder.add_empty_child(format!("Property: {}", prop.repr(i)));
					builder.add_empty_child(format!("Vals: {}", vals.as_ref().map(|x| &x.0).repr(i)));
					builder.end_child();
				}
			    DefaultStatement::AddFlag(f) => {
					builder.add_empty_child(format!("Add Flag: {}", f.repr(i)));
				}
			    DefaultStatement::RemoveFlag(f) => {
					builder.add_empty_child(format!("Remove Flag: {}", f.repr(i)));
				}
			}
		}
		builder.end_child();
	}
}

impl PropertyDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Property Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Vars: {}", self.vars.repr(i)));
		builder.end_child();
	}
}

impl FlagDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Flag Definition"));
		builder.add_empty_child(format!("Flag Name: {}", self.flag_name.repr(i)));
		builder.add_empty_child(format!("Var Name: {}", self.var_name.repr(i)));
		builder.add_empty_child(format!("Shift: {}", self.shift));
		builder.end_child();
	}
}

impl StatesDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("States Definition"));
		builder.add_empty_child(format!("Opts: {}", self.opts.repr(i)));
		builder.begin_child(format!("Body"));
		for item in self.body.iter() {
			match item {
			    StatesBodyItem::Label(a) => { builder.add_empty_child(format!("Label: {}", a.repr(i))); }
			    StatesBodyItem::Line(s) => {
					builder.begin_child(format!("State Line"));
					builder.add_empty_child(format!("Sprite: {}", s.sprite.repr(i)));
					builder.add_empty_child(format!("Frames: {}", s.frames.repr(i)));
					builder.add_empty_child(format!("Duration: {}", s.duration.repr(i)));
					builder.add_empty_child(format!("Flags: {:?}", s.metadata.flags));
					builder.add_empty_child(
						format!("Offset: {:?}", s.metadata.offset.as_ref().map(|x| format!("({}, {})", x.0.repr(i), x.1.repr(i))))
					);
					match &s.action {
						Some(StateLineAction::Call { func, args }) => {
							let v: Option<Vec<String>> = args.as_ref().map(|x| x.iter().map(|x| match x {
								FunctionCallArg::Unnamed(e) => e.repr(i),
								FunctionCallArg::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
							}).collect());
							let s = if let Some(v) = v { format!("({})", v.join(", ")) } else { "".to_string() };
							builder.add_empty_child(format!("Action: {}{}", func.repr(i), s));
						}
						Some(StateLineAction::Anonymous(c)) => {
							builder.begin_child(format!("Anonymous Statement"));
							c.tree(i, builder);
							builder.end_child();
						}
						None => { builder.add_empty_child(format!("Action: None")); }
					}
					builder.end_child();
				}
			    StatesBodyItem::Stop => { builder.add_empty_child(format!("Stop")); }
			    StatesBodyItem::Wait => { builder.add_empty_child(format!("Wait")); }
			    StatesBodyItem::Fail => { builder.add_empty_child(format!("Fail")); }
			    StatesBodyItem::Loop => { builder.add_empty_child(format!("Loop")); }
			    StatesBodyItem::Goto { target, offset } => {
					let s = match target {
					    StateGotoTarget::Unscoped(s)   => s.repr(i),
					    StateGotoTarget::Scoped(id, s) => format!("{}::{}", id.repr(i), s.repr(i)),
					    StateGotoTarget::Super(s)      => format!("super::{}", s.repr(i))
					};
					builder.begin_child(format!("Goto"));
					builder.add_empty_child(format!("Target: {}", s));
					builder.add_empty_child(format!("Offset: {}", offset.repr(i)));
					builder.end_child();
				}
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl TypeListOrVoid {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match self {
		    TypeListOrVoid::TypeList(tys) => {
				builder.begin_child(format!("Return Type(s)"));
				for ty in tys.iter() {
					builder.add_empty_child(ty.repr(i));
				}
				builder.end_child();
			}
		    TypeListOrVoid::Void => {
				builder.add_empty_child(format!("Void Return Type"));
			}
		}
	}
}

impl FuncParam {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(self.name.repr(i));
		builder.add_empty_child(format!("Type: {}", self.param_type.repr(i)));
		builder.add_empty_child(format!("Flags: {:?}", self.flags));
		builder.add_empty_child(format!("Init: {}", self.init.repr(i)));
		builder.end_child();
	}
}

impl FuncParams {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match self {
		    FuncParams::Void => { builder.add_empty_child(format!("Void Params")); }
		    FuncParams::List { args, variadic } => {
				if args.len() == 0 && !variadic {
					builder.add_empty_child(format!("No Params"));
				} else {
					builder.begin_child(format!("Params"));
					for a in args.iter() {
						a.tree(i, builder);
					}
					if *variadic { builder.add_empty_child(format!("...")); }
					builder.end_child();
				}
			}
		}
	}
}

impl ExprList {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Expr List"));
		for e in self.0.iter() {
			builder.add_empty_child(format!("{}", e.repr(i)));
		}
		builder.end_child();
	}
}

impl LocalVariableDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Local Variable Definition"));
		builder.add_empty_child(format!("Type: {}", self.var_type.repr(i)));
		builder.begin_child(format!("Inits"));
		for v in self.inits.iter() {
			match v {
			    VarInit::Single { name, val } => {
					builder.begin_child(format!("Single"));
					builder.add_empty_child(format!("Name: {}", name.repr(i)));
					builder.add_empty_child(format!("Value: {}", val.repr(i)));
					builder.end_child();
				}
			    VarInit::Array { name, sizes, vals } => {
					builder.begin_child(format!("Array"));
					builder.add_empty_child(format!("Name: {}", name.repr(i)));
					builder.add_empty_child(format!("Sizes: {}", sizes.as_ref().map(|x| &x.0).repr(i)));
					if let Some(v) = vals {
						builder.begin_child(format!("Values"));
						v.tree(i, builder);
						builder.end_child();
					} else {
						builder.add_empty_child(format!("Values: None"));
					}
					builder.end_child();
				}
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl StaticConstArray {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Static Const Array"));
		builder.add_empty_child(format!("Type: {}", self.arr_type.repr(i)));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Values"));
		self.exprs.tree(i, builder);
		builder.end_child();
		builder.end_child();
	}
}

impl Statement {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match self {
		    Statement::Labeled(l) => match l {
		        LabeledStatement::Default => { builder.add_empty_child(format!("Default Label")); }
		        LabeledStatement::Case(c) => {
					builder.add_empty_child(format!("Case Label: {}", c.as_ref().repr(i)));
				}
		    },
		    Statement::Compound(c) => { c.tree(i, builder); }
		    Statement::Expression(e) => { builder.add_empty_child(format!("Expression: {}", e.repr(i))); }
		    Statement::If { cond, body, else_body } => {
				builder.begin_child(format!("If"));
				builder.add_empty_child(format!("Cond: {}", cond.repr(i)));
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				if let Some(e) = else_body {
					let e = e.as_ref();
					builder.begin_child(format!("Else Body"));
					e.tree(i, builder);
					builder.end_child();
				}
				builder.end_child();
			}
		    Statement::Switch { val, body } => {
				builder.begin_child(format!("Switch"));
				builder.add_empty_child(format!("Value: {}", val.repr(i)));
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				builder.end_child();
			}
		    Statement::CondIter { cond, body, iter_type } => {
				builder.begin_child(format!("Condtional Loop"));
				builder.add_empty_child(format!("Type: {:?}", iter_type));
				builder.add_empty_child(format!("Cond: {}", cond.repr(i)));
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				builder.end_child();
			}
		    Statement::For { init, cond, update, body } => {
				builder.begin_child(format!("For Loop"));
				if let Some(init) = init {
					builder.begin_child(format!("Init"));
					match init {
					    ForInit::VarDef(v)   => { v.tree(i, builder); }
					    ForInit::ExprList(e) => { e.tree(i, builder); }
					}
					builder.end_child();
				} else {
					builder.add_empty_child(format!("Init: None"));
				}
				builder.add_empty_child(format!("Cond: {}", cond.repr(i)));
				if let Some(u) = update {
					builder.begin_child(format!("Update"));
					u.tree(i, builder);
					builder.end_child();
				} else {
					builder.add_empty_child(format!("Update: None"));
				}
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				builder.end_child();
			}
		    Statement::Break => { builder.add_empty_child(format!("Break")); }
		    Statement::Continue => { builder.add_empty_child(format!("Continue")); }
		    Statement::Return(r) => {
				builder.begin_child(format!("Return"));
				if let Some(r) = r {
					builder.begin_child(format!("Values"));
					r.tree(i, builder);
					builder.end_child();
				} else {
					builder.add_empty_child(format!("Values: None"));
				}
				builder.end_child();
			}
		    Statement::LocalVariableDefinition(l) => { l.tree(i, builder); }
		    Statement::MultiAssign { assignees, rhs } => {
				builder.begin_child(format!("Multi Assign"));

				builder.begin_child(format!("Assignees"));
				assignees.tree(i, builder);
				builder.end_child();

				builder.add_empty_child(format!("Expr: {}", rhs.repr(i)));

				builder.end_child();
			}
		    Statement::StaticConstArray(s) => { s.tree(i, builder); }
		    Statement::Empty => { builder.add_empty_child(format!("Empty")); }
		}
	}
}

impl CompoundStatement {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Compound Statement"));
		for s in self.0.iter() {
			s.tree(i, builder);
		}
		builder.end_child();
	}
}

impl Declaration {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match self {
		    Declaration::Function(f) => {
				builder.begin_child(format!("Function Declaration"));
				builder.add_empty_child(format!("Name: {}", f.name.repr(i)));
				builder.add_empty_child(format!("Constant: {}", f.constant));
				builder.add_empty_child(format!("Flags: {:?}", f.metadata.flags));
				builder.add_empty_child(format!("Version: {}", f.metadata.version.repr(i)));
				builder.add_empty_child(format!("Action: {}", f.metadata.action.repr(i)));
				f.return_types.tree(i, builder);
				f.params.tree(i, builder);
				match &f.body {
					Some(b) => { b.tree(i, builder); }
					None    => { builder.add_empty_child(format!("Body: None")); }
				}
				builder.end_child();
			}
		    Declaration::Member(m) => {
				builder.begin_child(format!("Member Declaration"));
				builder.add_empty_child(format!("Flags: {:?}", m.metadata.flags));
				builder.add_empty_child(format!("Version: {}", m.metadata.version.repr(i)));
				builder.add_empty_child(format!("Action: {}", m.metadata.action.repr(i)));
				builder.add_empty_child(format!("Type: {}", m.member_type.repr(i)));
				builder.add_empty_child(format!("Names: {}", m.var_names.repr(i)));
				builder.end_child();
			}
		}
	}
}
