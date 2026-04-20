/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 13.7 super expression - new to AS3: super(object).method(args)
package SuperArgsCallPkg {
    class SuperArgsCallBase {
        private var x : String
        function SuperArgsCallBase( a : String ) {
            x = a
        }
        function f() : String {
            return "base f(" + x + ")"
        }
    }
}

