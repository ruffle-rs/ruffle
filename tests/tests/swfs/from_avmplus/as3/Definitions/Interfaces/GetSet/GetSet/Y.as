/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 get / set allowed in interfaces
package GetSet {
    class Y implements A, B {
        var a_ : String = "unset";
        var b_ : String = "unset";
        var c_ : String = "unset";
        public function get a() : String {
            return "y.A::get a()";
        }
        public function set a(x:String) : void {
            a_ = x;
        }
        public function getA() : String {
            return a_;
        }
        public function get b() : String {
            return "y.A::get b()";
        }
        public function set b(x:String) : void {
            b_ = x;
        }
        public function getB() : String {
            return b_;
        }
        public function get c() : String {
            return "y.B::get c()";
        }
        public function set c(x:String) : void {
            c_ = x;
        }
        public function getC() : String {
            return c_;
        }
    }

}

