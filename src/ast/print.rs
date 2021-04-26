use ptree::TreeBuilder;

use super::*;
use crate::interner::Interners;

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
		format!("{:?}", s.string(*self))
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

pub(crate) trait Tree {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder);
}

impl<T: Tree> Tree for &T {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		(*self).tree(i, builder);
	}
}

impl Tree for TopLevel {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.add_empty_child(format!("Version: {:?}", self.version.as_ref().map(|x| format!("{}", x))));
		for d in self.definitions.iter() {
			match &d.kind {
				TopLevelDefinitionKind::Class(x)        => { x.tree(i, builder); }
				TopLevelDefinitionKind::Struct(x)       => { x.tree(i, builder); }
				TopLevelDefinitionKind::ExtendClass(x)  => { x.tree(i, builder); }
				TopLevelDefinitionKind::ExtendStruct(x) => { x.tree(i, builder); }
				TopLevelDefinitionKind::MixinClass(x)   => { x.tree(i, builder); }
				TopLevelDefinitionKind::Enum(x)         => { x.tree(i, builder); }
				TopLevelDefinitionKind::Const(x)        => { x.tree(i, builder); }
				TopLevelDefinitionKind::Include(x)      => { builder.add_empty_child(format!("Include: {}", x.repr(i))); }
			}
		}
	}
}

impl TopLevel {
	pub fn repr(&self, interners: &Interners) -> String {
		let mut builder = ptree::TreeBuilder::new("AST".to_string());
		self.tree(interners, &mut builder);
		let t = builder.build();
		let mut cursor: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
		ptree::write_tree(&t, &mut cursor).unwrap();
		let data = cursor.into_inner();
		std::str::from_utf8(&data).unwrap().to_string()
	}
}

impl Tree for ClassDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Class Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.add_empty_child(format!("Ancestor: {}", self.ancestor.repr(i)));
		builder.begin_child(format!("Metadata"));
		for m in self.metadata.iter() {
			match &m.kind {
				ClassMetadataItemKind::Abstract    => { builder.add_empty_child(format!("Abstract")); }
				ClassMetadataItemKind::Native      => { builder.add_empty_child(format!("Native"));   }
				ClassMetadataItemKind::UI          => { builder.add_empty_child(format!("UI"));       }
				ClassMetadataItemKind::Play        => { builder.add_empty_child(format!("Play"));     }

				ClassMetadataItemKind::Version(v)  => { builder.add_empty_child(format!("Version({})", v.repr(i))); }
				ClassMetadataItemKind::Replaces(d) => { builder.add_empty_child(format!("Replaces: {}", d.repr(i))); }
			}
		}
		builder.end_child();
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match &inner.kind {
				ClassInnerKind::Declaration(x)      => { x.tree(i, builder); }
				ClassInnerKind::Enum(x)             => { x.tree(i, builder); }
				ClassInnerKind::Struct(x)           => { x.tree(i, builder); }
				ClassInnerKind::States(x)           => { x.tree(i, builder); }
				ClassInnerKind::Default(x)          => { x.tree(i, builder); }
				ClassInnerKind::Const(x)            => { x.tree(i, builder); }
				ClassInnerKind::Property(x)         => { x.tree(i, builder); }
				ClassInnerKind::Flag(x)             => { x.tree(i, builder); }
				ClassInnerKind::StaticConstArray(x) => { x.tree(i, builder); }
				ClassInnerKind::Mixin(x)            => { builder.add_empty_child(format!("Mixin: {}", x.repr(i))); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for StructDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Struct Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.begin_child(format!("Metadata"));
		for m in self.metadata.iter() {
			match &m.kind {
				StructMetadataItemKind::ClearScope => { builder.add_empty_child(format!("ClearScope")); }
				StructMetadataItemKind::Native     => { builder.add_empty_child(format!("Native"));     }
				StructMetadataItemKind::UI         => { builder.add_empty_child(format!("UI"));         }
				StructMetadataItemKind::Play       => { builder.add_empty_child(format!("Play"));       }

				StructMetadataItemKind::Version(v) => { builder.add_empty_child(format!("Version({})", v.repr(i))); }
			}
		}
		builder.end_child();
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match &inner.kind {
				StructInnerKind::Declaration(x)      => { x.tree(i, builder); }
				StructInnerKind::Enum(x)             => { x.tree(i, builder); }
				StructInnerKind::Const(x)            => { x.tree(i, builder); }
				StructInnerKind::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for ExtendClass {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Class Extension"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match &inner.kind {
				ClassInnerKind::Declaration(x)      => { x.tree(i, builder); }
				ClassInnerKind::Enum(x)             => { x.tree(i, builder); }
				ClassInnerKind::Struct(x)           => { x.tree(i, builder); }
				ClassInnerKind::States(x)           => { x.tree(i, builder); }
				ClassInnerKind::Default(x)          => { x.tree(i, builder); }
				ClassInnerKind::Const(x)            => { x.tree(i, builder); }
				ClassInnerKind::Property(x)         => { x.tree(i, builder); }
				ClassInnerKind::Flag(x)             => { x.tree(i, builder); }
				ClassInnerKind::StaticConstArray(x) => { x.tree(i, builder); }
				ClassInnerKind::Mixin(x)            => { builder.add_empty_child(format!("Mixin: {}", x.repr(i))); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for ExtendStruct {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Struct Extension"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match &inner.kind {
				StructInnerKind::Declaration(x)      => { x.tree(i, builder); }
				StructInnerKind::Enum(x)             => { x.tree(i, builder); }
				StructInnerKind::Const(x)            => { x.tree(i, builder); }
				StructInnerKind::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
	}
}

impl Tree for MixinClassDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Mixin Class Definition"));
		builder.add_empty_child(format!("Name: {}", self.name.repr(i)));
		builder.add_empty_child(format!("Doc: {}", self.doc_comment.repr(i)));
		builder.begin_child(format!("Body"));
		for inner in self.inners.iter() {
			match &inner.kind {
				MixinClassInnerKind::Declaration(x)      => { x.tree(i, builder); }
				MixinClassInnerKind::Enum(x)             => { x.tree(i, builder); }
				MixinClassInnerKind::Struct(x)           => { x.tree(i, builder); }
				MixinClassInnerKind::States(x)           => { x.tree(i, builder); }
				MixinClassInnerKind::Default(x)          => { x.tree(i, builder); }
				MixinClassInnerKind::Const(x)            => { x.tree(i, builder); }
				MixinClassInnerKind::Property(x)         => { x.tree(i, builder); }
				MixinClassInnerKind::Flag(x)             => { x.tree(i, builder); }
				MixinClassInnerKind::StaticConstArray(x) => { x.tree(i, builder); }
			}
		}
		builder.end_child();
		builder.end_child();
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

impl Tree for StatesDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("States Definition"));
		builder.add_empty_child(format!("Opts: {}", self.opts.repr(i)));
		builder.begin_child(format!("Body"));
		for item in self.body.iter() {
			match &item.kind {
				StatesBodyItemKind::Label(a) => { builder.add_empty_child(format!("Label: {}", a.repr(i))); }
				StatesBodyItemKind::Line(s) => {
					builder.begin_child(format!("State Line"));
					builder.add_empty_child(format!("Sprite: {}", s.sprite.repr(i)));
					builder.add_empty_child(format!("Frames: {}", s.frames.repr(i)));
					builder.begin_child(format!("Metadata"));
					for m in s.metadata.iter() {
						match &m.kind {
							StateLineMetadataItemKind::Bright         => { builder.add_empty_child(format!("Bright")); }
							StateLineMetadataItemKind::Fast           => { builder.add_empty_child(format!("Fast")); }
							StateLineMetadataItemKind::Slow           => { builder.add_empty_child(format!("Slow")); }
							StateLineMetadataItemKind::NoDelay        => { builder.add_empty_child(format!("NoDelay")); }
							StateLineMetadataItemKind::CanRaise       => { builder.add_empty_child(format!("CanRaise")); }
							StateLineMetadataItemKind::Offset(e0, e1) => { builder.add_empty_child(format!("Offset({}, {})", e0.repr(i), e1.repr(i))); }
							StateLineMetadataItemKind::Light(l) => {
								let v: Vec<String> = l.iter().map(|x| x.repr(i)).collect();
								builder.add_empty_child(format!("Light({})", v.join(", ")));
							}
						}
					}
					builder.end_child();
					match &s.action {
						Some(s) => match &s.kind {
							StateLineActionKind::Call { func, args } => {
								let v: Option<Vec<String>> = args.as_ref().map(|x| x.iter().map(|x| match &x.kind {
									FunctionCallArgKind::Unnamed(e) => e.repr(i),
									FunctionCallArgKind::Named(n, e) => format!("{}: {}", n.repr(i), e.repr(i))
								}).collect());
								let s = if let Some(v) = v { format!("({})", v.join(", ")) } else { "".to_string() };
								builder.add_empty_child(format!("Action: {}{}", func.repr(i), s));
							}
							StateLineActionKind::Anonymous(c) => {
								builder.begin_child(format!("Anonymous Statement"));
								c.tree(i, builder);
								builder.end_child();
							}
						}
						None => { builder.add_empty_child(format!("Action: None")); }
					}
					builder.end_child();
				}
				StatesBodyItemKind::Stop => { builder.add_empty_child(format!("Stop")); }
				StatesBodyItemKind::Wait => { builder.add_empty_child(format!("Wait")); }
				StatesBodyItemKind::Fail => { builder.add_empty_child(format!("Fail")); }
				StatesBodyItemKind::Loop => { builder.add_empty_child(format!("Loop")); }
				StatesBodyItemKind::Goto { target, offset } => {
					let s = match &target.kind {
						StateGotoTargetKind::Unscoped(s)   => s.repr(i),
						StateGotoTargetKind::Scoped(id, s) => format!("{}::{}", id.repr(i), s.repr(i)),
						StateGotoTargetKind::Super(s)      => format!("super::{}", s.repr(i))
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

impl Tree for FuncParam {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(self.name.repr(i));
		builder.add_empty_child(format!("Type: {}", self.param_type.repr(i)));
		builder.begin_child(format!("Flags"));
		for f in self.flags.iter() {
			match &f.kind {
			    ParamFlagItemKind::In       => { builder.add_empty_child(format!("In"));       }
			    ParamFlagItemKind::Out      => { builder.add_empty_child(format!("Out"));      }
			    ParamFlagItemKind::Optional => { builder.add_empty_child(format!("Optional")); }
			}
		}
		builder.end_child();
		builder.add_empty_child(format!("Init: {}", self.init.repr(i)));
		builder.end_child();
	}
}

impl Tree for FuncParams {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match &self.kind {
			FuncParamsKind::Void => { builder.add_empty_child(format!("Void Params")); }
			FuncParamsKind::List { args, variadic } => {
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

impl Tree for ExprList {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Expr List"));
		for e in self.list.iter() {
			builder.add_empty_child(format!("{}", e.repr(i)));
		}
		builder.end_child();
	}
}

impl Tree for LocalVariableDefinition {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Local Variable Definition"));
		builder.add_empty_child(format!("Type: {}", self.var_type.repr(i)));
		builder.begin_child(format!("Inits"));
		for v in self.inits.iter() {
			match &v.kind {
				VarInitKind::Single { name, val } => {
					builder.begin_child(format!("Single"));
					builder.add_empty_child(format!("Name: {}", name.repr(i)));
					builder.add_empty_child(format!("Value: {}", val.repr(i)));
					builder.end_child();
				}
				VarInitKind::Array { name, sizes, vals } => {
					builder.begin_child(format!("Array"));
					builder.add_empty_child(format!("Name: {}", name.repr(i)));
					builder.add_empty_child(format!("Sizes: {}", sizes.as_ref().map(|x| &x.list).repr(i)));
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

impl Tree for Statement {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match &self.kind {
			StatementKind::Labeled(l) => match l {
				LabeledStatement::Default => { builder.add_empty_child(format!("Default Label")); }
				LabeledStatement::Case(c) => {
					builder.add_empty_child(format!("Case Label: {}", c.as_ref().repr(i)));
				}
			},
			StatementKind::Compound(c) => { c.tree(i, builder); }
			StatementKind::Expression(e) => { builder.add_empty_child(format!("Expression: {}", e.repr(i))); }
			StatementKind::If { cond, body, else_body } => {
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
			StatementKind::Switch { val, body } => {
				builder.begin_child(format!("Switch"));
				builder.add_empty_child(format!("Value: {}", val.repr(i)));
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				builder.end_child();
			}
			StatementKind::CondIter { cond, body, iter_type } => {
				builder.begin_child(format!("Condtional Loop"));
				builder.add_empty_child(format!("Type: {:?}", iter_type));
				builder.add_empty_child(format!("Cond: {}", cond.repr(i)));
				builder.begin_child(format!("Body"));
				body.tree(i, builder);
				builder.end_child();
				builder.end_child();
			}
			StatementKind::For { init, cond, update, body } => {
				builder.begin_child(format!("For Loop"));
				if let Some(init) = init {
					builder.begin_child(format!("Init"));
					match &init.kind {
						ForInitKind::VarDef(v)   => { v.tree(i, builder); }
						ForInitKind::ExprList(e) => { e.tree(i, builder); }
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
			StatementKind::Break => { builder.add_empty_child(format!("Break")); }
			StatementKind::Continue => { builder.add_empty_child(format!("Continue")); }
			StatementKind::Return(r) => {
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
			StatementKind::LocalVariableDefinition(l) => { l.tree(i, builder); }
			StatementKind::MultiAssign { assignees, rhs } => {
				builder.begin_child(format!("Multi Assign"));

				builder.begin_child(format!("Assignees"));
				assignees.tree(i, builder);
				builder.end_child();

				builder.add_empty_child(format!("Expr: {}", rhs.repr(i)));

				builder.end_child();
			}
			StatementKind::StaticConstArray(s) => { s.tree(i, builder); }
			StatementKind::Empty => { builder.add_empty_child(format!("Empty")); }
		}
	}
}

impl Tree for CompoundStatement {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Compound Statement"));
		for s in self.statements.iter() {
			s.tree(i, builder);
		}
		builder.end_child();
	}
}

impl Tree for Vec<DeclarationMetadataItem> {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		builder.begin_child(format!("Metadata"));
		for m in self.iter() {
			match &m.kind {
				DeclarationMetadataItemKind::Native                          => { builder.add_empty_child(format!("Native"));       }
				DeclarationMetadataItemKind::Static                          => { builder.add_empty_child(format!("Static"));       }
				DeclarationMetadataItemKind::Private                         => { builder.add_empty_child(format!("Private"));      }
				DeclarationMetadataItemKind::Protected                       => { builder.add_empty_child(format!("Protected"));    }
				DeclarationMetadataItemKind::Latent                          => { builder.add_empty_child(format!("Latent"));       }
				DeclarationMetadataItemKind::Final                           => { builder.add_empty_child(format!("Final"));        }
				DeclarationMetadataItemKind::Meta                            => { builder.add_empty_child(format!("Meta"));         }
				DeclarationMetadataItemKind::Transient                       => { builder.add_empty_child(format!("Transient"));    }
				DeclarationMetadataItemKind::ReadOnly                        => { builder.add_empty_child(format!("ReadOnly"));     }
				DeclarationMetadataItemKind::Internal                        => { builder.add_empty_child(format!("Internal"));     }
				DeclarationMetadataItemKind::Virtual                         => { builder.add_empty_child(format!("Virtual"));      }
				DeclarationMetadataItemKind::Override                        => { builder.add_empty_child(format!("Override"));     }
				DeclarationMetadataItemKind::Abstract                        => { builder.add_empty_child(format!("Abstract"));     }
				DeclarationMetadataItemKind::VarArg                          => { builder.add_empty_child(format!("VarArg"));       }
				DeclarationMetadataItemKind::UI                              => { builder.add_empty_child(format!("UI"));           }
				DeclarationMetadataItemKind::Play                            => { builder.add_empty_child(format!("Play"));         }
				DeclarationMetadataItemKind::ClearScope                      => { builder.add_empty_child(format!("ClearScope"));   }
				DeclarationMetadataItemKind::VirtualScope                    => { builder.add_empty_child(format!("VirtualScope")); }

				DeclarationMetadataItemKind::Deprecated { version, message } => { builder.add_empty_child(format!("Deprecated({}, {})", version.repr(i), message.repr(i))); }
				DeclarationMetadataItemKind::Version(v)                      => { builder.add_empty_child(format!("Version({})", v.repr(i))); }

				DeclarationMetadataItemKind::Action(l) => {
					if let Some(l) = l {
						let v: Vec<String> = l.iter().map(|x| x.repr(i)).collect();
						builder.add_empty_child(format!("Action({})", v.join(", ")));
					} else {
						builder.add_empty_child(format!("Action"));
					}
				}
			}
		}
		builder.end_child();
	}
}

impl Tree for Declaration {
	fn tree(&self, i: &Interners, builder: &mut TreeBuilder) {
		match self {
			Declaration::Function(f) => {
				builder.begin_child(format!("Function Declaration"));
				builder.add_empty_child(format!("Name: {}", f.name.repr(i)));
				builder.add_empty_child(format!("Doc: {}", f.doc_comment.repr(i)));
				builder.add_empty_child(format!("Constant: {}", f.constant));
				f.metadata.tree(i, builder);
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
				builder.add_empty_child(format!("Type: {}", m.member_type.repr(i)));
				builder.begin_child(format!("Names"));
				for c in m.vars.iter() {
					let v: Option<Vec<String>> = c.1.as_ref().map(|x| x.list.iter().map(|y| format!("[{}]", y.repr(i))).collect());
					builder.add_empty_child(format!("{}{}", c.0.repr(i), v.map(|x| x.join("")).unwrap_or_else(|| "".to_string())));
				}
				builder.add_empty_child(format!("Doc: {}", m.doc_comment.repr(i)));
				builder.end_child();
				m.metadata.tree(i, builder);
				builder.end_child();
			}
		}
	}
}
