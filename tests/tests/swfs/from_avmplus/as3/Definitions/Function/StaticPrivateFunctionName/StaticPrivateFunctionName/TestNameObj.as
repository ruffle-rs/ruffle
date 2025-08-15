/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package StaticPrivateFunctionName {

    public class TestNameObj {

        // constructor
        function TestNameObj() { res = "EmptyName"; }
    
        // not the constructor but looks like it
        function testNameObjInner() { return "not the constructor" }

        static private function a1 () { return "a1"; }
        static private function a_1 () { return "a_1"; }
        static private function _a1 () { return "_a1"; }
        static private function __a1 () { return "__a1"; }
        static private function _a1_ () { return "_a1_"; }
        static private function __a1__ () { return "__a1__"; }
        static private function $a1 () { return "$a1"; }
        static private function a$1 () { return "a$1"; }
        static private function a1$ () { return "a1$"; }
        static private function A1 () { return "A1"; }
        static private function cases () { return "cases"; }
        static private function Cases () { return "Cases"; }
        static private function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }

        public function pubTestConst() { return testNameObjInner(); }
        public function puba1 () { return a1(); }
        public function puba_1 () { return a_1(); }
        public function pub_a1 () { return _a1(); }
        public function pub__a1 () { return __a1(); }
        public function pub_a1_ () { return _a1_(); }
        public function pub__a1__ () { return __a1__(); }
        public function pub$a1 () { return $a1(); }
        public function puba$1 () { return a$1(); }
        public function puba1$ () { return a1$(); }
        public function pubA1 () { return A1(); }
        public function pubcases () { return cases(); }
        public function pubCases () { return Cases(); }
        public function puball () { return abcdefghijklmnopqrstuvwxyz0123456789$_(); }
    }

}

