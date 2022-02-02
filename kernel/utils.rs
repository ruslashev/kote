// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub fn is_po2_aligned(value: u64, po2: u64) -> bool {
    value & (po2 - 1) == 0
}

pub fn po2_round_down(value: u64, po2: u64) -> u64 {
    value & !(po2 - 1)
}

pub fn po2_round_up(value: u64, po2: u64) -> u64 {
    (value + po2 - 1) & !(po2 - 1)
}
