/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// ActionScript file
package SuperExprChainAccessPkg {

    class middle2 extends middle1 {
        override function h() : String {
            return "middle2 h()";
        }
        override function i() : String {
            return "middle2 i()";
        }
        function callh() : String {
            return super.h();
        }
        function calli() : String {
            return super.i();
        }
    }

}
