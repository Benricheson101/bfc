use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn build_add_ptr(
    context: &Context,
    builder: &Builder,
    amt: i32,
    ptr: &PointerValue,
) {
    let i32_type = context.i32_type();
    let i32_amt = i32_type.const_int(amt as u64, false);

    let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();

    let result = unsafe {
        builder.build_in_bounds_gep(ptr_load, &[i32_amt], "add to pointer")
    };

    builder.build_store(*ptr, result);
}
