use crate::locale::get_current_date_time;

// https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/MathUtils.cpp#L1546
const C1: i32 = 1376312589;
const C2: i32 = 789221;
const C3: i32 = 15731;
const K_RANDOM_PURE_MAX: i32 = 0x7FFFFFFF;

const U_XOR_MASK: u32 = 0x48000000;

// This struct should not be cloned or copied.
#[derive(Debug, Default)]
pub struct AvmRng {
    u_value: u32,
}

impl AvmRng {
    fn init_with_seed(&mut self, seed: u32) {
        self.u_value = seed;
    }

    fn random_fast_next(&mut self) -> i32 {
        if (self.u_value & 1) != 0 {
            self.u_value = (self.u_value >> 1) ^ U_XOR_MASK;
        } else {
            self.u_value >>= 1;
        }
        self.u_value as i32
    }

    fn random_pure_hasher(&self, mut i_seed: i32) -> i32 {
        i_seed = ((i_seed << 13) ^ i_seed).wrapping_sub(i_seed >> 21);

        let mut i_result = i_seed.wrapping_mul(i_seed);
        i_result = i_result.wrapping_mul(C3);
        i_result = i_result.wrapping_add(C2);
        i_result = i_result.wrapping_mul(i_seed);
        i_result = i_result.wrapping_add(C1);
        i_result &= K_RANDOM_PURE_MAX;

        i_result = i_result.wrapping_add(i_seed);

        i_result = ((i_result << 13) ^ i_result).wrapping_sub(i_result >> 21);

        i_result
    }

    pub fn generate_random_number(&mut self) -> i32 {
        // In avmplus, RNG is initialized on first use.
        if self.u_value == 0 {
            let seed = get_seed();
            self.init_with_seed(seed);
        }

        let mut a_num = self.random_fast_next();

        a_num = self.random_pure_hasher(a_num.wrapping_mul(71));

        a_num & K_RANDOM_PURE_MAX
    }
}

// https://github.com/adobe-flash/avmplus/blob/65a05927767f3735db37823eebf7d743531f5d37/VMPI/PosixSpecificUtils.cpp#L18
fn get_seed() -> u32 {
    get_current_date_time().timestamp_micros() as u32
}
