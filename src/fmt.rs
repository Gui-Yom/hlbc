use std::fmt;

use crate::{HlCode, HlNative, HlType, HlTypeRef};

/*
pub trait CodeDisplay {
    fn display<T: fmt::Display>(&self, ctx: &HlCode) -> T;
}*/

impl HlTypeRef {
    pub fn display(&self, ctx: &HlCode) -> impl fmt::Display {
        self.resolve(&ctx.types).display(ctx)
    }
}

impl HlType {
    pub fn display(&self, ctx: &HlCode) -> impl fmt::Display {
        format!("{self:?}")
    }
}

impl HlNative {
    pub fn display(&self, ctx: &HlCode) -> impl fmt::Display {
        format!(
            "fn:native {}/{}@{} {}",
            self.lib,
            self.name,
            self.findex,
            self.t.display(ctx)
        )
    }
}
