/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 13.7 super expression - new to AS3: super(object).method(args)
package SuperArgsCallPkg {
    public class SuperArgsCall extends SuperArgsCallBase {
        function SuperArgsCall( s : String ) {
            super( s )
        }
        override function f() : String {
            return "derived f()"
        }
        public function test0() {
            return super(this).f()
        }
        public function test1( o : SuperArgsCall ) {
            return super(o).f()
        }
    }
}

