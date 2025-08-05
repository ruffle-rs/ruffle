/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 interface can extend multiple interfaces
package ExtendMultipleInterfaces {
    class Y3 implements J3 {
        public function a() {
            return "y3.A::a()";
        }
        public function b() {
            return "y3.B::b()";
        }
        public function c() {
            return "y3.C::c()";
        }
        public function d() {
            return "y3.I3::d()";
        }
    }
}

