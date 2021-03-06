use llvm_sys as llvm;
use std::ptr;
use std::collections::HashMap;

use eir::SSAVariable;
use eir::op::OpKind;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::values::{ FunctionValue, ArrayValue, StructValue, PointerValue,
                       GlobalValue, AnyValueEnum, BasicValueEnum };
use inkwell::types::{ StructType, BasicType, BasicTypeEnum };
use inkwell::module::{ Module };

mod primitives;
mod emit;
mod nif_types;

use primitives::{ EnifEntryT, EnifFuncT, make_enif_entry_t };
use nif_types::NifTypes;
use emit::{ emit_eir_fun, mangle_ident };

use ::std::path::Path;
use ::std::io::Read;

use eir::{ FunctionIdent, Function };

fn gen_wrapper(context: &Context, module: &Module, nif_refs: &NifTypes,
               ident: &FunctionIdent, inner: FunctionValue) -> FunctionValue {
    let i64_type = context.i64_type();

    let fn_val = module.add_function("nif_wrapper", nif_refs.nif_fun_type, None);

    let builder = context.create_builder();

    let entry_bb = fn_val.append_basic_block("entry");
    let ret_bb = fn_val.append_basic_block("return");
    let raise_bb = fn_val.append_basic_block("return");

    // Main block
    builder.position_at_end(&entry_bb);

    let env_arg = fn_val.get_nth_param(0).unwrap();
    let arr_arg = fn_val.get_nth_param(2).unwrap();
    let ret_term_ptr_val = builder.build_alloca(nif_refs.term_type, "ret term");

    let mut inner_args: Vec<BasicValueEnum> = vec![
        ret_term_ptr_val.into(),
        env_arg.into(),
    ];
    inner_args.extend((0..(ident.arity)).map(|n| {
        let num = i64_type.const_int(n as u64, false);
        let gep = unsafe { builder.build_gep(
            arr_arg.into_pointer_value(), &[num], "arg") };
        builder.build_load(gep, "arg")
    }));

    let inner_call = builder.build_call(inner, &inner_args, "inner");
    let inner_call_ret = inner_call.try_as_basic_value().left().unwrap();

    let ret_term_val = builder.build_load(ret_term_ptr_val, "");

    builder.build_conditional_branch(inner_call_ret.into_int_value(),
                                     &ret_bb, &raise_bb);

    // Return block
    builder.position_at_end(&ret_bb);
    builder.build_return(Some(&ret_term_val));

    // Throw block
    builder.position_at_end(&raise_bb);
    let call = builder.build_call(nif_refs.enif_raise_exception, &[
        env_arg.into(),
        ret_term_val.into(),
    ], "raise");
    builder.build_return(Some(&call.try_as_basic_value().left().unwrap()));

    fn_val
}


fn gen_prototype(context: &Context, module: &Module, nif_refs: &NifTypes,
                 ident: &FunctionIdent) -> FunctionValue {
    let bool_type = context.bool_type();

    let mut arity = ident.arity;
    if ident.lambda.is_some() { arity += 1; }

    let mut args = vec![
        nif_refs.term_ptr_type.into(),
        nif_refs.env_ptr_type.into()
    ];
    args.extend((0..arity).map(|_| nif_refs.term_type));

    let fn_type = bool_type.fn_type(&args, false);

    let ident_mangled = mangle_ident(ident);
    module.add_function(&ident_mangled, fn_type, None)
}

fn gen_prototypes(context: &Context, module: &Module, nif_refs: &NifTypes,
                  fun: &Function, protos: &mut HashMap<FunctionIdent, FunctionValue>) {
    let all_calls = fun.lir.get_all_calls();

    let val = gen_prototype(context, module, nif_refs, &fun.ident);
    protos.insert(fun.ident.clone(), val);

    for call in all_calls.iter() {
        if !protos.contains_key(call) {
            let val = gen_prototype(context, module, nif_refs, call);
            protos.insert(call.clone(), val);
        }
    }
}

fn main() {

    let eir = {
        let mut text = String::new();
        std::fs::File::open("testing.core").unwrap()
            .read_to_string(&mut text).unwrap();
        let res = core_erlang_compiler::parser::parse(&text).unwrap();
        core_erlang_compiler::ir::from_parsed(&res.0)
    };

    let mut path = Path::new("module.bc");

    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();

    let nif_types = nif_types::make_nif_types(&context, &module);
    let i64_type = context.i64_type();
    let i32_type = context.i32_type();

    let mut target = None;

    let mut protos = HashMap::new();
    for fun in eir.functions.iter() {
        let mangled = mangle_ident(fun.0);
        println!("{}: {}", fun.0, mangled);
        if mangled == "GNIF7_testing3_woo1_n_n" {
            gen_prototypes(&context, &module, &nif_types, &fun.1, &mut protos);
            println!("{:?}", protos);
            emit_eir_fun(&context, &module, &nif_types, &fun.1, protos[&fun.1.ident]);
            target = Some((fun.1.ident.clone(), protos[&fun.1.ident]));
        }
    }

    //let fn_type = i64_type.fn_type(&[
    //    nif_types.term_ptr_type.into(),
    //    nif_types.env_ptr_type.into(),
    //    i32_type.into(),
    //    i64_type.ptr_type(AddressSpace::Generic).into(),
    //], false);
    //let fn_val = module.add_function("foo_nif", fn_type, None);

    let entry_t = EnifEntryT {
        major: 2,
        minor: 14,
        name: "Elixir.Something.Nif".into(),
        funcs: vec![
            EnifFuncT {
                name: "woohoo".into(),
                arity: 1,
                fun: {
                    let tar = target.unwrap();
                    gen_wrapper(&context, &module, &nif_types,
                                &tar.0, tar.1)
                        .as_global_value().as_pointer_value()
                },
                flags: 0
            },
        ],
        load: None,
        reload: None,
        upgrade: None,
        unload: None,
        vm_variant: "beam.vanilla".into(),
        options: 0,
        sizeof_erlnifresourcetypeinit: 0,
        min_erts: "0.0".into(),
    };
    let nif_entry = make_enif_entry_t(&context, &module, &entry_t);
    let nif_entry_pointer = nif_entry.as_pointer_value();

    let nif_init_fn_type = (nif_entry_pointer.get_type()).fn_type(&[], false);
    let nif_init_fn_val = module.add_function("nif_init", nif_init_fn_type, None);

    let builder = context.create_builder();
    {
        let basic_block = context.append_basic_block(&nif_init_fn_val, "entry");
        builder.position_at_end(&basic_block);

        builder.build_return(Some(&nif_entry_pointer));
    }


    println!("{:?}", module.verify());

    module.write_bitcode_to_path(&path);
}
