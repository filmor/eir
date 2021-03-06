extern crate core_erlang_compiler;
extern crate eir;

use eir::ClosureEnv;
use core_erlang_compiler::parser::Atom;

use std::io::Read;
use std::str::FromStr;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let infile = args.next().unwrap();

    let fun_name = args.next();
    let arity = args.next();
    let lambda_env = args.next();
    let lambda_num = args.next();

    let mut text = String::new();
    std::fs::File::open(&infile).unwrap()
        .read_to_string(&mut text).unwrap();

    let res = core_erlang_compiler::parser::parse(&text).unwrap();
    let hir = core_erlang_compiler::ir::from_parsed(&res.0);

    for fun in hir.functions.iter() {
        println!("{}", fun.0);
    }

    if let Some(fun_name) = fun_name {

        let name_sym = Atom::from_str(&fun_name);
        let arity = arity.unwrap().parse().unwrap();
        let lambda_env: Option<usize> = lambda_env.map(|s| s.parse().unwrap());
        let lambda_num: Option<usize> = lambda_num.map(|s| s.parse().unwrap());

        let lambda_d = lambda_env.as_ref().map(|v| (
            ClosureEnv::from_num(*v), lambda_num.unwrap()));

        //let funs: Vec<_> = hir.functions.iter().map(|f| f.0.clone()).collect();
        //println!("{:?}", funs);
        let fun = hir.functions.iter().find(|f| {
            f.0.name == name_sym
                && f.0.arity == arity
                && f.0.lambda == lambda_d
        }).unwrap();

        println!("Writing {} to {}.dot", fun.0, infile);
        let mut out = ::std::fs::File::create(infile + ".dot").unwrap();
        ::eir::text::function_to_dot(&fun.1, &mut out).unwrap();
        //core_erlang_compiler::ir::lir::to_dot::function_to_dot(
        //    &fun.0, &fun.1.lir, &mut out).unwrap();

    } else {
        println!("No function name provided");
    }

}
