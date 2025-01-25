use std::collections::HashMap;

use vec1::*;

use crate::err::{ParsingError, ParsingErrorLevel};
use crate::interner::*;
use crate::ir_common::*;
use crate::parser_manager::{FileIndexAndAst, FileSystemParseResult};
use crate::Span;

use crate::ast;
use crate::hir;

#[derive(Debug)]
pub struct HirLoweringResult {
    pub hir: hir::TopLevel,
}

fn shadow_warning(span: Span) -> ParsingError {
    ParsingError {
        level: ParsingErrorLevel::Warning,
        msg: "shadowed metadata item".to_string(),
        main_spans: vec1![span],
        info_spans: vec![],
    }
}

macro_rules! dedup_flag {
    ($span: expr, $errs: expr, $targ: expr, $flag: path) => {
        if $targ.contains($flag) {
            $errs.push(shadow_warning($span));
        } else {
            $targ |= $flag;
        }
    };
}

fn lower_array_type(ty: hir::Type, sizes: ArraySizes) -> hir::Type {
    let mut ret = ty;
    for s in sizes.list.into_iter().rev() {
        ret = hir::Type::Array(Box::new(ret), s);
    }
    ret
}

fn statement_to_compound(stmt: ast::Statement) -> ast::CompoundStatement {
    let span = stmt.span;
    let statements = match stmt.kind {
        ast::StatementKind::Compound(c) => {
            return c;
        }

        ast::StatementKind::Empty => vec![],
        _ => vec![stmt],
    };
    ast::CompoundStatement { span, statements }
}

pub struct HirLowerer<'outer> {
    errs: &'outer mut Vec<ParsingError>,
    base_archive: bool,
    archive_num: usize,
}

impl<'outer> HirLowerer<'outer> {
    fn lower_compound_statement(&mut self, s: ast::CompoundStatement) -> hir::CompoundStatement {
        let mut statements = vec![];
        for s in s.statements {
            self.lower_statement(s, &mut statements);
        }
        hir::CompoundStatement {
            span: Some(s.span),
            statements,
        }
    }

    fn lower_return_types(&mut self, return_types: ast::TypeListOrVoid) -> hir::TypeListOrVoid {
        let return_types_kind = match return_types.kind {
            ast::TypeListOrVoidKind::TypeList(l) => {
                hir::TypeListOrVoidKind::TypeList(l.mapped(|t| match t.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        lower_array_type(self.ast_type_to_hir_type(t), s)
                    }
                }))
            }
            ast::TypeListOrVoidKind::Void => hir::TypeListOrVoidKind::Void,
        };
        hir::TypeListOrVoid {
            span: return_types.span,
            kind: return_types_kind,
        }
    }

    fn ast_type_to_hir_type(&mut self, ty: ast::Type) -> hir::Type {
        match ty.kind {
            ast::TypeKind::SingleUserType(t) => hir::Type::SingleUserType(t),
            ast::TypeKind::DottedUserType(t) => hir::Type::DottedUserType(t),
            ast::TypeKind::NativeType(t) => hir::Type::NativeType(t),
            ast::TypeKind::ReadonlyType(t) => hir::Type::ReadonlyType(t),
            ast::TypeKind::ReadonlyNativeType(t) => hir::Type::ReadonlyNativeType(t),
            ast::TypeKind::Class(t) => hir::Type::Class(t),
            ast::TypeKind::Let => hir::Type::Let,
            ast::TypeKind::Error => hir::Type::Error,
            ast::TypeKind::Map(b) => {
                let (key, value) = *b;
                let t0 = match key.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        let new = self.ast_type_to_hir_type(t);
                        lower_array_type(new, s)
                    }
                };
                let t1 = match value.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        let new = self.ast_type_to_hir_type(t);
                        lower_array_type(new, s)
                    }
                };
                hir::Type::Map(Box::new((t0, t1)))
            }
            ast::TypeKind::MapIterator(b) => {
                let (key, value) = *b;
                let t0 = match key.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        let new = self.ast_type_to_hir_type(t);
                        lower_array_type(new, s)
                    }
                };
                let t1 = match value.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        let new = self.ast_type_to_hir_type(t);
                        lower_array_type(new, s)
                    }
                };
                hir::Type::MapIterator(Box::new((t0, t1)))
            }
            ast::TypeKind::DynArray(b) => {
                let t = *b;
                let t = match t.kind {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, s) => {
                        let new = self.ast_type_to_hir_type(t);
                        lower_array_type(new, s)
                    }
                };
                hir::Type::DynArray(Box::new(t))
            }
            ast::TypeKind::FuncPtr(ast::FuncPtrKind::Void) => {
                hir::Type::FuncPtr(hir::FuncPtrKind::Void)
            }
            ast::TypeKind::FuncPtr(ast::FuncPtrKind::Detailed(f)) => {
                let flag = match f.flag {
                    ast::FuncPtrFlag::Ui => hir::FuncPtrFlag::Ui,
                    ast::FuncPtrFlag::Play => hir::FuncPtrFlag::Play,
                    ast::FuncPtrFlag::ClearScope => hir::FuncPtrFlag::ClearScope,
                };
                let return_types = self.lower_return_types(f.return_types);
                let params_kind = match f.params.kind {
                    ast::FuncPtrParamsKind::Void => hir::FuncPtrParamsKind::Void,
                    ast::FuncPtrParamsKind::List(l) => hir::FuncPtrParamsKind::List(
                        l.into_iter()
                            .map(|p| {
                                let mut res = hir::FuncPtrParam {
                                    flags: hir::FuncParamFlags::empty(),
                                    ty: self.ast_type_to_hir_type(p.ty),
                                };
                                self.lower_function_param_flags(&mut res.flags, p.span, p.flags);
                                res
                            })
                            .collect(),
                    ),
                };
                let params = hir::FuncPtrParams {
                    span: f.params.span,
                    kind: params_kind,
                };
                hir::Type::FuncPtr(hir::FuncPtrKind::Detailed(hir::FuncPtrDetailed {
                    flag,
                    return_types,
                    params,
                }))
            }
        }
    }

    fn lower_variable_definition(
        &mut self,
        l: ast::LocalVariableDefinition,
    ) -> Vec<hir::LocalVariableDefinition> {
        let ast::LocalVariableDefinition {
            span,
            var_type,
            inits,
        } = l;
        let base_type = self.ast_type_to_hir_type(var_type);
        inits
            .into_iter()
            .map(|i| {
                let (names, var_type, init) = match i.kind {
                    ast::VarInitKind::Single { name, val } => (
                        vec1![name],
                        base_type.clone(),
                        val.map(hir::VarInit::Single),
                    ),
                    ast::VarInitKind::Multi { names, val } => {
                        (names, base_type.clone(), Some(hir::VarInit::Single(val)))
                    }
                    ast::VarInitKind::Array { name, sizes, vals } => {
                        match sizes {
                            Some(s) => {
                                let t = lower_array_type(base_type.clone(), s);
                                (vec1![name], t, vals.map(hir::VarInit::Compound))
                            }
                            None => {
                                // the only way the parser can produce an Array kind with no sizes
                                // is if there was a compound initializer
                                // in which case the zscript compiler completely ignores every value except the first
                                // (including for type checking purposes) and treats it as a non array type
                                // because good language design
                                let init = vals.unwrap();
                                self.errs.push(ParsingError {
                                    level: ParsingErrorLevel::Warning,
                                    msg: "compound initializer on a non-array type".to_string(),
                                    main_spans: vec1![init.span],
                                    info_spans: vec![],
                                });
                                let val = init.list.into_iter().next().unwrap();
                                (
                                    vec1![name],
                                    base_type.clone(),
                                    Some(hir::VarInit::Single(val)),
                                )
                            }
                        }
                    }
                };
                hir::LocalVariableDefinition {
                    span,
                    var_type,
                    names,
                    init,
                }
            })
            .collect()
    }

    fn lower_static_const_array(&mut self, sca: ast::StaticConstArray) -> hir::StaticConstArray {
        let ast::StaticConstArray {
            doc_comment,
            span,
            arr_type,
            name,
            exprs,
        } = sca;
        let arr_type = self.ast_type_to_hir_type(arr_type);
        hir::StaticConstArray {
            doc_comment,
            span,
            arr_type,
            name,
            exprs,
        }
    }

    fn lower_statement(&mut self, s: ast::Statement, stmts: &mut Vec<hir::Statement>) {
        macro_rules! add_simple {
            ($kind: expr) => {{
                let n = hir::Statement {
                    span: Some(s.span),
                    kind: $kind,
                };
                stmts.push(n);
            }};
        }
        match s.kind {
            ast::StatementKind::Labeled(l) => add_simple!(hir::StatementKind::Labeled(l)),
            ast::StatementKind::Compound(s) => add_simple!(hir::StatementKind::Compound(
                self.lower_compound_statement(s)
            )),
            ast::StatementKind::Expression(e) => add_simple!(hir::StatementKind::Expression(e)),
            ast::StatementKind::Break => add_simple!(hir::StatementKind::Break),
            ast::StatementKind::Continue => add_simple!(hir::StatementKind::Continue),
            ast::StatementKind::Return(e) => add_simple!(hir::StatementKind::Return(e)),
            ast::StatementKind::MultiAssign { assignees, rhs } => {
                add_simple!(hir::StatementKind::MultiAssign { assignees, rhs })
            }

            ast::StatementKind::If {
                cond,
                body,
                else_body,
            } => add_simple!(hir::StatementKind::If {
                cond,
                body: self.lower_compound_statement(statement_to_compound(*body)),
                else_body: else_body
                    .map(|x| self.lower_compound_statement(statement_to_compound(*x)))
            }),
            ast::StatementKind::Switch { val, body } => add_simple!(hir::StatementKind::Switch {
                val,
                body: self.lower_compound_statement(statement_to_compound(*body))
            }),
            ast::StatementKind::StaticConstArray(a) => {
                add_simple!(hir::StatementKind::StaticConstArray(
                    self.lower_static_const_array(a)
                ));
            }

            ast::StatementKind::Empty => {}

            ast::StatementKind::LocalVariableDefinition(l) => {
                for l in self.lower_variable_definition(l) {
                    add_simple!(hir::StatementKind::LocalVariableDefinition(l));
                }
            }
            ast::StatementKind::CondIter {
                cond,
                body,
                iter_type,
            } => {
                // conditional iteration loops are compiled roughly as such:
                // loop {
                //     <IF WHILE LOOP> if (!<COND>) { break; }
                //     <IF UNTIL LOOP> if (<COND>)  { break; }
                //     {
                //         <LOOP BODY>
                //     }
                //     <IF DO-WHILE LOOP> if (!<COND>) { break; }
                //     <IF DO-UNTIL LOOP> if (<COND>)  { break; }
                // }
                let mut loop_body = hir::CompoundStatement {
                    span: Some(body.span),
                    statements: vec![],
                };
                let cond_expr = if matches!(
                    iter_type,
                    ast::CondIterType::Until | ast::CondIterType::DoUntil
                ) {
                    cond
                } else {
                    Expression {
                        span: cond.span,
                        kind: ExpressionKind::Prefix {
                            op: PrefixOp::LogicalNot,
                            expr: Box::new(cond),
                        },
                    }
                };
                let cond_break_kind = hir::StatementKind::If {
                    cond: cond_expr,
                    body: hir::CompoundStatement {
                        span: None,
                        statements: vec![hir::Statement {
                            span: None,
                            kind: hir::StatementKind::Break,
                        }],
                    },
                    else_body: None,
                };
                match iter_type {
                    ast::CondIterType::While | ast::CondIterType::Until => {
                        loop_body.statements.push(hir::Statement {
                            span: None,
                            kind: cond_break_kind,
                        });
                        loop_body.statements.push(hir::Statement {
                            span: Some(body.span),
                            kind: hir::StatementKind::Compound(
                                self.lower_compound_statement(statement_to_compound(*body)),
                            ),
                        });
                    }
                    ast::CondIterType::DoWhile | ast::CondIterType::DoUntil => {
                        loop_body.statements.push(hir::Statement {
                            span: Some(body.span),
                            kind: hir::StatementKind::Compound(
                                self.lower_compound_statement(statement_to_compound(*body)),
                            ),
                        });
                        loop_body.statements.push(hir::Statement {
                            span: None,
                            kind: cond_break_kind,
                        });
                    }
                }
                add_simple!(hir::StatementKind::Loop(loop_body));
            }
            ast::StatementKind::For {
                init,
                cond,
                update,
                body,
            } => {
                // conditional iteration loops are compiled roughly as such:
                // {
                //     <INIT>
                //     loop {
                //         if (!<COND>) { break; }
                //         {
                //             <LOOP BODY>
                //         }
                //         <UPDATE>
                //     }
                // }
                let mut outer = hir::CompoundStatement {
                    span: Some(s.span),
                    statements: vec![],
                };
                if let Some(v) = init {
                    match v.kind {
                        ast::ForInitKind::VarDef(l) => {
                            let defs = self.lower_variable_definition(l);
                            for l in defs {
                                outer.statements.push(hir::Statement {
                                    span: Some(l.span),
                                    kind: hir::StatementKind::LocalVariableDefinition(l),
                                });
                            }
                        }
                        ast::ForInitKind::ExprList(exprs) => {
                            for e in exprs.list {
                                outer.statements.push(hir::Statement {
                                    span: e.span,
                                    kind: hir::StatementKind::Expression(e),
                                });
                            }
                        }
                    }
                }
                let mut loop_body = hir::CompoundStatement {
                    span: Some(body.span),
                    statements: vec![],
                };
                if let Some(c) = cond {
                    let cond_expr = Expression {
                        span: c.span,
                        kind: ExpressionKind::Prefix {
                            op: PrefixOp::LogicalNot,
                            expr: Box::new(c),
                        },
                    };
                    let kind = hir::StatementKind::If {
                        cond: cond_expr,
                        body: hir::CompoundStatement {
                            span: None,
                            statements: vec![hir::Statement {
                                span: None,
                                kind: hir::StatementKind::Break,
                            }],
                        },
                        else_body: None,
                    };
                    loop_body
                        .statements
                        .push(hir::Statement { span: None, kind })
                }
                loop_body.statements.push(hir::Statement {
                    span: Some(body.span),
                    // this must be made into a compound statement so that scoping rules are still followed
                    kind: hir::StatementKind::Compound(
                        self.lower_compound_statement(statement_to_compound(*body)),
                    ),
                });
                if let Some(exprs) = update {
                    for e in exprs.list {
                        loop_body.statements.push(hir::Statement {
                            span: e.span,
                            kind: hir::StatementKind::Expression(e),
                        });
                    }
                }
                outer.statements.push(hir::Statement {
                    span: loop_body.span,
                    kind: hir::StatementKind::Compound(loop_body),
                });
                add_simple!(hir::StatementKind::Loop(outer));
            }
        }
    }

    fn lower_function_param_flags(
        &mut self,
        target: &mut hir::FuncParamFlags,
        source_span: Span,
        source: Vec<ast::ParamFlagItem>,
    ) {
        for f in source.into_iter().rev() {
            match f.kind {
                ast::ParamFlagItemKind::In => {
                    dedup_flag!(source_span, self.errs, *target, hir::FuncParamFlags::IN);
                }
                ast::ParamFlagItemKind::Out => {
                    dedup_flag!(source_span, self.errs, *target, hir::FuncParamFlags::OUT);
                }
                ast::ParamFlagItemKind::Optional => {
                    dedup_flag!(
                        source_span,
                        self.errs,
                        *target,
                        hir::FuncParamFlags::OPTIONAL
                    );
                }
            }
        }
    }

    fn lower_function_param(&mut self, p: ast::FuncParam) -> hir::FuncParam {
        let mut ret = hir::FuncParam {
            span: p.span,
            flags: hir::FuncParamFlags::empty(),
            param_type: self.ast_type_to_hir_type(p.param_type),
            name: p.name,
            init: p.init,
        };
        self.lower_function_param_flags(&mut ret.flags, p.span, p.flags);
        ret
    }

    fn lower_action(&mut self, a: &[Identifier]) -> hir::ActionFlags {
        let mut flags = hir::ActionFlags::empty();
        for i in a.iter().rev() {
            if i.symbol == intern_name("actor") {
                dedup_flag!(i.span, self.errs, flags, hir::ActionFlags::ACTOR);
            } else if i.symbol == intern_name("overlay") {
                dedup_flag!(i.span, self.errs, flags, hir::ActionFlags::OVERLAY);
            } else if i.symbol == intern_name("weapon") {
                dedup_flag!(i.span, self.errs, flags, hir::ActionFlags::WEAPON);
            } else if i.symbol == intern_name("item") {
                dedup_flag!(i.span, self.errs, flags, hir::ActionFlags::ITEM);
            } else {
                self.errs.push(ParsingError {
                    level: ParsingErrorLevel::Error,
                    msg: "invalid action qualifier".to_string(),
                    main_spans: vec1![i.span],
                    info_spans: vec![],
                });
            }
        }
        flags
    }

    pub fn lower_function_declaration(
        &mut self,
        f: ast::FunctionDeclaration,
    ) -> hir::FunctionDeclaration {
        let ast::FunctionDeclaration {
            doc_comment,
            span,
            name,
            constant,
            metadata,
            return_types,
            params,
            body,
        } = f;

        let mut flags = hir::FunctionFlags::empty();
        let mut deprecated = None;
        let mut version = None;
        let mut action = None;

        for m in metadata.into_iter().rev() {
            match m.kind {
                ast::DeclarationMetadataItemKind::Native => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::NATIVE);
                }
                ast::DeclarationMetadataItemKind::Static => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::STATIC);
                }
                ast::DeclarationMetadataItemKind::Private => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::PRIVATE);
                }
                ast::DeclarationMetadataItemKind::Protected => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::PROTECTED);
                }
                ast::DeclarationMetadataItemKind::Final => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::FINAL);
                }
                ast::DeclarationMetadataItemKind::Transient => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::TRANSIENT);
                }
                ast::DeclarationMetadataItemKind::Virtual => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::VIRTUAL);
                }
                ast::DeclarationMetadataItemKind::Override => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::OVERRIDE);
                }
                ast::DeclarationMetadataItemKind::Abstract => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::ABSTRACT);
                }
                ast::DeclarationMetadataItemKind::VarArg => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::VAR_ARG);
                }
                ast::DeclarationMetadataItemKind::UI => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::UI);
                }
                ast::DeclarationMetadataItemKind::Play => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::PLAY);
                }
                ast::DeclarationMetadataItemKind::ClearScope => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::CLEAR_SCOPE);
                }
                ast::DeclarationMetadataItemKind::VirtualScope => {
                    dedup_flag!(m.span, self.errs, flags, hir::FunctionFlags::VIRTUAL_SCOPE);
                }

                ast::DeclarationMetadataItemKind::Deprecated { version, message } => {
                    if deprecated.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let version = parse_lump_version(&version.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        deprecated = Some(hir::Deprecated { version, message });
                    }
                }
                ast::DeclarationMetadataItemKind::Version(v) => {
                    if version.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let v = parse_lump_version(&v.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        version = Some(v);
                    }
                }
                ast::DeclarationMetadataItemKind::Action(l) => {
                    if action.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        match l {
                            Some(l) => {
                                action = Some(self.lower_action(&l));
                            }
                            None => {
                                action = Some(hir::ActionFlags::empty());
                            }
                        }
                    }
                }

                _ => self.errs.push(ParsingError {
                    level: ParsingErrorLevel::Error,
                    msg: "disallowed metadata item for function".to_string(),
                    main_spans: vec1![m.span],
                    info_spans: vec![],
                }),
            }
        }

        let return_types = self.lower_return_types(return_types);

        let params = match params.kind {
            ast::FuncParamsKind::Void => hir::FuncParams {
                span: params.span,
                args: vec![],
                variadic: false,
            },
            ast::FuncParamsKind::List { args, variadic } => {
                let mut found_init = None;
                hir::FuncParams {
                    span: params.span,
                    args: args
                        .into_iter()
                        .map(|p| {
                            let mut param = self.lower_function_param(p);
                            if param.init.is_none() && found_init.is_some() {
                                self.errs.push(ParsingError {
                                    level: ParsingErrorLevel::Error,
                                    msg:
                                        "parameter without a default value after one with a default value"
                                        .to_string(),
                                    main_spans: vec1![param.span],
                                    info_spans: vec![found_init.unwrap()]
                                });
                                param.init = Some(Expression {
                                    span: None,
                                    kind: ExpressionKind::Unknown,
                                });
                            }
                            if param.init.is_some() {
                                found_init = Some(param.span);
                            }
                            param
                        })
                        .collect(),
                    variadic,
                }
            }
        };

        hir::FunctionDeclaration {
            doc_comment,
            span,
            name,
            constant,
            flags,
            deprecated,
            version: version.unwrap_or_else(|| VersionInfo::new(0, 0, 0)),
            action,
            return_types,
            params,
            body: body.map(|c| self.lower_compound_statement(c)),
        }
    }

    pub fn lower_member_declaration(
        &mut self,
        mem: ast::MemberDeclaration,
        for_struct: bool,
    ) -> impl Iterator<Item = hir::MemberDeclaration> {
        let ast::MemberDeclaration {
            doc_comment,
            span,
            metadata,
            member_type,
            vars,
        } = mem;
        let base_member_type = match member_type.kind {
            ast::TypeListOrVoidKind::TypeList(l) if l.len() == 1 => {
                let k = l.into_iter().next().unwrap().kind;
                match k {
                    ast::TypeOrArrayKind::Type(t) => self.ast_type_to_hir_type(t),
                    ast::TypeOrArrayKind::Array(t, e) => {
                        let t = self.ast_type_to_hir_type(t);
                        lower_array_type(t, e)
                    }
                }
            }
            _ => {
                self.errs.push(ParsingError {
                    level: ParsingErrorLevel::Error,
                    msg: "member declarations must have only one non-void type".to_string(),
                    main_spans: vec1![member_type.span],
                    info_spans: vec![],
                });
                hir::Type::Error
            }
        };

        let mut flags = hir::MemberFlags::empty();
        let mut deprecated = None;
        let mut version = None;

        for m in metadata.into_iter().rev() {
            match m.kind {
                ast::DeclarationMetadataItemKind::Native => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::NATIVE);
                }
                ast::DeclarationMetadataItemKind::Private => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::PRIVATE);
                }
                ast::DeclarationMetadataItemKind::Protected => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::PROTECTED);
                }
                ast::DeclarationMetadataItemKind::Transient => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::TRANSIENT);
                }
                ast::DeclarationMetadataItemKind::ReadOnly => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::READ_ONLY);
                }
                ast::DeclarationMetadataItemKind::Internal => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::INTERNAL);
                }
                ast::DeclarationMetadataItemKind::VarArg => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::VAR_ARG);
                }
                ast::DeclarationMetadataItemKind::UI => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::UI);
                }
                ast::DeclarationMetadataItemKind::Play => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::PLAY);
                }

                ast::DeclarationMetadataItemKind::Meta if !for_struct => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::META);
                }
                ast::DeclarationMetadataItemKind::ClearScope if self.base_archive => {
                    dedup_flag!(m.span, self.errs, flags, hir::MemberFlags::CLEAR_SCOPE);
                }

                ast::DeclarationMetadataItemKind::Version(v) => {
                    if version.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let v = parse_lump_version(&v.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        version = Some(v);
                    }
                }
                ast::DeclarationMetadataItemKind::Deprecated { version, message } => {
                    if deprecated.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let version = parse_lump_version(&version.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        deprecated = Some(hir::Deprecated { version, message });
                    }
                }

                _ => self.errs.push(ParsingError {
                    level: ParsingErrorLevel::Error,
                    msg: "disallowed metadata item for member".to_string(),
                    main_spans: vec1![m.span],
                    info_spans: vec![],
                }),
            }
        }

        vars.into_iter().map(move |(name, sizes)| {
            let new_member_type = match sizes {
                Some(sizes) => lower_array_type(base_member_type.clone(), sizes),
                None => base_member_type.clone(),
            };
            hir::MemberDeclaration {
                doc_comment,
                span,
                member_type: new_member_type,

                flags,
                deprecated,
                version: version.unwrap_or_else(|| VersionInfo::new(0, 0, 0)),

                name,
            }
        })
    }

    fn lower_class_inners(
        &mut self,
        definitions: &HashMap<NameSymbol, Vec1<hir::TopLevelDefinition>>,
        inners_ast: impl Iterator<Item = ast::ClassInnerKind>,
        inners_hir: &mut HashMap<NameSymbol, Vec1<hir::ClassInner>>,
        defaults: &mut Vec<DefaultStatement>,
        states: &mut Vec<hir::StatesItem>,
    ) {
        for i in inners_ast {
            let hir_class_inners = match i {
                ast::ClassInnerKind::Declaration(d) => match d {
                    ast::Declaration::Function(f) => vec![hir::ClassInner {
                        span: f.span,
                        kind: hir::ClassInnerKind::FunctionDeclaration(
                            self.lower_function_declaration(f),
                        ),
                    }],
                    ast::Declaration::Member(m) => self
                        .lower_member_declaration(m, false)
                        .map(|x| hir::ClassInner {
                            span: x.span,
                            kind: hir::ClassInnerKind::MemberDeclaration(x),
                        })
                        .collect(),
                },
                ast::ClassInnerKind::Enum(e) => {
                    vec![hir::ClassInner {
                        span: e.span,
                        kind: hir::ClassInnerKind::Enum(e),
                    }]
                }
                ast::ClassInnerKind::Struct(s) => {
                    vec![hir::ClassInner {
                        span: s.span,
                        kind: hir::ClassInnerKind::Struct(self.lower_struct(s)),
                    }]
                }
                ast::ClassInnerKind::Const(c) => {
                    vec![hir::ClassInner {
                        span: c.span,
                        kind: hir::ClassInnerKind::Const(c),
                    }]
                }
                ast::ClassInnerKind::Property(p) => {
                    vec![hir::ClassInner {
                        span: p.span,
                        kind: hir::ClassInnerKind::Property(p),
                    }]
                }
                ast::ClassInnerKind::Flag(f) => {
                    vec![hir::ClassInner {
                        span: f.span,
                        kind: hir::ClassInnerKind::Flag(f),
                    }]
                }
                ast::ClassInnerKind::StaticConstArray(sca) => {
                    vec![hir::ClassInner {
                        span: sca.span,
                        kind: hir::ClassInnerKind::StaticConstArray(
                            self.lower_static_const_array(sca),
                        ),
                    }]
                }
                ast::ClassInnerKind::Mixin(m) => {
                    let mut defs = definitions.get(&m.symbol).map(|ds| {
                        ds.iter().filter_map(|d| {
                            if let hir::TopLevelDefinitionKind::MixinClass(m) = &d.kind {
                                if d.archive_num <= self.archive_num {
                                    Some(m)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    });
                    let mixin = if let Some(Some(d)) = defs.as_mut().map(|ds| ds.next()) {
                        let mut defs = defs.unwrap();
                        // if conflicting mixins exist then choose neither (for now)
                        // TODO: maybe review this in the future to check if merging the mixins
                        // but with types of everything set to Error to avoid errors from missing
                        // fields in other places is a good idea
                        if defs.next().is_some() {
                            continue;
                        }
                        d
                    } else {
                        self.errs.push(ParsingError {
                            level: ParsingErrorLevel::Error,
                            msg: format!("cannot find mixin `{}`", &*m.symbol.string()),
                            main_spans: vec1![m.span],
                            info_spans: vec![],
                        });
                        continue;
                    };
                    for (s, is) in mixin.inners.iter() {
                        use std::collections::hash_map::Entry;
                        match inners_hir.entry(*s) {
                            Entry::Occupied(mut oe) => {
                                oe.get_mut().extend(is.iter().cloned());
                            }
                            Entry::Vacant(ve) => {
                                ve.insert(is.clone());
                            }
                        }
                    }
                    defaults.extend(mixin.defaults.iter().cloned());
                    states.extend(mixin.states.iter().cloned());
                    continue;
                }

                ast::ClassInnerKind::States(s) => {
                    let action_flags = s.opts.map(|o| self.lower_action(&o));
                    for i in s.body {
                        states.push(hir::StatesItem {
                            span: i.span,
                            kind: match i.kind {
                                ast::StatesBodyItemKind::Label(l) => hir::StatesItemKind::Label(l),
                                ast::StatesBodyItemKind::Line(l) => {
                                    let mut flags = hir::StateLineFlags::empty();
                                    let mut offset = None;
                                    let mut light = None;
                                    for m in l.metadata.into_iter().rev() {
                                        match m.kind {
                                            ast::StateLineMetadataItemKind::Bright => {
                                                dedup_flag!(
                                                    m.span,
                                                    self.errs,
                                                    flags,
                                                    hir::StateLineFlags::BRIGHT
                                                )
                                            }
                                            ast::StateLineMetadataItemKind::Fast => {
                                                dedup_flag!(
                                                    m.span,
                                                    self.errs,
                                                    flags,
                                                    hir::StateLineFlags::FAST
                                                )
                                            }
                                            ast::StateLineMetadataItemKind::Slow => {
                                                dedup_flag!(
                                                    m.span,
                                                    self.errs,
                                                    flags,
                                                    hir::StateLineFlags::SLOW
                                                )
                                            }
                                            ast::StateLineMetadataItemKind::NoDelay => {
                                                dedup_flag!(
                                                    m.span,
                                                    self.errs,
                                                    flags,
                                                    hir::StateLineFlags::NO_DELAY
                                                )
                                            }
                                            ast::StateLineMetadataItemKind::CanRaise => {
                                                dedup_flag!(
                                                    m.span,
                                                    self.errs,
                                                    flags,
                                                    hir::StateLineFlags::CAN_RAISE
                                                )
                                            }
                                            ast::StateLineMetadataItemKind::Offset(e0, e1) => {
                                                if offset.is_some() {
                                                    self.errs.push(shadow_warning(m.span));
                                                } else {
                                                    offset = Some((e0, e1));
                                                }
                                            }
                                            ast::StateLineMetadataItemKind::Light(s) => {
                                                if light.is_some() {
                                                    self.errs.push(shadow_warning(m.span));
                                                } else {
                                                    light = Some(s);
                                                }
                                            }
                                        }
                                    }
                                    hir::StatesItemKind::Line(Box::new(hir::StateLine {
                                        span: l.span,
                                        sprite: l.sprite,
                                        frames: l.frames,
                                        duration: l.duration,
                                        flags,
                                        action_flags,
                                        offset,
                                        light,
                                        action: l.action.map(|a| hir::StateLineAction {
                                            span: a.span,
                                            kind: match a.kind {
                                                ast::StateLineActionKind::Call { func, args } => {
                                                    hir::StateLineActionKind::Call { func, args }
                                                }
                                                ast::StateLineActionKind::Anonymous(c) => {
                                                    hir::StateLineActionKind::Anonymous(
                                                        self.lower_compound_statement(c),
                                                    )
                                                }
                                            },
                                        }),
                                    }))
                                }
                                ast::StatesBodyItemKind::Stop => hir::StatesItemKind::Stop,
                                ast::StatesBodyItemKind::Wait => hir::StatesItemKind::Wait,
                                ast::StatesBodyItemKind::Fail => hir::StatesItemKind::Fail,
                                ast::StatesBodyItemKind::Loop => hir::StatesItemKind::Loop,
                                ast::StatesBodyItemKind::Goto { target, offset } => {
                                    hir::StatesItemKind::Goto {
                                        target: hir::StateGotoTarget {
                                            span: target.span,
                                            kind: match target.kind {
                                                ast::StateGotoTargetKind::Unscoped(u) => {
                                                    hir::StateGotoTargetKind::Unscoped(u)
                                                }
                                                ast::StateGotoTargetKind::Scoped(i, p) => {
                                                    hir::StateGotoTargetKind::Scoped(i, p)
                                                }
                                                ast::StateGotoTargetKind::Super(s) => {
                                                    hir::StateGotoTargetKind::Super(s)
                                                }
                                            },
                                        },
                                        offset,
                                    }
                                }
                            },
                        });
                    }
                    continue;
                }
                ast::ClassInnerKind::Default(ds) => {
                    for d in ds.statements {
                        defaults.push(d);
                    }
                    continue;
                }
            };
            for new in hir_class_inners {
                use std::collections::hash_map::Entry;
                match inners_hir.entry(new.name().symbol) {
                    Entry::Occupied(mut oe) => {
                        oe.get_mut().push(new);
                    }
                    Entry::Vacant(ve) => {
                        ve.insert(vec1![new]);
                    }
                }
            }
        }
    }

    fn lower_class(
        &mut self,
        definitions: &HashMap<NameSymbol, Vec1<hir::TopLevelDefinition>>,
        c: ast::ClassDefinition,
    ) -> hir::ClassDefinition {
        let mut ret = hir::ClassDefinition {
            doc_comment: c.doc_comment,
            span: c.span,
            ancestor: c.ancestor.map(|d| {
                if d.ids.len() > 1 {
                    self.errs.push(ParsingError {
                        level: ParsingErrorLevel::Error,
                        msg: "qualified name not allowed as ancestor".to_string(),
                        main_spans: vec1![d.span],
                        info_spans: vec![],
                    });
                }
                d.ids[0]
            }),
            name: c.name,

            flags: hir::ClassDefinitionFlags::empty(),
            version: None,
            replaces: None,

            inners: HashMap::new(),
            states: vec![],
            defaults: vec![],
        };

        for m in c.metadata.into_iter().rev() {
            match m.kind {
                ast::ClassMetadataItemKind::Abstract => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::ClassDefinitionFlags::ABSTRACT
                    );
                }
                ast::ClassMetadataItemKind::Native => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::ClassDefinitionFlags::NATIVE
                    );
                }
                ast::ClassMetadataItemKind::UI => {
                    dedup_flag!(m.span, self.errs, ret.flags, hir::ClassDefinitionFlags::UI);
                }
                ast::ClassMetadataItemKind::Play => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::ClassDefinitionFlags::PLAY
                    );
                }

                ast::ClassMetadataItemKind::Version(v) => {
                    if ret.version.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let v = parse_lump_version(&v.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        ret.version = Some(v);
                    }
                }
                ast::ClassMetadataItemKind::Replaces(r) => {
                    if ret.replaces.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        ret.replaces = Some(r);
                    }
                }
            }
        }

        self.lower_class_inners(
            definitions,
            c.inners.into_iter().map(|i| i.kind),
            &mut ret.inners,
            &mut ret.defaults,
            &mut ret.states,
        );

        ret
    }

    fn lower_mixin_class(
        &mut self,
        mixin: &ast::MixinClassDefinition,
    ) -> hir::MixinClassDefinition {
        let mut ret = hir::MixinClassDefinition {
            doc_comment: mixin.doc_comment,
            span: mixin.span,
            name: mixin.name,
            states: vec![],
            defaults: vec![],
            inners: HashMap::new(),
        };
        self.lower_class_inners(
            &HashMap::new(),
            mixin
                .inners
                .iter()
                .cloned()
                .map(|i| i.kind.map_to_class_inner_kind()),
            &mut ret.inners,
            &mut ret.defaults,
            &mut ret.states,
        );
        ret
    }

    fn lower_struct_inners(
        &mut self,
        inners_ast: impl Iterator<Item = ast::StructInnerKind>,
        inners_hir: &mut HashMap<NameSymbol, Vec1<hir::StructInner>>,
    ) {
        for i in inners_ast {
            let hir_struct_inners = match i {
                ast::StructInnerKind::Declaration(d) => match d {
                    ast::Declaration::Function(f) => vec![hir::StructInner {
                        span: f.span,
                        kind: hir::StructInnerKind::FunctionDeclaration(
                            self.lower_function_declaration(f),
                        ),
                    }],
                    ast::Declaration::Member(m) => self
                        .lower_member_declaration(m, false)
                        .map(|x| hir::StructInner {
                            span: x.span,
                            kind: hir::StructInnerKind::MemberDeclaration(x),
                        })
                        .collect(),
                },
                ast::StructInnerKind::Enum(e) => {
                    vec![hir::StructInner {
                        span: e.span,
                        kind: hir::StructInnerKind::Enum(e),
                    }]
                }
                ast::StructInnerKind::Const(c) => {
                    vec![hir::StructInner {
                        span: c.span,
                        kind: hir::StructInnerKind::Const(c),
                    }]
                }
                ast::StructInnerKind::StaticConstArray(sca) => {
                    vec![hir::StructInner {
                        span: sca.span,
                        kind: hir::StructInnerKind::StaticConstArray(
                            self.lower_static_const_array(sca),
                        ),
                    }]
                }
            };
            for new in hir_struct_inners {
                use std::collections::hash_map::Entry;
                match inners_hir.entry(new.name().symbol) {
                    Entry::Occupied(mut oe) => {
                        oe.get_mut().push(new);
                    }
                    Entry::Vacant(ve) => {
                        ve.insert(vec1![new]);
                    }
                }
            }
        }
    }

    fn lower_struct(&mut self, s: ast::StructDefinition) -> hir::StructDefinition {
        let mut ret = hir::StructDefinition {
            doc_comment: s.doc_comment,
            span: s.span,
            name: s.name,

            flags: hir::StructDefinitionFlags::empty(),
            version: None,

            inners: HashMap::new(),
        };

        for m in s.metadata.into_iter().rev() {
            match m.kind {
                ast::StructMetadataItemKind::Native => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::StructDefinitionFlags::NATIVE
                    );
                }
                ast::StructMetadataItemKind::UI => {
                    dedup_flag!(m.span, self.errs, ret.flags, hir::StructDefinitionFlags::UI);
                }
                ast::StructMetadataItemKind::Play => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::StructDefinitionFlags::PLAY
                    );
                }
                ast::StructMetadataItemKind::ClearScope => {
                    dedup_flag!(
                        m.span,
                        self.errs,
                        ret.flags,
                        hir::StructDefinitionFlags::CLEAR_SCOPE
                    )
                }

                ast::StructMetadataItemKind::Version(v) => {
                    if ret.version.is_some() {
                        self.errs.push(shadow_warning(m.span));
                    } else {
                        let v = parse_lump_version(&v.symbol.string())
                            .unwrap_or_else(|| VersionInfo::new(65535, 0, 0));
                        ret.version = Some(v);
                    }
                }
            }
        }

        self.lower_struct_inners(s.inners.into_iter().map(|i| i.kind), &mut ret.inners);

        ret
    }

    fn lower_system_pre_collect(
        &mut self,
        system: &FileSystemParseResult,
        definitions: &mut HashMap<NameSymbol, Vec1<hir::TopLevelDefinition>>,
    ) {
        for FileIndexAndAst { ast, .. } in system.asts.iter() {
            for d in ast.definitions.iter() {
                if let ast::TopLevelDefinitionKind::MixinClass(m) = &d.kind {
                    let new = self.lower_mixin_class(m);
                    let new = hir::TopLevelDefinition {
                        span: new.span,
                        kind: hir::TopLevelDefinitionKind::MixinClass(new),
                        archive_num: self.archive_num,
                    };
                    use std::collections::hash_map::Entry;
                    match definitions.entry(new.name().symbol) {
                        Entry::Occupied(mut oe) => {
                            oe.get_mut().push(new);
                        }
                        Entry::Vacant(ve) => {
                            ve.insert(vec1![new]);
                        }
                    }
                }
            }
        }
    }

    fn lower_system(
        &mut self,
        system: FileSystemParseResult,
        definitions: &mut HashMap<NameSymbol, Vec1<hir::TopLevelDefinition>>,
    ) {
        for FileIndexAndAst { ast, .. } in system.asts {
            for d in ast.definitions {
                if let Some(new) = match d.kind {
                    ast::TopLevelDefinitionKind::Class(c) => {
                        let new = self.lower_class(definitions, c);
                        Some(hir::TopLevelDefinition {
                            span: new.span,
                            kind: hir::TopLevelDefinitionKind::Class(new),
                            archive_num: self.archive_num,
                        })
                    }
                    ast::TopLevelDefinitionKind::Struct(s) => {
                        let new = self.lower_struct(s);
                        Some(hir::TopLevelDefinition {
                            span: new.span,
                            kind: hir::TopLevelDefinitionKind::Struct(new),
                            archive_num: self.archive_num,
                        })
                    }
                    ast::TopLevelDefinitionKind::ExtendClass(ec) => {
                        let cur_defs = definitions.get_mut(&ec.name.symbol);
                        if cur_defs.is_none() {
                            self.errs.push(ParsingError {
                                level: ParsingErrorLevel::Error,
                                msg: "extend target not defined (yet)".to_string(),
                                main_spans: vec1![ec.name.span],
                                info_spans: vec![],
                            });
                            continue;
                        }
                        let cur_defs = cur_defs.unwrap();
                        // if there are multiple things that might warrant extension then we'll
                        // just not do any of them
                        if cur_defs.len() > 1 {
                            continue;
                        }
                        let cur_def = &cur_defs[0];
                        if !matches!(&cur_def.kind, hir::TopLevelDefinitionKind::Class(_)) {
                            self.errs.push(ParsingError {
                                level: ParsingErrorLevel::Error,
                                msg: "`extend class` being used to extend non-class".to_string(),
                                main_spans: vec1![ec.name.span],
                                info_spans: vec![cur_def.name().span],
                            });
                            continue;
                        };
                        // the following is a bit of a borrow checker hack-around where we remove
                        // the class definition from the definitions map for a bit so that we can
                        // modify the definition while still giving an immutable borrow to
                        // lower_class_inners - it should be fast enough since removing from a
                        // hashmap isn't expensive but it's still non-ideal tbh lol
                        let v = definitions.remove(&ec.name.symbol).unwrap();
                        let mut item = v.into_vec().pop().unwrap();
                        let cur_class =
                            if let hir::TopLevelDefinitionKind::Class(c) = &mut item.kind {
                                c
                            } else {
                                unreachable!()
                            };
                        self.lower_class_inners(
                            definitions,
                            ec.inners.into_iter().map(|i| i.kind),
                            &mut cur_class.inners,
                            &mut cur_class.defaults,
                            &mut cur_class.states,
                        );
                        definitions.insert(ec.name.symbol, vec1![item]);
                        continue;
                    }
                    ast::TopLevelDefinitionKind::ExtendStruct(es) => {
                        let cur_defs = definitions.get_mut(&es.name.symbol);
                        if cur_defs.is_none() {
                            self.errs.push(ParsingError {
                                level: ParsingErrorLevel::Error,
                                msg: "extend target not defined (yet)".to_string(),
                                main_spans: vec1![es.name.span],
                                info_spans: vec![],
                            });
                            continue;
                        }
                        let cur_defs = cur_defs.unwrap();
                        // if there are multiple things that might warrant extension then we'll
                        // just not do any of them
                        if cur_defs.len() > 1 {
                            continue;
                        }
                        let cur_def = &mut cur_defs[0];
                        let cur_struct = match &mut cur_def.kind {
                            hir::TopLevelDefinitionKind::Struct(c) => c,
                            _ => {
                                self.errs.push(ParsingError {
                                    level: ParsingErrorLevel::Error,
                                    msg: "`extend struct` being used to extend non-struct"
                                        .to_string(),
                                    main_spans: vec1![es.name.span],
                                    info_spans: vec![cur_def.name().span],
                                });
                                continue;
                            }
                        };
                        self.lower_struct_inners(
                            es.inners.into_iter().map(|i| i.kind),
                            &mut cur_struct.inners,
                        );
                        continue;
                    }
                    ast::TopLevelDefinitionKind::Enum(e) => Some(hir::TopLevelDefinition {
                        span: e.span,
                        kind: hir::TopLevelDefinitionKind::Enum(e),
                        archive_num: self.archive_num,
                    }),
                    ast::TopLevelDefinitionKind::Const(c) => Some(hir::TopLevelDefinition {
                        span: c.span,
                        kind: hir::TopLevelDefinitionKind::Const(c),
                        archive_num: self.archive_num,
                    }),
                    ast::TopLevelDefinitionKind::Include(_)
                    | ast::TopLevelDefinitionKind::MixinClass(_) => None,
                } {
                    use std::collections::hash_map::Entry;
                    match definitions.entry(new.name().symbol) {
                        Entry::Occupied(mut oe) => {
                            oe.get_mut().push(new);
                        }
                        Entry::Vacant(ve) => {
                            ve.insert(vec1![new]);
                        }
                    }
                }
            }
        }
    }

    pub fn new(errs: &'outer mut Vec<ParsingError>) -> Self {
        Self {
            errs,
            base_archive: true,
            archive_num: 0,
        }
    }

    pub fn lower(mut self, systems: Vec<FileSystemParseResult>) -> HirLoweringResult {
        let mut definitions = HashMap::new();
        for (i, s) in systems.iter().enumerate() {
            self.base_archive = i == 0;
            self.archive_num = i;
            self.lower_system_pre_collect(s, &mut definitions);
        }
        for (i, s) in systems.into_iter().enumerate() {
            self.base_archive = i == 0;
            self.archive_num = i;
            self.lower_system(s, &mut definitions);
        }
        HirLoweringResult {
            hir: hir::TopLevel { definitions },
        }
    }
}
