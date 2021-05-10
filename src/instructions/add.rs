use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn build_add(
    context: &Context,
    builder: &Builder,
    amt: i32,
    ptr: &PointerValue,
) {
    let i8_type = context.i8_type();
    let i8_amt = i8_type.const_int(amt as u64, false);

    let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();
    let ptr_val = builder.build_load(ptr_load, "load ptr value");

    let result = builder.build_int_add(
        ptr_val.into_int_value(),
        i8_amt,
        "add data to ptr",
    );

    builder.build_store(ptr_load, result);
}
