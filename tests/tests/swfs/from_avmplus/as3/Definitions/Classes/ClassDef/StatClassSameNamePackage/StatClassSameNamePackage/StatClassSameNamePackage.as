/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// bug 134955: test access of statics from a class in a package with the same name
package StatClassSameNamePackage {
    public class StatClassSameNamePackage {
        public static var aStatic : String  = "x.x.a";
        public static function fStatic() : String {
            return "x.x.f()";
        }
    }
}
