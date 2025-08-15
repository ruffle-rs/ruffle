/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 interface can extend multiple interfaces
package ExtendMultipleInterfaces {
    class Y2 implements J2 {
        public function a() {
            return "y2.A::a()";
        }
        public function b() {
            return "y2.B::b()";
        }
        public function c() {
            return "y2.C::c()";
        }
    }
}

