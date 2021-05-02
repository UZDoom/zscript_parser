use ptree::TreeBuilder;

use super::*;
use crate::print::*;
use crate::interner::*;

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
		write!(f, "{:?}", self.val)?;
		if !self.double { write!(f, "f")?; }
		Ok(())
	}
}

impl Repr for StringConst {
	fn repr(&self, (_, s): &Interners) -> String {
		format!("{:?}", s.string(self.symbol))
	}
}

impl Repr for NameConst {
	fn repr(&self, (n, _): &Interners) -> String {
		format!("'{}'", n.string(self.symbol))
	}
}

impl Repr for Identifier {
	fn repr(&self, (n, _): &Interners) -> String {
		n.string(self.symbol).to_string()
	}
}

impl Repr for NonWhitespace {
	fn repr(&self, (n, _): &Interners) -> String {
		format!("\"{}\"", n.string(self.symbol))
	}
}

impl Repr for DottableId {
	fn repr(&self, i: &Interners) -> String {
		let v: Vec<String> = self.ids.iter().map(|x| x.repr(i)).collect();
		v.join(".")
	}
}

impl Repr for Const {
	fn repr(&self, i: &Interners) -> String {
		match &self.kind {
			ConstKind::String(s)  => s.repr(i),
			ConstKind::Name(n)    => n.repr(i),
			ConstKind::Int(i)     => format!("{}", i),
			ConstKind::Float(f)   => format!("{}", f),
			ConstKind::Bool(b)    => format!("{}", b),
			ConstKind::Null       => "NULL".to_string()
		}
	}
}

impl Repr for Expression {
	fn repr(&self, i: &Interners) -> String {
		match &self.kind {
			ExpressionKind::Super => "Super".to_string(),
			ExpressionKind::Ident(id) => id.repr(i),
			ExpressionKind::Const(c) => c.repr(i),
			ExpressionKind::Binary { op, exprs } => {
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
			ExpressionKind::Prefix { op, expr } => {
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
			ExpressionKind::Postfix { op, expr } => {
				let op_str = match op {
					PostfixOp::Increment => "++",
					PostfixOp::Decrement => "--"
				};
				let expr = expr.as_ref();
				format!("({}{})", expr.repr(i), op_str)
			}
			ExpressionKind::Ternary(t) => {
				let t = t.as_ref();
				format!("({} ? {} : {})", t.cond.repr(i), t.if_true.repr(i), t.if_false.repr(i))
			}
			ExpressionKind::ArrayIndex(a) => {
				let a = a.as_ref();
				format!("({}[{}])", a.lhs.repr(i), a.index.repr(i))
			}
			ExpressionKind::FunctionCall { lhs, exprs } => {
				let v: Vec<String> = exprs.iter().map(|x| match &x.kind {
					FunctionCallArgKind::Unnamed(e) => e.repr(i),
					FunctionCallArgKind::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
				}).collect();
				format!("({}({}))", lhs.as_ref().repr(i), v.join(", "))
			}
			ExpressionKind::Vector2(v) => {
				let (x, y) = v.as_ref();
				format!("({}, {})", x.repr(i), y.repr(i))
			}
			ExpressionKind::Vector3(v) => {
				let (x, y, z) = v.as_ref();
				format!("({}, {}, {})", x.repr(i), y.repr(i), z.repr(i))
			}
			ExpressionKind::ClassCast(id, exprs) => {
				let v: Vec<String> = exprs.iter().map(|x| match &x.kind {
					FunctionCallArgKind::Unnamed(e) => e.repr(i),
					FunctionCallArgKind::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
				}).collect();
				format!("((class<{}>) ({}))", id.repr(i), v.join(", "))
			}
		}
	}
}

impl Repr for Type {
	fn repr(&self, i: &Interners) -> String {
		match &self.kind {
			TypeKind::SingleUserType(id)     => id.repr(i),
			TypeKind::DottedUserType(ids)    => format!(".{}", ids.repr(i)),
			TypeKind::NativeType(id)         => format!("@{}", id.repr(i)),
			TypeKind::ReadonlyType(id)       => format!("ReadOnly< {} >", id.repr(i)),
			TypeKind::ReadonlyNativeType(id) => format!("ReadOnly< @{} >", id.repr(i)),
			TypeKind::Class(ids) => {
				format!("Class{}", if let Some(ids) = ids { format!("< {} >", ids.repr(i)) } else { "".to_string() })
			}
			TypeKind::Map(tys) => {
				let tys = tys.as_ref();
				format!("Map< {}, {} >", tys.0.repr(i), tys.1.repr(i))
			}
			TypeKind::DynArray(ty) => {
				let ty = ty.as_ref();
				format!("Array< {} >", ty.repr(i))
			}
			TypeKind::Let => { "let".to_string() }
		}
	}
}

impl Repr for TypeOrArray {
	fn repr(&self, i: &Interners) -> String {
		match &self.kind {
			TypeOrArrayKind::Type(t) => t.repr(i),
			TypeOrArrayKind::Array(t, e) => {
				let v: Vec<String> = e.list.iter().map(|x| format!("[{}]", x.repr(i))).collect();
				format!("{}{}", t.repr(i), v.join(""))
			}
		}
	}
}

impl Repr for TypeListOrVoid {
	fn repr(&self, i: &Interners) -> String {
		match &self.kind {
			TypeListOrVoidKind::TypeList(tys) => {
				let v: Vec<String> = tys.iter().map(|x| x.repr(i)).collect();
				v.join(", ")
			}
			TypeListOrVoidKind::Void => "void".to_string()
		}
	}
}

impl Tree for EnumDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Enum Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.add_empty_child(format!("Type: {:?}", self.enum_type));
		builder.begin_child(format!("Body"));
		for EnumVariant { name, init, doc_comment, .. } in self.variants.iter() {
			builder.begin_child(format!("Variant: {}", name.repr(i)));
			builder.add_empty_child(format!("Doc: {}", doc_comment.repr(i)));
			if let Some(e) = init {
				builder.add_empty_child(format!("Init: {}", e.repr(i)));
			}
			builder.end_child();
		}
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for ConstDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Const Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.add_empty_child(format!("Expression: {}", self.expr.repr(i)));
		builder.end_child();
	}
}

impl Tree for DefaultDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Default Definition"));
		for d in self.statements.iter() {
			match &d.kind {
				DefaultStatementKind::Property { prop, vals } => {
					builder.begin_child(format!("Property"));
					builder.add_empty_child(format!("Property: {}", prop.repr(i)));
					builder.add_empty_child(format!("Vals: {}", vals.as_ref().map(|x| &x.list).repr(i)));
					builder.end_child();
				}
				DefaultStatementKind::AddFlag(f) => {
					builder.add_empty_child(format!("Add Flag: {}", f.repr(i)));
				}
				DefaultStatementKind::RemoveFlag(f) => {
					builder.add_empty_child(format!("Remove Flag: {}", f.repr(i)));
				}
			}
		}
		builder.end_child();
	}
}

impl Tree for PropertyDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Property Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.add_empty_child(format!("Vars: {}", self.vars.repr(i)));
		builder.end_child();
	}
}

impl Tree for FlagDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Flag Definition"));
		builder.add_empty_child(format!("Flag Name: {}", self.flag_name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.add_empty_child(format!("Var Name: {}", self.var_name.repr(i)));
		builder.add_empty_child(format!("Shift: {}", self.shift));
		builder.end_child();
	}
}

impl Tree for StaticConstArray {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Static Const Array"));
		builder.add_empty_child(format!("Type: {}", self.arr_type.repr(i)));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.begin_child(format!("Values"));
		self.exprs.tree(i, builder);
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for TypeListOrVoid {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match &self.kind {
			TypeListOrVoidKind::TypeList(tys) => {
				builder.begin_child(format!("Return Type(s)"));
				for ty in tys.iter() {
					builder.add_empty_child(ty.repr(i));
				}
				builder.end_child();
			}
			TypeListOrVoidKind::Void => {
				builder.add_empty_child(format!("Void Return Type"));
			}
		}
	}
}

impl Tree for ExprList {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Expr List"));
		for e in self.list.iter() {
			builder.add_empty_child(format!("{}", e.repr(i)));
		}
		builder.end_child();
	}
}

