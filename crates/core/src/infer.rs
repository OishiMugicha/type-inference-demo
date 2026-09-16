use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ast::{BinaryOp, Expr, ExprKind as E},
    error::{Diagnostic, ErrorKind, LangResult, Stage},
    span::Span,
    types::{Type as T, TypeEnv, TypeScheme, TypeVar},
};

/// 空の型環境で、式の主型を推論する。型変数の番号は自由。
pub fn infer(expr: &Expr) -> LangResult<T> {
    let mut inference = Inference::default();
    let ty = inference.expression(expr, &TypeEnv::new())?;
    Ok(inference.resolve(&ty))
}

#[derive(Default)]
struct Inference {
    next: TypeVar,
    substitutions: BTreeMap<TypeVar, T>,
}

fn free_variables(ty: &T, variables: &mut BTreeSet<TypeVar>) {
    match ty {
        T::Var(id) => {
            variables.insert(*id);
        }
        T::Function(input, output) => {
            free_variables(input, variables);
            free_variables(output, variables);
        }
        T::Int | T::Bool => {}
    }
}

fn replace(ty: &T, replacements: &BTreeMap<TypeVar, T>) -> T {
    match ty {
        T::Var(id) => replacements.get(id).cloned().unwrap_or_else(|| ty.clone()),
        T::Function(input, output) => T::Function(
            Box::new(replace(input, replacements)),
            Box::new(replace(output, replacements)),
        ),
        _ => ty.clone(),
    }
}

impl Inference {
    fn fresh(&mut self) -> T {
        let id = self.next;
        self.next += 1;
        T::Var(id)
    }

    fn resolve(&self, ty: &T) -> T {
        match ty {
            T::Var(id) => self
                .substitutions
                .get(id)
                .map_or_else(|| ty.clone(), |ty| self.resolve(ty)),
            T::Function(input, output) => T::Function(
                Box::new(self.resolve(input)),
                Box::new(self.resolve(output)),
            ),
            _ => ty.clone(),
        }
    }

    fn unify(&mut self, left: &T, right: &T, span: Span) -> LangResult<()> {
        let left = self.resolve(left);
        let right = self.resolve(right);
        if left == right {
            return Ok(());
        }
        match (&left, &right) {
            (T::Var(id), ty) | (ty, T::Var(id)) => {
                let mut variables = BTreeSet::new();
                free_variables(ty, &mut variables);
                if variables.contains(id) {
                    return Err(Diagnostic::new(
                        Stage::Inference,
                        ErrorKind::InfiniteType,
                        "無限型が必要になる式です。",
                        Some(span),
                    ));
                }
                self.substitutions.insert(*id, ty.clone());
                Ok(())
            }
            (T::Function(a, b), T::Function(c, d)) => {
                self.unify(a, c, span)?;
                self.unify(b, d, span)
            }
            _ => Err(Diagnostic::new(
                Stage::Inference,
                ErrorKind::TypeMismatch,
                format!("型が一致しません: {left} と {right}"),
                Some(span),
            )),
        }
    }

    fn instantiate(&mut self, scheme: &TypeScheme) -> T {
        let replacements = scheme
            .quantified
            .iter()
            .map(|id| (*id, self.fresh()))
            .collect();
        replace(&scheme.ty, &replacements)
    }

    fn generalize(&self, ty: &T, env: &TypeEnv) -> TypeScheme {
        let ty = self.resolve(ty);
        let mut variables = BTreeSet::new();
        free_variables(&ty, &mut variables);
        for scheme in env.values() {
            let mut environment_variables = BTreeSet::new();
            free_variables(&self.resolve(&scheme.ty), &mut environment_variables);
            for id in &scheme.quantified {
                environment_variables.remove(id);
            }
            variables.retain(|id| !environment_variables.contains(id));
        }
        TypeScheme {
            quantified: variables.into_iter().collect(),
            ty,
        }
    }

    fn expression(&mut self, expr: &Expr, env: &TypeEnv) -> LangResult<T> {
        match &expr.kind {
            E::Int(_) => Ok(T::Int),
            E::Bool(_) => Ok(T::Bool),
            E::Var(name) => {
                let scheme = env.get(name).ok_or_else(|| {
                    Diagnostic::new(
                        Stage::Inference,
                        ErrorKind::UnboundVariable,
                        format!("未束縛の変数です: {name}"),
                        Some(expr.span),
                    )
                })?;
                Ok(self.instantiate(scheme))
            }
            E::Negate(inner) => {
                let ty = self.expression(inner, env)?;
                self.unify(&ty, &T::Int, inner.span)?;
                Ok(T::Int)
            }
            E::Binary { op, left, right } => {
                let a = self.expression(left, env)?;
                self.unify(&a, &T::Int, left.span)?;
                let b = self.expression(right, env)?;
                self.unify(&b, &T::Int, right.span)?;
                Ok(if *op == BinaryOp::Less {
                    T::Bool
                } else {
                    T::Int
                })
            }
            E::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.expression(condition, env)?;
                self.unify(&condition_type, &T::Bool, condition.span)?;
                let a = self.expression(then_branch, env)?;
                let b = self.expression(else_branch, env)?;
                self.unify(&a, &b, expr.span)?;
                Ok(self.resolve(&a))
            }
            E::Fun { parameter, body } => {
                let input = self.fresh();
                let mut local = env.clone();
                local.insert(
                    parameter.clone(),
                    TypeScheme {
                        quantified: vec![],
                        ty: input.clone(),
                    },
                );
                let output = self.expression(body, &local)?;
                Ok(T::Function(
                    Box::new(self.resolve(&input)),
                    Box::new(output),
                ))
            }
            E::Apply { function, argument } => {
                let function_type = self.expression(function, env)?;
                let argument_type = self.expression(argument, env)?;
                let output = self.fresh();
                self.unify(
                    &function_type,
                    &T::Function(Box::new(argument_type), Box::new(output.clone())),
                    expr.span,
                )?;
                Ok(self.resolve(&output))
            }
            E::Let { name, value, body } => {
                let ty = self.expression(value, env)?;
                let scheme = self.generalize(&ty, env);
                let mut local = env.clone();
                local.insert(name.clone(), scheme);
                self.expression(body, &local)
            }
        }
    }
}
