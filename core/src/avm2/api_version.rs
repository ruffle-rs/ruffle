use std::sync::OnceLock;

use crate::PlayerRuntime;
use enum_map::{enum_map, Enum, EnumMap};

// Based on https://github.com/adobe/avmplus/blob/master/core/api-versions.h
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, FromPrimitive, Enum)]
#[allow(non_camel_case_types)]
pub enum ApiVersion {
    AllVersions = 0,
    AIR_1_0 = 1,
    FP_10_0 = 2,
    AIR_1_5 = 3,
    AIR_1_5_1 = 4,
    FP_10_0_32 = 5,
    AIR_1_5_2 = 6,
    FP_10_1 = 7,
    AIR_2_0 = 8,
    AIR_2_5 = 9,
    FP_10_2 = 10,
    AIR_2_6 = 11,
    SWF_12 = 12,
    AIR_2_7 = 13,
    SWF_13 = 14,
    AIR_3_0 = 15,
    SWF_14 = 16,
    AIR_3_1 = 17,
    SWF_15 = 18,
    AIR_3_2 = 19,
    SWF_16 = 20,
    AIR_3_3 = 21,
    SWF_17 = 22,
    AIR_3_4 = 23,
    SWF_18 = 24,
    AIR_3_5 = 25,
    SWF_19 = 26,
    AIR_3_6 = 27,
    SWF_20 = 28,
    AIR_3_7 = 29,
    SWF_21 = 30,
    AIR_3_8 = 31,
    SWF_22 = 32,
    AIR_3_9 = 33,
    SWF_23 = 34,
    AIR_4_0 = 35,
    SWF_24 = 36,
    AIR_13_0 = 37,
    SWF_25 = 38,
    AIR_14_0 = 39,
    SWF_26 = 40,
    AIR_15_0 = 41,
    SWF_27 = 42,
    AIR_16_0 = 43,
    SWF_28 = 44,
    AIR_17_0 = 45,
    SWF_29 = 46,
    AIR_18_0 = 47,
    SWF_30 = 48,
    AIR_19_0 = 49,
    SWF_31 = 50,
    AIR_20_0 = 51,
    VM_INTERNAL = 52,
}

static TRANSFER_TABLE: OnceLock<EnumMap<ApiVersion, (ApiVersion, ApiVersion)>> = OnceLock::new();

impl ApiVersion {
    pub fn to_valid_playerglobals_version(self, runtime: PlayerRuntime) -> ApiVersion {
        // This maps an ApiVersion from our playerglobals SWF to the closest valid version,
        // based on the active runtime.
        //
        // If our runtime is AIR, then we leave ApiVersion::AIR_* unchanged, and map
        // ApiVersion::FP_* to the closest AIR version. This has the effect of exposing
        // API versioned with an AIR-specific version, and also exposing all of the normal FP
        // APIs that were included in that AIR release.
        //
        // If our runtime is FlashPlayer, then we leave ApiVersion::FP_* unchanged, and
        // map ApiVersion::AIR_* to VM_INTERNAL. This hides all AIR-specific APIs when
        // running in FlashPlayer.
        //
        // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/api-versions.cpp#L63
        let active_series = TRANSFER_TABLE.get_or_init(|| {
            enum_map! {
                ApiVersion::AllVersions => (ApiVersion::AllVersions, ApiVersion::AllVersions),
                ApiVersion::AIR_1_0 => (ApiVersion::AIR_1_0, ApiVersion::VM_INTERNAL),
                ApiVersion::FP_10_0 => (ApiVersion::AIR_1_5, ApiVersion::FP_10_0),
                ApiVersion::AIR_1_5 => (ApiVersion::AIR_1_5, ApiVersion::VM_INTERNAL),
                ApiVersion::AIR_1_5_1 => (ApiVersion::AIR_1_5_1, ApiVersion::VM_INTERNAL),
                ApiVersion::FP_10_0_32 => (ApiVersion::AIR_1_5_2, ApiVersion::FP_10_0_32),
                ApiVersion::AIR_1_5_2 => (ApiVersion::AIR_1_5_2, ApiVersion::VM_INTERNAL),
                ApiVersion::FP_10_1 => (ApiVersion::AIR_2_0, ApiVersion::FP_10_1),
                ApiVersion::AIR_2_0 => (ApiVersion::AIR_2_0, ApiVersion::VM_INTERNAL),
                ApiVersion::AIR_2_5 => (ApiVersion::AIR_2_5, ApiVersion::VM_INTERNAL),
                ApiVersion::FP_10_2 => (ApiVersion::AIR_2_6, ApiVersion::FP_10_2),
                ApiVersion::AIR_2_6 => (ApiVersion::AIR_2_6, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_12 => (ApiVersion::SWF_12, ApiVersion::SWF_12),
                ApiVersion::AIR_2_7 => (ApiVersion::AIR_2_7, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_13 => (ApiVersion::AIR_3_0, ApiVersion::SWF_13),
                ApiVersion::AIR_3_0 => (ApiVersion::AIR_3_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_14 => (ApiVersion::AIR_3_1, ApiVersion::SWF_14),
                ApiVersion::AIR_3_1 => (ApiVersion::AIR_3_1, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_15 => (ApiVersion::AIR_3_2, ApiVersion::SWF_15),
                ApiVersion::AIR_3_2 => (ApiVersion::AIR_3_2, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_16 => (ApiVersion::AIR_3_3, ApiVersion::SWF_16),
                ApiVersion::AIR_3_3 => (ApiVersion::AIR_3_3, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_17 => (ApiVersion::AIR_3_4, ApiVersion::SWF_17),
                ApiVersion::AIR_3_4 => (ApiVersion::AIR_3_4, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_18 => (ApiVersion::AIR_3_5, ApiVersion::SWF_18),
                ApiVersion::AIR_3_5 => (ApiVersion::AIR_3_5, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_19 => (ApiVersion::AIR_3_6, ApiVersion::SWF_19),
                ApiVersion::AIR_3_6 => (ApiVersion::AIR_3_6, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_20 => (ApiVersion::AIR_3_7, ApiVersion::SWF_20),
                ApiVersion::AIR_3_7 => (ApiVersion::AIR_3_7, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_21 => (ApiVersion::AIR_3_8, ApiVersion::SWF_21),
                ApiVersion::AIR_3_8 => (ApiVersion::AIR_3_8, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_22 => (ApiVersion::AIR_3_9, ApiVersion::SWF_22),
                ApiVersion::AIR_3_9 => (ApiVersion::AIR_3_9, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_23 => (ApiVersion::AIR_4_0, ApiVersion::SWF_23),
                ApiVersion::AIR_4_0 => (ApiVersion::AIR_4_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_24 => (ApiVersion::AIR_13_0, ApiVersion::SWF_24),
                ApiVersion::AIR_13_0 => (ApiVersion::AIR_13_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_25 => (ApiVersion::AIR_14_0, ApiVersion::SWF_25),
                ApiVersion::AIR_14_0 => (ApiVersion::AIR_14_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_26 => (ApiVersion::AIR_15_0, ApiVersion::SWF_26),
                ApiVersion::AIR_15_0 => (ApiVersion::AIR_15_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_27 => (ApiVersion::AIR_16_0, ApiVersion::SWF_27),
                ApiVersion::AIR_16_0 => (ApiVersion::AIR_16_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_28 => (ApiVersion::AIR_17_0, ApiVersion::SWF_28),
                ApiVersion::AIR_17_0 => (ApiVersion::AIR_17_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_29 => (ApiVersion::AIR_18_0, ApiVersion::SWF_29),
                ApiVersion::AIR_18_0 => (ApiVersion::AIR_18_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_30 => (ApiVersion::AIR_19_0, ApiVersion::SWF_30),
                ApiVersion::AIR_19_0 => (ApiVersion::AIR_19_0, ApiVersion::VM_INTERNAL),
                ApiVersion::SWF_31 => (ApiVersion::AIR_20_0, ApiVersion::SWF_31),
                ApiVersion::AIR_20_0 => (ApiVersion::AIR_20_0, ApiVersion::VM_INTERNAL),
                ApiVersion::VM_INTERNAL => (ApiVersion::VM_INTERNAL, ApiVersion::VM_INTERNAL),
            }
        })[self];

        match runtime {
            PlayerRuntime::AIR => active_series.0,
            PlayerRuntime::FlashPlayer => active_series.1,
        }
    }

    pub fn from_swf_version(val: u8, runtime: PlayerRuntime) -> Option<ApiVersion> {
        // Based on this table: https://github.com/ruffle-rs/ruffle/wiki/SWF-version-chart
        match (val, runtime) {
            // There's no specific entry for SWF 9 in avmplus,
            // so map it to the lowest entry.
            (9, _) => Some(ApiVersion::AllVersions),
            (10, PlayerRuntime::FlashPlayer) => Some(ApiVersion::FP_10_1),
            (10, PlayerRuntime::AIR) => Some(ApiVersion::AIR_2_0),
            (11, PlayerRuntime::FlashPlayer) => Some(ApiVersion::FP_10_2),
            (11, PlayerRuntime::AIR) => Some(ApiVersion::AIR_2_6),
            (12, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_12),
            (12, PlayerRuntime::AIR) => Some(ApiVersion::AIR_2_7),
            (13, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_13),
            (13, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_0),
            (14, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_14),
            (14, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_1),
            (15, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_15),
            (15, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_2),
            (16, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_16),
            (16, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_3),
            (17, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_17),
            (17, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_4),
            (18, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_18),
            (18, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_5),
            (19, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_19),
            (19, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_6),
            (20, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_20),
            (20, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_7),
            (21, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_21),
            (21, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_8),
            (22, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_22),
            (22, PlayerRuntime::AIR) => Some(ApiVersion::AIR_3_9),
            (23, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_23),
            (23, PlayerRuntime::AIR) => Some(ApiVersion::AIR_4_0),
            (24, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_24),
            (24, PlayerRuntime::AIR) => Some(ApiVersion::AIR_13_0),
            (25, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_25),
            (25, PlayerRuntime::AIR) => Some(ApiVersion::AIR_14_0),
            (26, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_26),
            (26, PlayerRuntime::AIR) => Some(ApiVersion::AIR_15_0),
            (27, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_27),
            (27, PlayerRuntime::AIR) => Some(ApiVersion::AIR_16_0),
            (28, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_28),
            (28, PlayerRuntime::AIR) => Some(ApiVersion::AIR_17_0),
            (29, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_29),
            (29, PlayerRuntime::AIR) => Some(ApiVersion::AIR_18_0),
            (30, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_30),
            (30, PlayerRuntime::AIR) => Some(ApiVersion::AIR_19_0),
            (31, PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_31),
            (31, PlayerRuntime::AIR) => Some(ApiVersion::AIR_20_0),

            // We haven't yet created entries from higher versions - just map them
            // to the highest non-VM_INTERNAL version.
            (32.., PlayerRuntime::FlashPlayer) => Some(ApiVersion::SWF_31),
            (32.., PlayerRuntime::AIR) => Some(ApiVersion::AIR_20_0),
            _ => None,
        }
    }
}
