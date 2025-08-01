/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// 9 get / set allowed in interfaces
package GetSet {
    interface A {
        function get a() : String;
        function set a(aa:String) : void;
        function get b() : String;
        function set c(cc:String) : void;
    }
}

