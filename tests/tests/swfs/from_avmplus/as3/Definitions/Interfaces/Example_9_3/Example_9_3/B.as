/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9.3 example (public vs interface name) with additional test
// for user-defined namespaces inside interfaces per section 9.3
package Example_9_3 {
    class B implements T {
        public function f() {
            return "b.T::f()";
        }
        public function g() {
            return "b.g()";
        }
    }

}

