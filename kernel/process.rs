// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

pub fn init() {
    println!("{:x?}", &LOOP_ELF[0..4]);
}
