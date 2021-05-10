use inkwell::{
    builder::Builder,
    context::Context,
    values::{FunctionValue, PointerValue},
};

pub fn build_put(
    context: &Context,
    builder: &Builder,
    putchar_fn: &FunctionValue,
    ptr: &PointerValue,
) {
    let char_to_put = builder.build_load(
        builder
            .build_load(*ptr, "load ptr value")
            .into_pointer_value(),
        "load ptr ptr value",
    );

    let s_ext = builder.build_int_s_extend(
        char_to_put.into_int_value(),
        context.i32_type(),
        "putchar sign extend",
    );

    builder.build_call(*putchar_fn, &[s_ext.into()], "putchar call");
}
