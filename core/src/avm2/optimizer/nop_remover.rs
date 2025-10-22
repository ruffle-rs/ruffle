use crate::avm2::op::Op;
use crate::avm2::verify::Exception;

pub fn remove_nops<'gc>(code: &mut Vec<Op<'gc>>, exceptions: &mut [Exception<'gc>]) {
    let mut offset_vec = vec![0; code.len()];
    let mut current_offset = 0;

    // First, remove nops
    let mut i = 0;
    while i < code.len() {
        offset_vec[i] = i - current_offset;
        if code[i].is_nop() {
            current_offset += 1;
        } else {
            // Shift the ops over the nops
            code[i - current_offset] = code[i];
        }

        i += 1;
    }

    // The ops have all been shifted over now, so remove the garbage ops left
    // at the end of the code Vec
    code.truncate(code.len() - current_offset);

    // Rewrite jump offsets
    for op in code {
        match op {
            Op::IfTrue { offset }
            | Op::IfFalse { offset }
            | Op::Jump { offset }
            | Op::PopJump { offset } => {
                *offset = offset_vec[*offset];
            }
            Op::LookupSwitch(lookup_switch) => {
                for target in lookup_switch
                    .case_offsets
                    .iter()
                    .chain(std::slice::from_ref(&lookup_switch.default_offset))
                {
                    target.set(offset_vec[target.get()]);
                }
            }
            _ => {}
        }
    }

    // Rewrite exception offsets too
    for exception in exceptions {
        exception.from_offset = offset_vec[exception.from_offset];
        exception.to_offset = offset_vec[exception.to_offset];
        exception.target_offset = offset_vec[exception.target_offset];
    }
}
