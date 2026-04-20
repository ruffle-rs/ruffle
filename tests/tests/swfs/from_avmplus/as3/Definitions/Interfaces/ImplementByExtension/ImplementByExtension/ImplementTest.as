/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// Verify that the implementation of one or more interfaces can
// be assembled via inheritance from one or more classes
package ImplementByExtension {
    public class ImplementTest {
        var a1 : A1 = new A1;
        var a2 : A2 = new A2;
        var a3 : A3 = new A3;
        var a4 : A4 = new A4;
        var a5 : A5 = new A5;
        var a6 : A6 = new A6;
        var b1 : B1 = new B1;
        var b2 : B2 = new B2;
        var b3 : B3 = new B3;
        var c1 : C1 = new C1;
        var c2 : C2 = new C2;
        var c3 : C3 = new C3;
        var c4 : C4 = new C4;
        var c5 : C5 = new C5;
        var c6 : C6 = new C6;
        var c7 : C7 = new C7;
        var c8 : C8 = new C8;
        var c9 : C9 = new C9;
        public function doCallAF() : String {
            return a1.f() + "," + a2.f() + "," + a3.f() + "," + a4.f() + "," + a5.f() + "," + a6.f();
        }
        public function doCallAG() : String {
            return a5.g() + "," + a6.g();
        }
        public function doCallBF() : String {
            return b1.f() + "," + b2.f() + "," + b3.f();
        }
        public function doCallBG() : String {
            return b1.g() + "," + b2.g() + "," + b3.g();
        }
        public function doCallCF() : String {
            return c1.f() + "," + c2.f() + "," + c3.f() + "," + c4.f() + "," + c5.f() + "," + c6.f() + "," + c7.f() + "," + c8.f() + "," + c9.f();
        }
        public function doCallCG() : String {
            return c1.g() + "," + c2.g() + "," + c3.g() + "," + c4.g() + "," + c5.g() + "," + c6.g() + "," + c7.g() + "," + c8.g() + "," + c9.g();
        }
    }
}

