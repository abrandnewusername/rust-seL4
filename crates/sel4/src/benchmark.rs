//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

use crate::cap::{Tcb, UnspecifiedFrame};

use crate::{sys, Error, Result, Word};

pub fn benchmark_reset_log() -> Result<()> {
    Error::wrap(sys::seL4_BenchmarkResetLog())
}

pub fn benchmark_finalize_log() -> Word {
    sys::seL4_BenchmarkFinalizeLog()
}

pub fn benchmark_set_log_buffer(frame: UnspecifiedFrame) -> Result<()> {
    Error::wrap(sys::seL4_BenchmarkSetLogBuffer(frame.bits()))
}

sel4_cfg_if! {
    if #[sel4_cfg(BENCHMARK_TRACK_UTILISATION)] {
        pub fn benchmark_get_thread_utilisation(tcb: Tcb) {
            sys::seL4_BenchmarkGetThreadUtilisation(tcb.bits())
        }

        pub fn benchmark_reset_thread_utilisation(tcb: Tcb) {
            sys::seL4_BenchmarkResetThreadUtilisation(tcb.bits())
        }

        sel4_cfg_if! {
            if #[sel4_cfg(DEBUG_BUILD)] {
                pub fn benchmark_dump_all_thread_utilisation() {
                    sys::seL4_BenchmarkDumpAllThreadsUtilisation()
                }

                pub fn benchmark_reset_all_thread_utilisation() {
                    sys::seL4_BenchmarkResetAllThreadsUtilisation()
                }
            }
        }
    }
}

sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_X86_DANGEROUS_MSR)] {
        pub fn x86_dangerous_rdmsr(msr: Word) -> u64 {
            sys::seL4_X86DangerousRDMSR(msr)
        }

        pub fn x86_dangerous_wrmsr(msr: Word, value: Word) {
            sys::seL4_X86DangerousWRMSR(msr, value)
        }

    }
}
