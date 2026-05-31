mod blocks;
mod dce;
mod int_interpretation;
mod nop_remover;
mod peephole;
mod type_aware;
pub(crate) mod utils;

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::method::{Method, ResolvedParamConfig};
use crate::avm2::op::Op;
use crate::avm2::verify::Exception;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::HashSet;

/// Run all the optimizer passes on the given code. This method should be
/// run regardless of whether or not the "disable AVM2 optimizer" player option
/// is on. It will not perform observable optimizations if the "disable
/// AVM2 optimizer" player option is off.
pub fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    code: &mut Vec<Op<'gc>>,
    method_exceptions: &mut [Exception<'gc>],
    resolved_parameters: &[ResolvedParamConfig<'gc>],
    mut jump_targets: HashSet<usize>,
) -> Result<(), Error<'gc>> {
    let code_slice = Cell::from_mut(code.as_mut_slice());
    let code_slice = code_slice.as_slice_of_cells();

    // We run the preprocess peephole before assembling blocks because it removes
    // zero-length jumps, which usually reduces the number of blocks in obfuscated code.
    peephole::preprocess_peephole(code_slice);

    let mut empty_stack_positions = BTreeMap::new();

    type_aware::type_aware_optimize(
        activation,
        method,
        code_slice,
        method_exceptions,
        resolved_parameters,
        &mut jump_targets,
        &mut empty_stack_positions,
    )?;

    peephole::postprocess_peephole(
        code_slice,
        &jump_targets,
        &mut empty_stack_positions,
        !method_exceptions.is_empty(),
    );

    dce::eliminate_dead_code(code_slice, &jump_targets);

    int_interpretation::run_analysis(
        activation,
        method,
        code_slice,
        &jump_targets,
        &empty_stack_positions,
        !method_exceptions.is_empty(),
    );

    nop_remover::remove_nops(code, method_exceptions);

    Ok(())
}
