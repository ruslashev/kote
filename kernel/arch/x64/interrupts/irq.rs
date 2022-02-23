// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::pic;

#[no_mangle]
pub extern "C" fn irq_dispatch(vec: u8) {
    println!("In IRQ {} handler", vec);

    pic::irq_eoi(vec);
}
