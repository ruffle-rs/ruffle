/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// ActionScript file
package SuperExprChainAccessPkg {

    class derived extends middle2 {
        override function f() : String {
            return "derived f()";
        }
        override function g() : String {
            return "derived g()";
        }

        public function callf1() : String {
            return f();
        }
        public function callf2() : String {
            return super.f();
        }
        public function callg1() : String {
            return g();
        }
        public function callg2() : String {
            return super.g();
        }
        public function callh1() : String {
            return h();
        }
        public function callh2() : String {
            return super.h();
        }
        public function callh3() : String {
            return callh();
        }
        public function calli1() : String {
            return i();
        }
        public function calli2() : String {
            return super.i();
        }
        public function calli3() : String {
            return calli();
        }
    }

}
