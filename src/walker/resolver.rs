use std::{cell::RefCell, collections::HashMap, rc::Rc};

use thiserror::Error;

use crate::{
    shared::scanner::Token,
    walker::ast::{Expr, Stmt},
};

#[derive(Error, Clone, Debug, PartialEq)]
pub enum ResolutionError {
    #[error("{msg}")]
    Error { msg: String },
}

pub type ResolverResult = Result<(), ResolutionError>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScopeStack<'s>(Vec<Rc<RefCell<HashMap<&'s str, bool>>>>);
pub type Locals<'s> = HashMap<&'s Expr<'s>, usize>;

#[derive(Debug, PartialEq)]
enum FunctionType {
    Function,
}

impl ScopeStack<'_> {
    fn push(&mut self) {
        self.0.push(Rc::new(RefCell::new(HashMap::new())))
    }

    fn pop(&mut self) {
        self.0.pop();
    }
}
#[derive(PartialEq, Debug, Default)]
struct Resolver<'s> {
    scopes: RefCell<ScopeStack<'s>>,
    locals: RefCell<Locals<'s>>,
    current_function_type: RefCell<Option<FunctionType>>,
}

impl<'s> Resolver<'s> {
    fn resolve_statement(&self, stmt: &'s Stmt<'s>) -> ResolverResult {
        match stmt {
            Stmt::Block { stmts } => {
                self.scopes.borrow_mut().push();

                for s in stmts {
                    self.resolve_statement(s)?
                }

                self.scopes.borrow_mut().pop();
            }
            Stmt::Break => {}
            Stmt::Expression { expr } => self.resolve_expression(expr)?,
            Stmt::Function { name, params, body } => {
                let enclosing_function_type = self
                    .current_function_type
                    .replace(Some(FunctionType::Function));

                self.declare(name)?;
                self.define(name);

                self.scopes.borrow_mut().push();

                for token in params {
                    self.declare(token)?;
                    self.define(token);
                }

                for s in body {
                    self.resolve_statement(s)?
                }

                self.scopes.borrow_mut().pop();

                self.current_function_type.replace(enclosing_function_type);
            }
            Stmt::If {
                condition,
                then,
                els,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then)?;
                if let Some(e) = els {
                    self.resolve_statement(e)?;
                }
            }
            Stmt::Print { expr } => {
                self.resolve_expression(expr)?;
            }
            Stmt::Return { value } => {
                if self.current_function_type.borrow().is_none() {
                    return Err(ResolutionError::Error {
                        msg: "Cannot return from global scope".into(),
                    });
                }

                if let Some(v) = value {
                    self.resolve_expression(v)?;
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(i) = initializer {
                    self.resolve_expression(i)?;
                }
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
            }
            Stmt::Class { name, .. } => {
                self.declare(name)?;
                self.define(name);
            }
        }

        Ok(())
    }

    fn resolve_expression(&self, expr: &'s Expr<'s>) -> ResolverResult {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(expr, name);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Call { callee, args } => {
                self.resolve_expression(callee)?;

                for a in args {
                    self.resolve_expression(a)?;
                }
            }
            Expr::Unary { right, .. } => {
                self.resolve_expression(right)?;
            }
            Expr::Grouping { expr } => {
                self.resolve_expression(expr)?;
            }
            Expr::Literal { .. } => {}
            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Variable { name } => {
                if let Some(false) = self
                    .scopes
                    .borrow()
                    .0
                    .last()
                    .and_then(|s| s.borrow().get(name.lexeme).cloned())
                {
                    return Err(ResolutionError::Error {
                        msg: "Cannot read local variable in its own initializer".to_string(),
                    });
                }

                self.resolve_local(expr, name);
            }
            Expr::Get { object, .. } => {
                self.resolve_expression(object)?;
            }
        }

        Ok(())
    }

    fn declare(&self, name: &'s Token<'s>) -> ResolverResult {
        self.scopes
            .borrow_mut()
            .0
            .last_mut()
            .map(|s| {
                let mut scope = s.borrow_mut();
                if scope.contains_key(name.lexeme) {
                    Err(ResolutionError::Error {
                        msg: format!("Variable {} was already defined in this scope", name.lexeme),
                    })
                } else {
                    scope.insert(name.lexeme, false);
                    Ok(())
                }
            })
            .unwrap_or(Ok(()))
    }

    fn define(&self, name: &'s Token<'s>) {
        self.scopes
            .borrow_mut()
            .0
            .last_mut()
            .map(|s| s.borrow_mut().insert(name.lexeme, true));
    }

    fn resolve_local(&self, expr: &'s Expr<'s>, name: &'s Token<'s>) {
        if let Some(depth) = self
            .scopes
            .borrow()
            .0
            .iter()
            .rposition(|s| s.borrow().contains_key(name.lexeme))
        {
            self.locals.borrow_mut().insert(expr, depth);
        }
    }

    fn locals(self) -> Locals<'s> {
        self.locals.into_inner()
    }
}

pub fn resolve<'s>(stmts: &'s [Stmt<'s>]) -> Result<Locals<'s>, ResolutionError> {
    let resolver = Resolver::default();

    for stmt in stmts {
        resolver.resolve_statement(stmt)?;
    }

    Ok(resolver.locals())
}
