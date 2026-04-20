/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package NamespaceFunctionName {

    public class TestNameObj {
        // constructor
        function TestNameObj() {}

        // not the constructor but looks like it
        testns function testNameObj() { return "not the constructor"; }

        testns function a1 () { return "a1"; }
        testns function a_1 () { return "a_1"; }
        testns function _a1 () { return "_a1"; }
        testns function __a1 () { return "__a1"; }
        testns function _a1_ () { return "_a1_"; }
        testns function __a1__ () { return "__a1__"; }
        testns function $a1 () { return "$a1"; }
        testns function a$1 () { return "a$1"; }
        testns function a1$ () { return "a1$"; }
        testns function A1 () { return "A1"; }
        testns function cases () { return "cases"; }
        testns function Cases () { return "Cases"; }
        testns function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
    }

}

