/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9.3 example (public vs interface name) with additional test
// for user-defined namespaces inside interfaces per section 9.3
package Example_9_3 {
    public class ExampleTest {
        var a : A = new A();
        var b : B = new B();
        public function doTestPublic() : String {
            return a.f();
        }
        public function doTestNS1() : String {
            return a.g();
        }
        public function doTestIName() : String {
            return b.f();
        }
        public function doTestNS2() : String {
            return b.g();
        }
    }
}

