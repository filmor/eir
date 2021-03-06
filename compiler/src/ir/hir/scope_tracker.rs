use ::std::collections::HashMap;
use ::Atom;
use ::eir::FunctionIdent;
use ::eir::{ ModuleEnvs, ClosureEnv };
use ::ssa::{ SSAVariable, SSAVariableGenerator };

//#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
//pub struct LambdaEnvIdx(pub usize);

#[derive(Debug)]
enum Scope {
    /// A simple scope contains one or more variable assignments
    Simple(HashMap<ScopeDefinition, SSAVariable>),
    /// A tracking scope contains no assignments, but tracks variables
    /// that are referenced across it. Used to capture variables for
    /// closures.
    Tracking(HashMap<ScopeDefinition, (SSAVariable, SSAVariable)>),
}

#[derive(Debug)]
pub struct VerboseLambdaEnv {
    pub env: ClosureEnv,
    pub captures: Vec<(ScopeDefinition, SSAVariable, SSAVariable)>,
    pub meta_binds: Vec<(FunctionIdent, SSAVariable, Option<SSAVariable>)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ScopeDefinition {
    Variable(Atom),
    Function(FunctionIdent),
}

#[derive(Debug)]
pub struct ScopeTracker {
    scopes: Vec<Scope>,
    ssa_var_generator: SSAVariableGenerator,
    module_envs: ModuleEnvs,
    lambda_envs: HashMap<ClosureEnv, VerboseLambdaEnv>,
}
impl ScopeTracker {

    pub fn new() -> Self {
        ScopeTracker {
            scopes: vec![],
            ssa_var_generator: SSAVariableGenerator::initial(),
            module_envs: ModuleEnvs::new(),
            lambda_envs: HashMap::new(),
        }
    }

    pub fn clone_ssa_generator(&self) -> SSAVariableGenerator {
        self.ssa_var_generator.clone()
    }

    pub fn push_scope(&mut self, bindings: HashMap<ScopeDefinition, SSAVariable>) {
        self.scopes.push(Scope::Simple(bindings));
    }
    pub fn push_tracking(&mut self) {
        self.scopes.push(Scope::Tracking(HashMap::new()));
    }
    pub fn pop_scope(&mut self) {
        match self.scopes.pop().unwrap() {
            Scope::Simple(_) => (),
            _ => panic!(),
        }
    }
    pub fn pop_tracking(&mut self) -> HashMap<ScopeDefinition, (SSAVariable, SSAVariable)> {
        match self.scopes.pop().unwrap() {
            Scope::Tracking(tracked) => {
                tracked
            },
            _ => panic!(),
        }
    }

    pub fn new_ssa(&mut self) -> SSAVariable {
        self.ssa_var_generator.next()
    }

    pub fn get(&mut self, var: &ScopeDefinition) -> Option<SSAVariable> {
        let mut ssa_var_root: Option<SSAVariable> = None;
        let mut end_idx: usize = 0;

        for (idx, scope) in self.scopes.iter().rev().enumerate() {
            if let &Scope::Simple(ref bindings) = scope {
                if let Some(ssa) = bindings.get(var) {
                    ssa_var_root = Some(*ssa);
                    end_idx = idx;
                    break;
                }
            }
        }
        let scopes_len = self.scopes.len();
        end_idx = scopes_len - end_idx;

        if let Some(mut ssa_var) = ssa_var_root {
            for scope in &mut self.scopes[end_idx..scopes_len] {
                if let &mut Scope::Tracking(ref mut escapes) = scope {
                    if let Some(&(outer_ssa, inner_ssa)) = escapes.get(var) {
                        assert!(ssa_var == outer_ssa);
                        ssa_var = inner_ssa;
                    } else {
                        let inner_ssa = self.ssa_var_generator.next();
                        escapes.insert(var.clone(), (ssa_var, inner_ssa));
                        ssa_var = inner_ssa;
                    }
                }
            }
            Some(ssa_var)
        } else {
            None
        }
    }

    pub fn gen_env_idx(&mut self) -> ClosureEnv {
        self.module_envs.add()
    }

    pub fn add_lambda_env(&mut self, env_idx: ClosureEnv, env: VerboseLambdaEnv) {
        assert!(!self.lambda_envs.contains_key(&env_idx));
        self.lambda_envs.insert(env_idx, env);
    }

    pub fn get_lambda_env<'a>(&'a self, env_idx: ClosureEnv)
                          -> &'a VerboseLambdaEnv {
        &self.lambda_envs[&env_idx]
    }

    pub fn finish(mut self) -> ModuleEnvs {
        for (env, data) in self.lambda_envs {
            self.module_envs.env_set_captures_num(env, data.captures.len());
            for meta_bind in data.meta_binds {
                self.module_envs.env_add_meta_bind(env, meta_bind.0.clone());
            }
        }
        self.module_envs
    }

}
