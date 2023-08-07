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

#[cfg(not(target_env = "musl"))]
pub mod console;
pub mod error;
#[cfg(feature = "gtk")]
pub mod gtk;
pub mod input;
#[cfg(not(target_env = "musl"))]
pub mod pixman;
pub mod utils;
#[cfg(feature = "vnc")]
pub mod vnc;

mod data;
