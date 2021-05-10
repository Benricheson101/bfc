mod instructions;

use std::{collections::VecDeque, error, fs};

use clap::{crate_authors, crate_version, App, Arg};
use inkwell::{
    context::Context,
    module::Linkage,
    targets::{
        CodeModel,
        FileType,
        InitializationConfig,
        RelocMode,
        Target,
        TargetMachine,
    },
    AddressSpace,
    OptimizationLevel,
};
use instructions::*;

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("Brainfuck Compiler")
        .about("Compiles a brainfuck file to an object file. Use GCC or ld to link and make executable.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("INPUT")
                .help("source brainfuck file to compile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .help("output filename")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("assembly")
                .short("S")
                .help("output assembly")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    let context = Context::create();
    let module = context.create_module("brainfuck_rust");
    let builder = context.create_builder();

    let i64_type = context.i64_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::Global);

    let calloc_fn_type =
        i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let calloc_fn =
        module.add_function("calloc", calloc_fn_type, Some(Linkage::External));

    let i32_type = context.i32_type();
    let main_fn_type = i32_type.fn_type(&[], false);

    let getchar_fn_type = i32_type.fn_type(&[], false);
    let getchar_fn = module.add_function(
        "getchar",
        getchar_fn_type,
        Some(Linkage::External),
    );

    let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);
    let putchar_fn = module.add_function(
        "putchar",
        putchar_fn_type,
        Some(Linkage::External),
    );

    let main_fn =
        module.add_function("main", main_fn_type, Some(Linkage::External));

    let basic_block = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(basic_block);

    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);

    let data = builder.build_alloca(i8_ptr_type, "data");
    let ptr = builder.build_alloca(i8_ptr_type, "ptr");

    let i64_type = context.i64_type();
    let i64_memory_size = i64_type.const_int(30_000, false);
    let i64_element_size = i64_type.const_int(1, false);

    let data_ptr = builder.build_call(
        calloc_fn,
        &[i64_memory_size.into(), i64_element_size.into()],
        "calloc_call",
    );

    let data_ptr_result: Result<_, _> =
        data_ptr.try_as_basic_value().flip().into();
    let data_ptr_basic_val =
        data_ptr_result.map_err(|_| "calloc returned void")?;

    builder.build_store(data, data_ptr_basic_val);
    builder.build_store(ptr, data_ptr_basic_val);

    let source_filename = matches.value_of("INPUT").unwrap();
    let program = fs::read_to_string(source_filename).unwrap();

    let mut while_blocks = VecDeque::new();

    for command in program.chars() {
        match command {
            '>' => build_add_ptr(&context, &builder, 1, &ptr),
            '<' => build_add_ptr(&context, &builder, -1, &ptr),
            '+' => build_add(&context, &builder, 1, &ptr),
            '-' => build_add(&context, &builder, -1, &ptr),
            '.' => build_put(&context, &builder, &putchar_fn, &ptr),
            ',' => build_get(&context, &builder, &getchar_fn, &ptr)?,
            '[' => build_while_start(
                &context,
                &builder,
                &main_fn,
                &ptr,
                &mut while_blocks,
            ),
            ']' => build_while_end(&builder, &mut while_blocks)?,
            _ => (),
        }
    }

    builder.build_free(builder.build_load(data, "load").into_pointer_value());

    let i32_zero = i32_type.const_int(0, false);
    builder.build_return(Some(&i32_zero));

    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    let target =
        Target::from_triple(&target_triple).map_err(|e| format!("{:?}", e))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            &cpu,
            &features,
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| "Unable to create target machine".to_string())?;

    let out_file = matches.value_of("output").unwrap();
    let out_type = if matches.is_present("assembly") {
        FileType::Assembly
    } else {
        FileType::Object
    };
    target_machine
        .write_to_file(&module, out_type, out_file.as_ref())
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

// fn build_add_ptr(
//     context: &Context,
//     builder: &Builder,
//     amt: i32,
//     ptr: &PointerValue,
// ) {
//     let i32_type = context.i32_type();
//     let i32_amt = i32_type.const_int(amt as u64, false);

//     let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();

//     let result = unsafe {
//         builder.build_in_bounds_gep(ptr_load, &[i32_amt], "add to pointer")
//     };

//     builder.build_store(*ptr, result);
// }

// fn build_add(
//     context: &Context,
//     builder: &Builder,
//     amt: i32,
//     ptr: &PointerValue,
// ) {
//     let i8_type = context.i8_type();
//     let i8_amt = i8_type.const_int(amt as u64, false);

//     let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();
//     let ptr_val = builder.build_load(ptr_load, "load ptr value");

//     let result = builder.build_int_add(
//         ptr_val.into_int_value(),
//         i8_amt,
//         "add data to ptr",
//     );

//     builder.build_store(ptr_load, result);
// }

// fn build_get(
//     context: &Context,
//     builder: &Builder,
//     getchar_fn: &FunctionValue,
//     ptr: &PointerValue,
// ) -> Result<(), String> {
//     let getchar_call = builder.build_call(*getchar_fn, &[], "getchar call");

//     let getchar_result: Result<_, _> =
//         getchar_call.try_as_basic_value().flip().into();
//     let getchar_basic_val =
//         getchar_result.map_err(|_| "getchar returned void")?;
//     let i8_type = context.i8_type();

//     let truncated = builder.build_int_truncate(
//         getchar_basic_val.into_int_value(),
//         i8_type,
//         "getchar truncate result",
//     );

//     let ptr_val = builder
//         .build_load(*ptr, "load ptr value")
//         .into_pointer_value();
//     builder.build_store(ptr_val, truncated);

//     Ok(())
// }

// fn build_put(
//     context: &Context,
//     builder: &Builder,
//     putchar_fn: &FunctionValue,
//     ptr: &PointerValue,
// ) {
//     let char_to_put = builder.build_load(
//         builder
//             .build_load(*ptr, "load ptr value")
//             .into_pointer_value(),
//         "load ptr ptr value",
//     );

//     let s_ext = builder.build_int_s_extend(
//         char_to_put.into_int_value(),
//         context.i32_type(),
//         "putchar sign extend",
//     );

//     builder.build_call(*putchar_fn, &[s_ext.into()], "putchar call");
// }

// struct WhileBlock<'ctx> {
//     while_start: BasicBlock<'ctx>,
//     while_body: BasicBlock<'ctx>,
//     while_end: BasicBlock<'ctx>,
// }

// fn build_while_start<'ctx>(
//     context: &'ctx Context,
//     builder: &Builder,
//     main_fn: &FunctionValue,
//     ptr: &PointerValue,
//     while_blocks: &mut VecDeque<WhileBlock<'ctx>>,
// ) {
//     let num_while_blocks = while_blocks.len() + 1;
//     let while_block = WhileBlock {
//         while_start: context.append_basic_block(
//             *main_fn,
//             format!("while_start {}", num_while_blocks).as_str(),
//         ),
//         while_body: context.append_basic_block(
//             *main_fn,
//             format!("while_body {}", num_while_blocks).as_str(),
//         ),
//         while_end: context.append_basic_block(
//             *main_fn,
//             format!("while_end {}", num_while_blocks).as_str(),
//         ),
//     };

//     while_blocks.push_front(while_block);
//     let while_block = while_blocks.front().unwrap();

//     builder.build_unconditional_branch(while_block.while_start);
//     builder.position_at_end(while_block.while_start);

//     let i8_type = context.i8_type();
//     let i8_zero = i8_type.const_int(0, false);
//     let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();

//     let ptr_val = builder
//         .build_load(ptr_load, "load ptr value")
//         .into_int_value();

//     let cmp = builder.build_int_compare(
//         IntPredicate::NE,
//         ptr_val,
//         i8_zero,
//         "compare value at pointer to zero",
//     );

//     builder.build_conditional_branch(
//         cmp,
//         while_block.while_body,
//         while_block.while_end,
//     );
//     builder.position_at_end(while_block.while_body);
// }

// fn build_while_end<'ctx>(
//     builder: &Builder,
//     while_blocks: &mut VecDeque<WhileBlock<'ctx>>,
// ) -> Result<(), String> {
//     if let Some(while_block) = while_blocks.pop_front() {
//         builder.build_unconditional_branch(while_block.while_start);
//         builder.position_at_end(while_block.while_end);
//         Ok(())
//     } else {
//         Err("error: unmatched `]`".into())
//     }
// }
