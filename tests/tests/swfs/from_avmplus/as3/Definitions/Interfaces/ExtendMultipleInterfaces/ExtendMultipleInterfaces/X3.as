/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 interface can extend multiple interfaces
package ExtendMultipleInterfaces {
    class X3 implements I3 {
        public function c() {
            return "x3.C::c()";
        }
        public function d() {
            return "x3.I3::d()";
        }
    }
}

