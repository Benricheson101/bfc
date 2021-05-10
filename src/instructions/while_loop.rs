use std::collections::VecDeque;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    values::{FunctionValue, PointerValue},
    IntPredicate,
};

pub struct WhileBlock<'ctx> {
    pub while_start: BasicBlock<'ctx>,
    pub while_body: BasicBlock<'ctx>,
    pub while_end: BasicBlock<'ctx>,
}

pub fn build_while_start<'ctx>(
    context: &'ctx Context,
    builder: &Builder,
    main_fn: &FunctionValue,
    ptr: &PointerValue,
    while_blocks: &mut VecDeque<WhileBlock<'ctx>>,
) {
    let num_while_blocks = while_blocks.len() + 1;
    let while_block = WhileBlock {
        while_start: context.append_basic_block(
            *main_fn,
            format!("while_start {}", num_while_blocks).as_str(),
        ),
        while_body: context.append_basic_block(
            *main_fn,
            format!("while_body {}", num_while_blocks).as_str(),
        ),
        while_end: context.append_basic_block(
            *main_fn,
            format!("while_end {}", num_while_blocks).as_str(),
        ),
    };

    while_blocks.push_front(while_block);
    let while_block = while_blocks.front().unwrap();

    builder.build_unconditional_branch(while_block.while_start);
    builder.position_at_end(while_block.while_start);

    let i8_type = context.i8_type();
    let i8_zero = i8_type.const_int(0, false);
    let ptr_load = builder.build_load(*ptr, "load ptr").into_pointer_value();

    let ptr_val = builder
        .build_load(ptr_load, "load ptr value")
        .into_int_value();

    let cmp = builder.build_int_compare(
        IntPredicate::NE,
        ptr_val,
        i8_zero,
        "compare value at pointer to zero",
    );

    builder.build_conditional_branch(
        cmp,
        while_block.while_body,
        while_block.while_end,
    );
    builder.position_at_end(while_block.while_body);
}

pub fn build_while_end<'ctx>(
    builder: &Builder,
    while_blocks: &mut VecDeque<WhileBlock<'ctx>>,
) -> Result<(), String> {
    if let Some(while_block) = while_blocks.pop_front() {
        builder.build_unconditional_branch(while_block.while_start);
        builder.position_at_end(while_block.while_end);
        Ok(())
    } else {
        Err("error: unmatched `]`".into())
    }
}
