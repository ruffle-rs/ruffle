/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 / 16.4 instances of classes that implement an interface belong
// to the type represented by the interface
package InterfaceAsType {
    public class TypeTest {
        var ax : A = new X();
        var bx : B = new X();
        var x : X = new X();
        var ay : A = new Y();
        var by : B = new Y();
        var c : C = new Y();
        var y : Y = new Y();
        public function doCallXViaA() : String {
            return ax.a();
        }
        public function doCallXViaB() : String {
            return bx.b();
        }
        private function callArgA(aa:A) : String {
            return aa.a();
        }
        private function callArgB(bb:B) : String {
            return bb.b();
        }
        public function doCallXViaArgs() : String {
            return callArgA(x) + "," + callArgB(x);
        }
        private function returnXAsA() : A {
            return new X();
        }
        private function returnXAsB() : B {
            return new X();
        }
        public function doCallXViaReturn() : String {
            return returnXAsA().a() + "," + returnXAsB().b();
        }

        public function doCallYViaA() : String {
            return ay.a();
        }
        public function doCallYViaB() : String {
            return by.b();
        }
        public function doCallYViaC() : String {
            return c.a() + "," + c.b();
        }
        private function callArgC(cc:C) : String {
            return cc.a() + "," + cc.b();
        }
        public function doCallYViaArgs() : String {
            return callArgA(y) + "," + callArgB(y);
        }
        public function doCallYViaArgC() : String {
            return callArgC(y);
        }
        private function returnYAsA() : A {
            return new Y();
        }
        private function returnYAsB() : B {
            return new Y();
        }
        private function returnYAsC() : C {
            return new Y();
        }
        public function doCallYViaReturn() : String {
            return returnYAsA().a() + "," + returnYAsB().b();
        }
        public function doCallYViaReturnC() : String {
            return returnYAsC().a() + "," + returnYAsC().b();
        }
    }
}

