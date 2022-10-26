// Copyright (c) 2022 Huawei Technologies Co.,Ltd. All rights reserved.
//
// StratoVirt is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("Signal")]
    Signal {
        #[from]
        source: vmm_sys_util::errno::Error,
    },

    #[error("Failed to create kvm vcpu: {0}!")]
    CreateVcpu(String),
    #[error("Failed to configure kvm vcpu: {0}!")]
    RealizeVcpu(String),
    #[error("Failed to starting kvm vcpu: {0}!")]
    StartVcpu(String),
    #[error("Failed to stopping kvm vcpu: {0}!")]
    StopVcpu(String),
    #[error("Failed to kick kvm vcpu: {0}!")]
    KickVcpu(String),
    #[error("Failed to destroy kvm vcpu: {0}!")]
    DestroyVcpu(String),
    #[error("CPU {0}/KVM halted!")]
    VcpuHltEvent(u8),
    #[error("CPU {0}/KVM received an unexpected exit reason: {1}!")]
    VcpuExitReason(u8, String),
    #[error("CPU {0}/KVM received an unhandled kvm exit event!")]
    UnhandledKvmExit(u8),
    #[error("Vcpu not present in local thread.")]
    VcpuLocalThreadNotPresent,
    #[error("No Machine Interface saved in CPU")]
    NoMachineInterface,
    #[cfg(target_arch = "aarch64")]
    #[error("Failed to get system register: {0}!")]
    GetSysRegister(String),
    #[cfg(target_arch = "aarch64")]
    #[error("Failed to Set system register: {0}!")]
    SetSysRegister(String),
}
