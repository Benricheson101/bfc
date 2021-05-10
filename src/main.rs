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
