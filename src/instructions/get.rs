use inkwell::{
    builder::Builder,
    context::Context,
    values::{FunctionValue, PointerValue},
};

pub fn build_get(
    context: &Context,
    builder: &Builder,
    getchar_fn: &FunctionValue,
    ptr: &PointerValue,
) -> Result<(), String> {
    let getchar_call = builder.build_call(*getchar_fn, &[], "getchar call");

    let getchar_result: Result<_, _> =
        getchar_call.try_as_basic_value().flip().into();
    let getchar_basic_val =
        getchar_result.map_err(|_| "getchar returned void")?;
    let i8_type = context.i8_type();

    let truncated = builder.build_int_truncate(
        getchar_basic_val.into_int_value(),
        i8_type,
        "getchar truncate result",
    );

    let ptr_val = builder
        .build_load(*ptr, "load ptr value")
        .into_pointer_value();
    builder.build_store(ptr_val, truncated);

    Ok(())
}
