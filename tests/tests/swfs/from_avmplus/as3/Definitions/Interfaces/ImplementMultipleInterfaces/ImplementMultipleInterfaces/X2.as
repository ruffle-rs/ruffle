/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 8 / 9 / 16.3.3 a variety of implements multiple and extends plus implements single / multiple
package ImplementMultipleInterfaces {
    class X2 implements A, B {
        public function a() {
            return "x2.A::a()";
        }
        public function b() {
            return "x2.B::b()";
        }
    }
}

