/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package PrivateFunctionName {

    public class TestNameObj {
    
        private function a1 () { return "a1"; }
        private function a_1 () { return "a_1"; }
        private function _a1 () { return "_a1"; }
        private function __a1 () { return "__a1"; }
        private function _a1_ () { return "_a1_"; }
        private function __a1__ () { return "__a1__"; }
        private function $a1 () { return "$a1"; }
        private function a$1 () { return "a$1"; }
        private function a1$ () { return "a1$"; }
        private function A1 () { return "A1"; }
        private function cases () { return "cases"; }
        private function Cases () { return "Cases"; }
        private function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
    
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

