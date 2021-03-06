use std::fmt::{ Display, Formatter };
use std::collections::HashMap;

use cranelift_entity::EntityRef;

pub mod intern;
pub use intern::Atom;

pub mod op;

pub mod text;

pub mod pattern;
pub use pattern::{ Clause, Pattern };

pub mod fun;
pub use fun::{ Function, FunctionBuilder, EbbCall, Ebb, Op, Value };
pub use fun::{ FunctionCfg, CfgNode, CfgEdge, ValueType, Direction };

pub mod env;
pub use env::{ ModuleEnvs, ClosureEnv };

pub mod constant;
pub use constant::{ AtomicTerm, ConstantTerm };

#[derive(Debug)]
pub struct Module {
    pub name: Atom,
    pub envs: ModuleEnvs,
    pub functions: HashMap<FunctionIdent, Function>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FunctionIdent {
    pub module: Atom,
    pub name: Atom,
    pub arity: usize,
    pub lambda: Option<(ClosureEnv, usize)>,
}

impl Display for FunctionIdent {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some((env, e_idx)) = self.lambda {
            write!(f, "{}:{}@{}.{}/{}", self.module, self.name,
                   env.index(), e_idx, self.arity)
        } else {
            write!(f, "{}:{}/{}", self.module, self.name, self.arity)
        }
    }
}
