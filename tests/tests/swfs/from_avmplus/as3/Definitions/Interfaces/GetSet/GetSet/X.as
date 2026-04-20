/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 get / set allowed in interfaces
package GetSet {
    class X implements A {
        var a_ : String = "unset";
        var c_ : String = "unset";
        public function get a() : String {
            return "x.A::get a()";
        }
        public function set a(x:String) : void {
            a_ = "x.A::set a()";
        }
        public function getA() : String {
            return a_;
        }
        public function get b() : String {
            return "x.A::get b()";
        }
        public function set c(x:String) : void {
            c_ = "x.A::set c()";
        }
        public function getC() : String {
            return c_;
        }
    }
}

