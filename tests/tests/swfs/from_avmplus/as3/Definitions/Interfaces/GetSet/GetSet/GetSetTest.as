/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 get / set allowed in interfaces
package GetSet {

    public class GetSetTest {
        var x : X = new X();
        var y : Y = new Y();
        public function doGetAX() : String {
            return x.a;
        }
        public function doSetAX() : String {
            x.a = "ignored";
            return x.getA();
        }
        public function doGetBX() : String {
            return x.b;
        }
        public function doSetCX() : String {
            x.c = "ignored";
            return x.getC();
        }
        public function doGetAY() : String {
            return y.a;
        }
        public function doSetAY() : String {
            y.a = "y.A::set a()";
            return y.getA();
        }
        public function doGetBY() : String {
            return y.b;
        }
        public function doSetBY() : String {
            y.b = "y.B::set b()";
            return y.getB();
        }
        public function doGetCY() : String {
            return y.c;
        }
        public function doSetCY() : String {
            y.c = "y.A::set c()";
            return y.getC();
        }
    }
}

