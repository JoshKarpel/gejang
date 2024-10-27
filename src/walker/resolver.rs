use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

struct Resolver<'s, 'io, I, O, E> {
    interpreter: RefCell<Interpreter<'s, 'io, I, O, E>>,
    scopes: RefCell<ScopeStack<'s>>,
}

impl<'s, 'io, I, O, E> Resolver<'s, 'io, I, O, E> {
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
            Stmt::Expression { .. } => {}
            Stmt::Function { .. } => {}
            Stmt::If { .. } => {}
            Stmt::Print { .. } => {}
            Stmt::Return { .. } => {}
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(i) = initializer {
                    self.resolve_expression(i)?;
                }
                self.define(name);
            }
            Stmt::While { .. } => {}
        }

        Ok(())
    }

    fn resolve_expression(&self, expr: &'s Expr<'s>) -> ResolverResult {
        match expr {
            Expr::Assign { .. } => {}
            Expr::Binary { .. } => {}
            Expr::Call { .. } => {}
            Expr::Unary { .. } => {}
            Expr::Grouping { .. } => {}
            Expr::Literal { .. } => {}
            Expr::Logical { .. } => {}
            Expr::Variable { name } => {
                if let Some(false) = self
                    .scopes
                    .borrow()
                    .0
                    .last()
                    .map(|s| s.borrow().get(name.lexeme))
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
