use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    rc::Rc,
};

use thiserror::Error;

use crate::{
    shared::scanner::Token,
    walker::{
        ast::{Expr, Stmt},
        interpreter::Interpreter,
    },
};

#[derive(Error, Clone, Debug, PartialEq)]
enum ResolutionError {
    #[error("{msg}")]
    Error { msg: String },
}

type ResolverResult = Result<(), ResolutionError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeStack<'s>(Vec<Rc<RefCell<HashMap<&'s str, bool>>>>);

impl<'s> ScopeStack<'s> {
    fn push(&mut self) {
        self.0.push(Rc::new(RefCell::new(HashMap::new())))
    }

    fn pop(&mut self) {
        self.0.pop();
    }
}

struct Resolver<'s, 'io, I: Read, O: Write, E: Write> {
    interpreter: RefCell<Interpreter<'s, 'io, I, O, E>>,
    scopes: RefCell<ScopeStack<'s>>,
}

impl<'s, 'io, I: Read, O: Write, E: Write> Resolver<'s, 'io, I, O, E> {
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
                self.declare(name);
                self.define(name);

                self.scopes.borrow_mut().push();

                for token in params {
                    self.declare(token);
                    self.define(token);
                }

                for s in body {
                    self.resolve_statement(s)?
                }

                self.scopes.borrow_mut().pop();
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
                if let Some(v) = value {
                    self.resolve_expression(v)?;
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(i) = initializer {
                    self.resolve_expression(i)?;
                }
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
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
                    .map(|s| s.borrow().get(name.lexeme).cloned())
                    .flatten()
                {
                    return Err(ResolutionError::Error {
                        msg: "Cannot read local variable in its own initializer".to_string(),
                    });
                }

                self.resolve_local(expr, name);
            }
        }

        Ok(())
    }

    fn declare(&self, name: &'s Token<'s>) {
        self.scopes
            .borrow_mut()
            .0
            .last_mut()
            .map(|s| s.borrow_mut().insert(name.lexeme, false));
    }

    fn define(&self, name: &'s Token<'s>) {
        self.scopes
            .borrow_mut()
            .0
            .last_mut()
            .map(|s| s.borrow_mut().insert(name.lexeme, true));
    }

    fn resolve_local(&self, expr: &'s Expr<'s>, name: &'s Token<'s>) {
        let len = self.scopes.borrow().0.len();
        if let Some(depth) = self
            .scopes
            .borrow()
            .0
            .iter()
            .rposition(|s| s.borrow().contains_key(name.lexeme))
            .map(|index| len - index)
        {
            self.interpreter.borrow_mut().resolve(expr, depth)
        }
    }
}
