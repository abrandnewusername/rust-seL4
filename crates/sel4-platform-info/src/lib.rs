#![no_std]
#![allow(clippy::single_range_in_vec_init)]

use sel4_platform_info_types::PlatformInfo;

include! {
    concat!(env!("OUT_DIR"), "/gen.rs")
}
