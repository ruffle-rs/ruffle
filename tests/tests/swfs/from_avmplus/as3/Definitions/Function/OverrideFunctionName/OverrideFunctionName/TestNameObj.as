/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package OverrideFunctionName {

    public class TestNameObj extends TestNameObjBase {
        // constructor
        public function TestNameObj() {}

        // not the constructor but looks like it
        override public function testNameObj() { return "not the constructor" }

        override public function a1 () { return "a1"; }
        override public function a_1 () { return "a_1"; }
        override public function _a1 () { return "_a1"; }
        override public function __a1 () { return "__a1"; }
        override public function _a1_ () { return "_a1_"; }
        override public function __a1__ () { return "__a1__"; }
        override public function $a1 () { return "$a1"; }
        override public function a$1 () { return "a$1"; }
        override public function a1$ () { return "a1$"; }
        override public function A1 () { return "A1"; }
        override public function cases () { return "cases"; }
        override public function Cases () { return "Cases"; }
        override public function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
    }

}

