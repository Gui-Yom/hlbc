use crate::decompiler::ast::{cst_int, Expr, Statement};

#[derive(Debug)]
pub(crate) enum ScopeType {
    Len(i32),
    Manual,
}

#[derive(Debug)]
pub(crate) enum ScopeData {
    Root,
    If {
        cond: Expr,
    },
    Else {
        if_cond: Expr,
        if_stmts: Vec<Statement>,
    },
    Switch {
        arg: Expr,
        offsets: Vec<usize>,
        cases: Vec<(Expr, Vec<Statement>)>,
    },
    SwitchCase {
        pattern: Expr,
    },
    Loop {
        start: usize,
        cond: Expr,
    },
    Try,
    Catch,
}

#[derive(Debug)]
pub(crate) struct Scope {
    pub(crate) ty: ScopeType,
    pub(crate) stmts: Vec<Statement>,
    pub(crate) data: ScopeData,
}

impl Scope {
    fn new(ty: ScopeType, data: ScopeData) -> Self {
        Self {
            ty,
            stmts: Vec::new(),
            data,
        }
    }

    /// Finish the scope by creating a statement from it
    pub(crate) fn make_stmt(self) -> Statement {
        match self.data {
            ScopeData::If { cond } => Statement::IfElse {
                cond,
                if_: self.stmts,
                else_: Vec::new(),
            },
            ScopeData::Else { if_cond, if_stmts } => Statement::IfElse {
                cond: if_cond,
                if_: if_stmts,
                else_: self.stmts,
            },
            ScopeData::Switch { arg, cases, .. } => Statement::Switch {
                arg,
                default: self.stmts,
                cases,
            },
            ScopeData::Loop { cond, .. } => Statement::While {
                cond,
                stmts: self.stmts,
            },
            ScopeData::Try => Statement::Try { stmts: self.stmts },
            ScopeData::Catch => Statement::Catch { stmts: self.stmts },
            _ => {
                unreachable!()
            }
        }
    }
}

/// Helper to process a stack of scopes (branches, loops)
pub(crate) struct Scopes {
    /// There is always at least one scope, the root scope
    pub(crate) scopes: Vec<Scope>,
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![Scope::new(ScopeType::Manual, ScopeData::Root)],
        }
    }

    pub(crate) fn push_stmt(&mut self, stmt: Statement) {
        self.scopes.last_mut().unwrap().stmts.push(stmt);
    }

    pub(crate) fn advance(&mut self) {
        let mut stmt = None;
        for i in (0..self.scopes.len()).rev() {
            if matches!(self.scopes[i].ty, ScopeType::Len(len) if len == 1) {
                let mut scope = self.scopes.remove(i);
                if let Some(stmt) = stmt.take() {
                    scope.stmts.push(stmt);
                }
                // Exception for Switch where a switch scope can be closed with a switch case open
                match &mut scope.data {
                    ScopeData::Switch { cases, .. } => {
                        let case = self.scopes.remove(i);
                        match case.data {
                            ScopeData::SwitchCase { pattern } => {
                                cases.push((pattern, case.stmts));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                stmt = Some(scope.make_stmt());
            } else {
                let scope = &mut self.scopes[i];
                if let Some(stmt) = stmt.take() {
                    scope.stmts.push(stmt);
                }
                match &mut scope.ty {
                    ScopeType::Len(len) => {
                        *len -= 1;
                    }
                    ScopeType::Manual => {}
                }
            }
        }
    }

    pub(crate) fn statements(mut self) -> Vec<Statement> {
        if let Some(Scope { stmts, data, .. }) = self.scopes.pop() {
            if matches!(data, ScopeData::Root) {
                stmts
            } else {
                panic!(
                    "Remaining scopes other than the root scope :\n{:#?}",
                    self.scopes
                );
            }
        } else {
            panic!("No remaining scopes ? Not even the root scope ?");
        }
    }

    pub(crate) fn push_if(&mut self, len: i32, cond: Expr) {
        self.scopes
            .push(Scope::new(ScopeType::Len(len), ScopeData::If { cond }))
    }

    pub(crate) fn push_else(&mut self, len: i32) {
        let (if_cond, stmts) = self
            .scopes
            .pop()
            .and_then(|s| match s.data {
                ScopeData::If { cond } => Some((cond, s.stmts)),
                _ => None,
            })
            .expect("Else without If ?");

        self.scopes.push(Scope::new(
            ScopeType::Len(len),
            ScopeData::Else {
                if_cond,
                if_stmts: stmts,
            },
        ));
    }

    pub(crate) fn push_switch(&mut self, len: i32, arg: Expr, offsets: Vec<usize>) {
        self.scopes.push(Scope::new(
            ScopeType::Len(len),
            ScopeData::Switch {
                arg,
                offsets,
                cases: Vec::new(),
            },
        ))
    }

    pub(crate) fn push_switch_case(&mut self, cst: usize) {
        // End the previous switch case scope
        let previous = {
            let scope = self.scopes.pop().unwrap();
            match scope.data {
                ScopeData::SwitchCase { pattern } => Some((pattern, scope.stmts)),
                _ => {
                    self.scopes.push(scope);
                    None
                }
            }
        };

        let scope = self.scopes.last_mut().unwrap();
        match &mut scope.data {
            ScopeData::Switch { cases, .. } => {
                if let Some(previous) = previous {
                    cases.push(previous);
                }

                self.scopes.push(Scope::new(
                    ScopeType::Manual,
                    ScopeData::SwitchCase {
                        pattern: cst_int(cst as i32),
                    },
                ));
            }
            _ => {
                panic!("Pushing a switch case with no outer switch !");
            }
        }
    }

    pub(crate) fn push_loop(&mut self, start: usize) {
        self.scopes.push(Scope::new(
            ScopeType::Manual,
            ScopeData::Loop {
                start,
                cond: Expr::Unknown("no condition".to_owned()),
            },
        ))
    }

    pub(crate) fn push_try(&mut self, len: i32) {
        self.scopes
            .push(Scope::new(ScopeType::Len(len), ScopeData::Try))
    }

    pub(crate) fn push_catch(&mut self, len: i32) {
        self.scopes
            .push(Scope::new(ScopeType::Len(len), ScopeData::Catch))
    }

    //region QUERIES
    /// Returns a mutable reference to the loop condition if the current scope is a loop
    pub(crate) fn update_last_loop_cond(&mut self) -> Option<&mut Expr> {
        self.scopes.last_mut().and_then(|s| match &mut s.data {
            ScopeData::Loop { cond, .. } => Some(cond),
            _ => None,
        })
    }

    /// Returns the start index of the last loop in the scope stack
    pub(crate) fn last_loop_start(&self) -> Option<usize> {
        self.scopes.iter().rev().find_map(|s| match s.data {
            ScopeData::Loop { start, .. } => Some(start),
            _ => None,
        })
    }

    /// End the last scope if its a loop
    pub(crate) fn end_last_loop(&mut self) -> Option<Statement> {
        self.scopes.pop().and_then(|s| match s.data {
            ScopeData::Loop { .. } => Some(s.make_stmt()),
            _ => None,
        })
    }

    /// Returns the switch jump offsets if the current scope is a switch (or a switch case)
    pub(crate) fn last_is_switch_ctx(&self) -> Option<&[usize]> {
        self.scopes.last().and_then(|s| match &s.data {
            ScopeData::Switch { offsets, .. } => Some(offsets.as_slice()),
            ScopeData::SwitchCase { .. } => match &self.scopes[self.scopes.len() - 2].data {
                ScopeData::Switch { offsets, .. } => Some(offsets.as_slice()),
                _ => None,
            },
            _ => None,
        })
    }

    pub(crate) fn last_is_if(&self) -> bool {
        self.scopes
            .last()
            .map(|s| matches!(&s.data, ScopeData::If { .. }))
            .unwrap_or(false)
    }

    pub(crate) fn has_scopes(&self) -> bool {
        self.scopes.len() > 1
    }
    //endregion
}
