use xlang::prelude::v1::{Box, HashMap, None as XLangNone, Pair, Some as XLangSome, Vec};
use xlang_struct::{
    AggregateDefinition, BinaryOp, Block, BlockItem, BranchCondition, Expr, File,
    FunctionDeclaration, LValueOp, OverflowBehaviour, Path, PointerType, ScalarType,
    ScalarTypeHeader, ScalarTypeKind, StackItem, StackValueKind, StaticDefinition, Type, UnaryOp,
    Value,
};

struct TypeState {
    tys: HashMap<Path, Type>,
    aggregate: HashMap<Path, Option<AggregateDefinition>>,
}

impl TypeState {
    pub fn get_field_type<'a>(&'a self, ty: &'a Type, name: &str) -> Option<&'a Type> {
        match ty {
            Type::TaggedType(_, ty) | Type::Aligned(_, ty) => self.get_field_type(ty, name),
            Type::Product(tys) => {
                let v = name.parse::<usize>().unwrap();
                Some(&tys[v])
            }
            Type::Aggregate(n) => Some(
                n.fields
                    .iter()
                    .filter(|Pair(s, _)| s == name)
                    .map(|Pair(_, ty)| ty)
                    .next()
                    .unwrap(),
            ),
            Type::Named(p) => self.aggregate.get(p).map(Option::as_ref).map_or_else(
                || self.get_field_type(&self.tys[p], name),
                |ag| {
                    Some(
                        ag.unwrap()
                            .fields
                            .iter()
                            .filter(|Pair(s, _)| s == name)
                            .map(|Pair(_, ty)| ty)
                            .next()
                            .unwrap(),
                    )
                },
            ),
            _ => None,
        }
    }

    pub fn refiy_type<'a>(&'a self, ty: &'a Type) -> &'a Type {
        match ty {
            Type::Named(p) => self.tys.get(p).unwrap_or(ty),
            Type::TaggedType(_, ty) | Type::Aligned(_, ty) => ty,
            ty => ty,
        }
    }
}

fn tycheck_function(x: &mut FunctionDeclaration, tys: &TypeState) {
    if let FunctionDeclaration {
        ty,
        body: XLangSome(body),
    } = x
    {
        let local_tys = ty
            .params
            .iter()
            .cloned()
            .chain(body.locals.iter().cloned())
            .collect::<Vec<_>>();
        let ret = tycheck_block(&mut body.block, tys, &local_tys, &mut Vec::new());
        if ty.ret == Type::Void {
            assert_eq!(ret.len(), 0);
        } else {
            assert_eq!(ret.len(), 1);
            check_unify(&ret[0].ty, &ty.ret, tys);
        }
    }
}

fn check_unify(ty1: &Type, ty2: &Type, type_state: &TypeState) {
    match (type_state.refiy_type(ty1), type_state.refiy_type(ty2)) {
        (Type::Null, _) | (_, Type::Null) => {}
        (Type::Scalar(sty1), Type::Scalar(sty2)) => {
            assert_eq!(
                sty1.header.bitsize, sty2.header.bitsize,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );
            assert_eq!(
                sty1.header.vectorsize, sty2.header.vectorsize,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );

            match (sty1.kind, sty2.kind) {
                (
                    ScalarTypeKind::Integer {
                        signed: signed1, ..
                    },
                    ScalarTypeKind::Integer {
                        signed: signed2, ..
                    },
                ) => assert_eq!(
                    signed1, signed2,
                    "cannot unify types {:?} and {:?}",
                    ty1, ty2
                ),
                (
                    ScalarTypeKind::Fixed {
                        fractbits: fractbits1,
                    },
                    ScalarTypeKind::Fixed {
                        fractbits: fractbits2,
                    },
                ) => assert_eq!(
                    fractbits1, fractbits2,
                    "cannot unify types {:?} and {:?}",
                    ty1, ty2
                ),
                (
                    ScalarTypeKind::Char { flags: flags1 },
                    ScalarTypeKind::Char { flags: flags2 },
                ) => assert_eq!(flags1, flags2, "cannot unify types {:?} and {:?}", ty1, ty2),
                (
                    ScalarTypeKind::Float { decimal: dec1 },
                    ScalarTypeKind::Float { decimal: dec2 },
                ) => assert_eq!(dec1, dec2, "cannot unify types {:?} and {:?}", ty1, ty2),
                (ScalarTypeKind::LongFloat, ScalarTypeKind::LongFloat) => {}
                (_, _) => panic!("cannot unify types {:?} and {:?}", ty1, ty2),
            }
        }
        (Type::FnType(fnty1), Type::FnType(fnty2)) => {
            assert_eq!(
                fnty1.tag, fnty2.tag,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );

            assert_eq!(
                fnty1.variadic, fnty2.variadic,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );

            assert_eq!(
                fnty1.params.len(),
                fnty2.params.len(),
                "cannot unify types {:?} and {:?}",
                ty1,
                ty2
            );

            fnty1
                .params
                .iter()
                .zip(&fnty2.params)
                .for_each(|(ty1, ty2)| check_unify(ty1, ty2, type_state));
            check_unify(&fnty1.ret, &fnty2.ret, type_state);
        }
        (Type::Pointer(pty1), Type::Pointer(pty2)) => {
            check_unify(&pty1.inner, &pty2.inner, type_state)
        }
        (Type::Product(tys1), Type::Product(tys2)) => tys1
            .iter()
            .zip(tys2)
            .for_each(|(ty1, ty2)| check_unify(ty1, ty2, type_state)),
        (Type::Aggregate(defn1), Type::Aggregate(defn2)) => {
            assert_eq!(
                defn1.kind, defn2.kind,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );

            assert_eq!(
                defn1.annotations, defn2.annotations,
                "cannot unify types {:?} and {:?}",
                ty1, ty2
            );

            defn1.fields.iter().zip(&defn2.fields).for_each(
                |(Pair(name1, ty1), Pair(name2, ty2))| {
                    check_unify(ty1, ty2, type_state);
                    assert_eq!(name1, name2, "cannot unify types {:?} and {:?}", ty1, ty2);
                },
            );
        }
        (Type::Array(arr1), Type::Array(arr2)) => match (&arr1.len, &arr2.len) {
            (
                Value::Integer {
                    ty: sty1,
                    val: val1,
                },
                Value::Integer {
                    ty: sty2,
                    val: val2,
                },
            ) => {
                check_unify(&Type::Scalar(*sty1), &Type::Scalar(*sty2), type_state);
                assert_eq!(val1, val2, "Cannot unify types {:?} and {:?}", ty1, ty2);
            }
            (_, _) => panic!("Cannot unify types {:?} and {:?}", ty1, ty2),
        },
        (Type::Named(n1), Type::Named(n2)) => {
            assert_eq!(n1, n2, "Cannot unify types {:?} and {:?}", ty1, ty2)
        }
        (ty1, ty2) => panic!("Cannot unify types {:?} and {:?}", ty1, ty2),
    }
}

fn check_unify_stack(stack: &[StackItem], target: &[StackItem], tys: &TypeState) {
    let begin = stack.len() - target.len();
    let stack = &stack[begin..];

    for (a, b) in stack.iter().zip(target) {
        assert!(
            a.kind == b.kind,
            "Could not unify stack items {:?} and {:?}",
            a,
            b
        );
        check_unify(&a.ty, &b.ty, tys);
    }
}

#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::similar_names
)] // What does clippy want, for me to change the value kinds in xir
fn tycheck_expr(
    expr: &mut Expr,
    locals: &[Type],
    block_exits: &mut Vec<Option<Vec<StackItem>>>,
    vstack: &mut Vec<StackItem>,
    targets: &HashMap<u32, Vec<StackItem>>,
    tys: &TypeState,
) -> bool {
    match expr {
        xlang_struct::Expr::Const(v) => match v {
            xlang_struct::Value::Invalid(ty)
            | xlang_struct::Value::String { ty, .. }
            | xlang_struct::Value::Uninitialized(ty) => vstack.push(StackItem {
                ty: ty.clone(),
                kind: StackValueKind::RValue,
            }),
            xlang_struct::Value::GenericParameter(_) => todo!(),
            xlang_struct::Value::Integer { ty, .. } => vstack.push(StackItem {
                ty: Type::Scalar(*ty),
                kind: StackValueKind::RValue,
            }),
            xlang_struct::Value::GlobalAddress { ty, item } => {
                if let Type::Null = ty {
                    if let Some(t) = tys.tys.get(item) {
                        *ty = t.clone();
                    }
                }

                vstack.push(StackItem {
                    ty: Type::Pointer(PointerType {
                        inner: Box::new(ty.clone()),
                        ..Default::default()
                    }),
                    kind: StackValueKind::RValue,
                });
            }
            xlang_struct::Value::ByteString { .. } => vstack.push(StackItem {
                ty: Type::Pointer(PointerType {
                    inner: Box::new(Type::Scalar(ScalarType {
                        header: ScalarTypeHeader {
                            bitsize: 8,
                            ..Default::default()
                        },
                        kind: ScalarTypeKind::Integer {
                            signed: false,
                            min: XLangNone,
                            max: XLangNone,
                        },
                    })),
                    ..Default::default()
                }),
                kind: StackValueKind::RValue,
            }),
            xlang_struct::Value::LabelAddress(n) => {
                assert_eq!(targets[n].len(), 0);
                vstack.push(StackItem {
                    ty: Type::Pointer(PointerType {
                        inner: Box::new(Type::Void),
                        ..Default::default()
                    }),
                    kind: StackValueKind::RValue,
                });
            }
        },
        xlang_struct::Expr::ExitBlock { blk, values } => {
            let exit = &mut block_exits[*blk as usize];
            let pos = vstack.len().checked_sub(*values as usize).unwrap();
            let stack = vstack.split_off(pos);
            match exit {
                None => *exit = Some(stack),
                Some(items) => {
                    assert!(items.len() == (*values).into());
                    check_unify_stack(&stack, items, tys);
                }
            }
        }
        xlang_struct::Expr::BinaryOp(op, v) => {
            let val1 = vstack.pop().unwrap();
            let val2 = vstack.pop().unwrap();

            assert_eq!(val1.kind, StackValueKind::RValue);
            assert_eq!(val2.kind, StackValueKind::RValue);
            match tys.refiy_type(&val1.ty) {
                Type::Scalar(_) => match *op {
                    BinaryOp::CmpInt | BinaryOp::Cmp => vstack.push(StackItem {
                        ty: Type::Scalar(ScalarType {
                            header: ScalarTypeHeader {
                                bitsize: 32,
                                ..Default::default()
                            },
                            kind: ScalarTypeKind::Integer {
                                signed: true,
                                min: XLangNone,
                                max: XLangNone,
                            },
                        }),
                        kind: StackValueKind::RValue,
                    }),
                    BinaryOp::CmpLt
                    | BinaryOp::CmpLe
                    | BinaryOp::CmpGt
                    | BinaryOp::CmpGe
                    | BinaryOp::CmpNe
                    | BinaryOp::CmpEq => {
                        vstack.push(StackItem {
                            ty: Type::Scalar(ScalarType {
                                header: ScalarTypeHeader {
                                    bitsize: 1,
                                    ..Default::default()
                                },
                                kind: ScalarTypeKind::Integer {
                                    signed: false,
                                    min: XLangNone,
                                    max: XLangNone,
                                },
                            }),
                            kind: StackValueKind::RValue,
                        });
                    }
                    _ => match *v {
                        OverflowBehaviour::Checked => {
                            vstack.push(val1);
                            vstack.push(StackItem {
                                ty: Type::Scalar(ScalarType {
                                    header: ScalarTypeHeader {
                                        bitsize: 1,
                                        ..Default::default()
                                    },
                                    kind: ScalarTypeKind::Integer {
                                        signed: false,
                                        min: XLangNone,
                                        max: XLangNone,
                                    },
                                }),
                                kind: StackValueKind::RValue,
                            });
                        }
                        _ => vstack.push(val1),
                    },
                },
                x => todo!("{} {:?}", op, x),
            }
        }
        xlang_struct::Expr::UnaryOp(op, v) => {
            let val = vstack.pop().unwrap();

            assert_eq!(
                val.kind,
                StackValueKind::RValue,
                "Cannot apply {:?} to {:?}",
                op,
                val
            );

            match &val.ty {
                Type::Scalar(_) => match *op {
                    UnaryOp::Minus => {
                        vstack.push(val);
                        match *v {
                            OverflowBehaviour::Checked => {
                                vstack.push(StackItem {
                                    ty: Type::Scalar(ScalarType {
                                        header: ScalarTypeHeader {
                                            bitsize: 1,
                                            ..Default::default()
                                        },
                                        kind: ScalarTypeKind::Integer {
                                            signed: false,
                                            min: XLangNone,
                                            max: XLangNone,
                                        },
                                    }),
                                    kind: StackValueKind::RValue,
                                });
                            }
                            _ => {}
                        }
                    }
                    UnaryOp::LogicNot => {
                        vstack.push(StackItem {
                            ty: Type::Scalar(ScalarType {
                                header: ScalarTypeHeader {
                                    bitsize: 1,
                                    ..Default::default()
                                },
                                kind: ScalarTypeKind::Integer {
                                    signed: false,
                                    min: XLangNone,
                                    max: XLangNone,
                                },
                            }),
                            kind: StackValueKind::RValue,
                        });
                    }
                    UnaryOp::BitNot => {
                        vstack.push(val);
                    }
                    op => panic!("Cannot apply {:?}", op),
                },
                ty => panic!("Cannot apply {:?} to {:?}", op, ty),
            }
        }
        xlang_struct::Expr::Tailcall(_) => todo!("tailcall"),
        xlang_struct::Expr::CallFunction(fnty) => {
            let params = vstack.split_off_back(fnty.params.len());
            let dest = vstack.pop().unwrap();

            assert!(!fnty.variadic, "Cannot call with variadic signature");
            assert_eq!(dest.kind, StackValueKind::RValue, "Cannot call {:?}", dest);

            let destty = dest.ty;

            match tys.refiy_type(&destty) {
                Type::Pointer(ty) => match tys.refiy_type(&ty.inner) {
                    Type::FnType(ty) => {
                        if ty.variadic {
                            assert!(ty.params.len() <= fnty.params.len());
                        } else {
                            assert!(fnty.params.len() == fnty.params.len());
                        }

                        ty.params
                            .iter()
                            .zip(&fnty.params)
                            .for_each(|(ty1, ty2)| check_unify(ty1, ty2, tys));

                        check_unify(&ty.ret, &fnty.ret, tys);

                        params.iter().zip(&fnty.params).for_each(|(ty1, ty2)| {
                            assert_eq!(
                                ty1.kind,
                                StackValueKind::RValue,
                                "Cannot pass {:?} to {:?}",
                                ty1,
                                fnty
                            );
                            check_unify(&ty1.ty, ty2, tys);
                        });
                    }
                    ty => panic!("Cannot call {:?}", ty),
                },
                ty => panic!("Cannot call {:?}", ty),
            }

            vstack.push(StackItem {
                kind: StackValueKind::RValue,
                ty: fnty.ret.clone(),
            });
        }
        xlang_struct::Expr::Branch { cond, target } => {
            match *cond {
                BranchCondition::Always | BranchCondition::Never => {}
                _ => {
                    let ctrl = vstack.pop().unwrap();
                    assert_eq!(
                        ctrl.kind,
                        StackValueKind::RValue,
                        "Cannot branch on {:?}",
                        ctrl
                    );

                    match tys.refiy_type(&ctrl.ty) {
                        Type::Scalar(ScalarType {
                            kind: ScalarTypeKind::Integer { .. },
                            ..
                        }) => {}
                        ty => panic!("Cannot branch on {:?}", ty),
                    }
                }
            }

            let tstack = &targets[target];
            check_unify_stack(vstack, tstack, tys);
        }
        xlang_struct::Expr::BranchIndirect => {
            let target = vstack.pop().unwrap();
            assert_eq!(
                target.kind,
                StackValueKind::RValue,
                "Cannot branch to {:?}",
                target
            );

            check_unify(
                &target.ty,
                &Type::Pointer(PointerType {
                    inner: Box::new(Type::Void),
                    ..Default::default()
                }),
                tys,
            );
        }
        xlang_struct::Expr::Convert(_, _) => todo!("convert"),
        xlang_struct::Expr::Derive(pty, expr) => {
            tycheck_expr(expr, locals, block_exits, vstack, targets, tys);
            let mut ptr = vstack.pop().unwrap();
            assert_eq!(ptr.kind, StackValueKind::RValue);
            match &mut ptr.ty {
                Type::Pointer(ptrty) => {
                    check_unify(&pty.inner, &ptrty.inner, tys);
                    ptrty.alias = pty.alias;
                    ptrty.valid_range = pty.valid_range;
                    ptrty.decl = pty.decl;
                }
                ty => panic!("Cannot apply derive to {:?}", ty),
            }
        }
        xlang_struct::Expr::Local(n) => vstack.push(StackItem {
            ty: locals[(*n) as usize].clone(),
            kind: StackValueKind::LValue,
        }),
        xlang_struct::Expr::Pop(n) => {
            let back = vstack.len().checked_sub((*n) as usize).unwrap();
            vstack.shrink(back);
        }
        xlang_struct::Expr::Dup(n) => {
            let back = vstack.len().checked_sub((*n) as usize).unwrap();
            let stack = vstack.split_off(back);
            vstack.extend(stack.clone());
            vstack.extend(stack.clone());
        }
        xlang_struct::Expr::Pivot(n, m) => {
            let back1 = vstack.len().checked_sub((*m) as usize).unwrap();
            let back2 = back1.checked_sub((*n) as usize).unwrap();
            let stack1 = vstack.split_off(back1);
            let stack2 = vstack.split_off(back2);
            vstack.extend(stack1);
            vstack.extend(stack2);
        }
        xlang_struct::Expr::Aggregate(ctor) => {
            let ty = &ctor.ty;
            let back = vstack.len().checked_sub(ctor.fields.len()).unwrap();
            let values = vstack.split_off(back);
            for (field, item) in ctor.fields.iter().zip(values) {
                assert_eq!(item.kind, StackValueKind::RValue);
                check_unify(tys.get_field_type(ty, field).unwrap(), &item.ty, tys);
            }

            vstack.push(StackItem {
                ty: ty.clone(),
                kind: StackValueKind::RValue,
            });
        }
        xlang_struct::Expr::Member(name) => {
            let val = vstack.pop().unwrap();

            assert_eq!(val.kind, StackValueKind::LValue);

            let ty = tys.get_field_type(&val.ty, name).unwrap();

            vstack.push(StackItem {
                ty: ty.clone(),
                kind: StackValueKind::LValue,
            });
        }
        xlang_struct::Expr::MemberIndirect(name) => {
            let mut val = vstack.pop().unwrap();

            assert_eq!(val.kind, StackValueKind::RValue);

            match &mut val.ty {
                Type::Pointer(ptr) => {
                    let ty = tys.get_field_type(&ptr.inner, name).unwrap().clone();

                    *ptr = PointerType {
                        inner: Box::new(ty),
                        ..Default::default()
                    };
                }
                ty => panic!("Cannot use member indirect {} on {:?}", name, ty),
            }

            vstack.push(val);
        }
        xlang_struct::Expr::Block { n, block } => {
            assert!((*n as usize) == block_exits.len());
            let res = tycheck_block(block, tys, locals, block_exits);

            vstack.extend(res);
        }
        xlang_struct::Expr::Assign(_) => {
            let rvalue = vstack.pop().unwrap();
            let lvalue = vstack.pop().unwrap();

            assert_eq!(rvalue.kind, StackValueKind::RValue);
            assert_eq!(lvalue.kind, StackValueKind::LValue);

            check_unify(&rvalue.ty, &lvalue.ty, tys);
        }
        xlang_struct::Expr::AsRValue(_) => {
            let val = vstack.pop().unwrap();
            assert_eq!(val.kind, StackValueKind::LValue);

            vstack.push(StackItem {
                ty: val.ty,
                kind: StackValueKind::RValue,
            });
        }
        xlang_struct::Expr::CompoundAssign(op, v, _) => {
            let rvalue = vstack.pop().unwrap();
            let lvalue = vstack.pop().unwrap();

            assert_eq!(rvalue.kind, StackValueKind::RValue);
            assert_eq!(lvalue.kind, StackValueKind::LValue);

            check_unify(&lvalue.ty, &rvalue.ty, tys);

            match tys.refiy_type(&lvalue.ty) {
                Type::Scalar(_) => {
                    if *v == OverflowBehaviour::Checked {
                        vstack.push(StackItem {
                            ty: Type::Scalar(ScalarType {
                                header: ScalarTypeHeader {
                                    bitsize: 1,
                                    ..Default::default()
                                },
                                kind: ScalarTypeKind::Integer {
                                    signed: false,
                                    min: XLangNone,
                                    max: XLangNone,
                                },
                            }),
                            kind: StackValueKind::RValue,
                        });
                    }
                }
                ty => todo!("compound_assign {}: {:?}", op, ty),
            }
        }
        xlang_struct::Expr::LValueOp(op, v, _) => match *op {
            LValueOp::Xchg => {
                let val1 = vstack.pop().unwrap();
                let val2 = vstack.pop().unwrap();

                assert_eq!(val1.kind, StackValueKind::LValue);
                assert_eq!(val2.kind, StackValueKind::LValue);

                check_unify(&val1.ty, &val2.ty, tys);
            }
            LValueOp::Cmpxchg | LValueOp::Wcmpxchg => {
                let control = vstack.pop().unwrap();
                let swap = vstack.pop().unwrap();
                let dest = vstack.pop().unwrap();

                assert_eq!(dest.kind, StackValueKind::LValue);
                assert_eq!(swap.kind, StackValueKind::LValue);
                assert_eq!(control.kind, StackValueKind::RValue);

                check_unify(&dest.ty, &control.ty, tys);
                check_unify(&dest.ty, &swap.ty, tys);

                vstack.push(StackItem {
                    ty: Type::Scalar(ScalarType {
                        header: ScalarTypeHeader {
                            bitsize: 1,
                            ..Default::default()
                        },
                        kind: ScalarTypeKind::Integer {
                            signed: false,
                            min: XLangNone,
                            max: XLangNone,
                        },
                    }),
                    kind: StackValueKind::RValue,
                });
            }
            LValueOp::PreDec | LValueOp::PreInc => {
                let lvalue = vstack.pop().unwrap();

                assert_eq!(lvalue.kind, StackValueKind::LValue);

                match tys.refiy_type(&lvalue.ty) {
                    Type::Scalar(_) => {
                        vstack.push(lvalue);
                        if *v == OverflowBehaviour::Checked {
                            vstack.push(StackItem {
                                ty: Type::Scalar(ScalarType {
                                    header: ScalarTypeHeader {
                                        bitsize: 1,
                                        ..Default::default()
                                    },
                                    kind: ScalarTypeKind::Integer {
                                        signed: false,
                                        min: XLangNone,
                                        max: XLangNone,
                                    },
                                }),
                                kind: StackValueKind::RValue,
                            });
                        }
                    }
                    ty => panic!("Cannot use {:?} on {:?}", op, ty),
                }
            }
            LValueOp::PostDec | LValueOp::PostInc => {
                let mut lvalue = vstack.pop().unwrap();

                assert_eq!(lvalue.kind, StackValueKind::LValue);

                match tys.refiy_type(&lvalue.ty) {
                    Type::Scalar(_) => {
                        if *v == OverflowBehaviour::Checked {
                            lvalue.kind = StackValueKind::RValue;
                            vstack.push(lvalue);
                            vstack.push(StackItem {
                                ty: Type::Scalar(ScalarType {
                                    header: ScalarTypeHeader {
                                        bitsize: 1,
                                        ..Default::default()
                                    },
                                    kind: ScalarTypeKind::Integer {
                                        signed: false,
                                        min: XLangNone,
                                        max: XLangNone,
                                    },
                                }),
                                kind: StackValueKind::RValue,
                            });
                        } else {
                            vstack.push(lvalue);
                        }
                    }
                    ty => panic!("Cannot use {:?} on {:?}", op, ty),
                }
            }
            op => todo!("{:?}", op),
        },
        xlang_struct::Expr::Indirect => {
            let mut val = vstack.pop().unwrap();

            assert_eq!(val.kind, StackValueKind::RValue);

            match val.ty {
                Type::Pointer(ptr) => {
                    val.ty = Box::into_inner(ptr.inner);
                    val.kind = StackValueKind::LValue;
                }
                ty => panic!("Cannot use indirect on {:?}", ty),
            }

            vstack.push(val);
        }
        xlang_struct::Expr::AddrOf => {
            let mut val = vstack.pop().unwrap();

            assert_eq!(val.kind, StackValueKind::LValue);

            val.ty = Type::Pointer(PointerType {
                inner: Box::new(val.ty),
                ..Default::default()
            });

            vstack.push(val);
        }
        xlang_struct::Expr::Null
        | xlang_struct::Expr::Sequence(_)
        | xlang_struct::Expr::Fence(_) => {}
        xlang_struct::Expr::Switch(switch) => {
            let ctrl = vstack.pop().unwrap();

            assert_eq!(
                ctrl.kind,
                StackValueKind::RValue,
                "Cannot switch on {:?}",
                ctrl
            );

            match tys.refiy_type(&ctrl.ty) {
                Type::Scalar(ScalarType {
                    kind: ScalarTypeKind::Integer { .. },
                    ..
                }) => {}
                ty => panic!("Cannot switch on {:?}", ty),
            }

            let mut ustack = None::<&Vec<StackItem>>;

            match switch {
                xlang_struct::Switch::Hash(h) => {
                    for Pair(val, targ) in &h.cases {
                        match val {
                            Value::Integer { ty, val: _ } => {
                                check_unify(&ctrl.ty, &Type::Scalar(*ty), tys);
                            }
                            val => panic!("Cannot switch on case {:?}", val),
                        }

                        let tstack = &targets[targ];

                        if let Some(stack) = ustack {
                            assert_eq!(
                                stack.len(),
                                tstack.len(),
                                "Cannot unify stack for target @{} ({:?}) with switch stack {:?}",
                                targ,
                                tstack,
                                stack
                            );
                            stack.iter().zip(tstack).for_each(|(ty1,ty2)| {
                                assert_eq!(ty1.kind,ty2.kind,"Cannot unify stack for target @{} ({:?}) with switch stack {:?}",targ,tstack,stack);
                                check_unify(&ty1.ty,&ty2.ty,tys);
                            })
                        } else {
                            check_unify_stack(vstack, tstack, tys);
                            ustack = Some(tstack);
                        }
                    }
                }
                xlang_struct::Switch::Linear(_) => todo!(),
            }
        }
    }
    false
}

fn tycheck_block(
    block: &mut Block,
    tys: &TypeState,
    locals: &[Type],
    block_exits: &mut Vec<Option<Vec<StackItem>>>,
) -> Vec<StackItem> {
    block_exits.push(None);
    let mut vstack = Vec::new();

    let mut targets = HashMap::<_, _>::new();
    let mut diverged = false;

    for item in &block.items {
        if let BlockItem::Target { num, stack } = item {
            assert!(
                targets.insert(*num, stack.clone()).is_none(),
                "Target @{} redeclared",
                num
            );
        }
    }

    for item in &mut block.items {
        match item {
            BlockItem::Expr(expr) => {
                diverged = tycheck_expr(expr, locals, block_exits, &mut vstack, &targets, tys);
            }
            BlockItem::Target { stack, .. } => {
                if !diverged {
                    check_unify_stack(&vstack, stack, tys);
                }
                diverged = false;
                vstack.clear();
                vstack.extend(stack.iter().cloned());
            }
        }
    }

    block_exits.pop().unwrap().unwrap_or_default()
}

pub fn tycheck(x: &mut File) {
    let mut typestate = TypeState {
        tys: HashMap::new(),
        aggregate: HashMap::new(),
    };

    // pass one, gather global address types
    for Pair(path, member) in &x.root.members {
        match &member.member_decl {
            xlang_struct::MemberDeclaration::Function(FunctionDeclaration { ty, .. }) => {
                typestate
                    .tys
                    .insert(path.clone(), Type::FnType(Box::new(ty.clone())));
            }
            xlang_struct::MemberDeclaration::Static(StaticDefinition { ty, .. }) => {
                typestate.tys.insert(path.clone(), ty.clone());
            }
            xlang_struct::MemberDeclaration::AggregateDefinition(defn) => {
                typestate.aggregate.insert(path.clone(), Some(defn.clone()));
            }
            xlang_struct::MemberDeclaration::OpaqueAggregate(_) => {
                typestate.aggregate.insert(path.clone(), None);
            }
            _ => {}
        }
    }

    for Pair(_, member) in &mut x.root.members {
        match &mut member.member_decl {
            xlang_struct::MemberDeclaration::Function(
                decl @ FunctionDeclaration {
                    body: XLangSome(_), ..
                },
            ) => {
                tycheck_function(decl, &typestate);
            }
            xlang_struct::MemberDeclaration::Static(_) => todo!(),
            _ => {}
        }
    }
}
