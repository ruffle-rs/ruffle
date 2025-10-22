mod blocks;
mod nop_remover;
mod peephole;
mod type_aware;

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::method::{Method, ResolvedParamConfig};
use crate::avm2::op::Op;
use crate::avm2::verify::Exception;

use std::cell::Cell;
use std::collections::HashSet;

/// Run all the optimizer passes on the given code. This method should be
/// run regardless of whether or not the AVM2 optimizer is enabled. It will not
/// perform observable optimizations if the AVM2 optimizer is disabled.
pub fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    code: &mut Vec<Op<'gc>>,
    method_exceptions: &mut [Exception<'gc>],
    resolved_parameters: &[ResolvedParamConfig<'gc>],
    jump_targets: &HashSet<usize>,
) -> Result<(), Error<'gc>> {
    let code_slice = Cell::from_mut(code.as_mut_slice());
    let code_slice = code_slice.as_slice_of_cells();

    // We run the preprocess peephole before assembling blocks because it removes
    // zero-length jumps, which usually reduces the number of blocks in obfuscated code.
    peephole::preprocess_peephole(code_slice);

    type_aware::type_aware_optimize(
        activation,
        method,
        code_slice,
        method_exceptions,
        resolved_parameters,
        jump_targets,
    )?;

    peephole::postprocess_peephole(code_slice, jump_targets, !method_exceptions.is_empty());

    nop_remover::remove_nops(code, method_exceptions);

    Ok(())
}
