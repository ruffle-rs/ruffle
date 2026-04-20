/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package OverrideFunctionName {

    public class TestNameObjBase {
        // constructor
        public function TestNameObjBase() {}

        // not the constructor but looks like it
        public function testNameObj() { return null; }

        public function a1 () { return null; }
        public function a_1 () { return null; }
        public function _a1 () { return null; }
        public function __a1 () { return null; }
        public function _a1_ () { return null; }
        public function __a1__ () { return null; }
        public function $a1 () { return null; }
        public function a$1 () { return null; }
        public function a1$ () { return null; }
        public function A1 () { return null; }
        public function cases () { return null; }
        public function Cases () { return null; }
        public function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return null; }
    }

}

