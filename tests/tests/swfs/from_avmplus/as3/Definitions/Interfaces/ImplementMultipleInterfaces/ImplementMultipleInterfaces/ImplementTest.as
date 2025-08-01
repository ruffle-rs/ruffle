/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 8 / 9 / 16.3.3 a variety of implements multiple and extends plus implements single / multiple
package ImplementMultipleInterfaces {
    public class ImplementTest {
        var x1 : X1 = new X1();
        var x2 : X2 = new X2();
        var x3 : X3 = new X3();
        var x4 : X4 = new X4();
        var y1 : Y1 = new Y1();
        var y2 : Y2 = new Y2();
        var y3 : Y3 = new Y3();
        var y4 : Y4 = new Y4();
        public function doTestX1() : String {
            return x1.a();
        }
        public function doTestX2() : String {
            return x2.a() + "," + x2.b();
        }
        public function doTestX3() : String {
            return x3.a() + "," + x3.b() + "," + x3.c();
        }
        public function doTestX4() : String {
            return x4.a() + "," + x4.b() + "," + x4.c() + "," + x4.d();
        }
        public function doTestY1() : String {
            return y1.a() + "," + y1.b();
        }
        public function doTestY2() : String {
            return y2.a() + "," + y2.b() + "," + y2.c();
        }
        public function doTestY3() : String {
            return y3.a() + "," + y3.b() + "," + y3.c() + "," + y3.d();
        }
        public function doTestY4() : String {
            return y4.a() + "," + y4.b() + "," + y4.c() + "," + y4.d();
        }
    }
}

