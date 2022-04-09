// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_SIZE_LARGE: u64 = 2 * 1024 * 1024; // 2 MiB

pub const KERNEL_BASE: u64 = 0xffffffff80000000;

/* Memory layout:
 * ┌───────────────────────────────┐ 0xffffffffffffffff
 * │                               │
 * │                               │
 * │  Identity mapping for kernel  │ 0xffffffff80000000 KERNEL_BASE
 * ├───────────────────────────────┤
 * │   Page allocation structures  │
 * ├───────────────────────────────┤
 * │      Framebuffer mapping      │
 * ├───────────────────────────────┤
 * │                               │
 * │          Kernel heap          │
 *
 * ╵               .               ╵
 * ╵               .               ╵
 *
 * │                               │
 * └───────────────────────────────┘
 */
