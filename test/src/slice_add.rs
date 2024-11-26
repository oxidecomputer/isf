// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use isf::AssemblyInstruction;

isf_macro::isf!("isf/testcase/slice-add.isf");

#[test]
fn slice_add() -> Result<(), anyhow::Error> {
    let mut a = SliceAdd::parse_assembly("add r4 0x1122").unwrap();
    assert_eq!(a.get_src(), 0x1122);
    a.set_src(0x3344);
    assert_eq!(a.get_src(), 0x3344);

    Ok(())
}
