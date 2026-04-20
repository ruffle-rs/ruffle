/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package SuperImplicitlyCalledPackage {
    public class SuperImplicitlyCalled {
        private static var x : Number = 0
        function SuperImplicitlyCalled() {
            x = x + 1
        }
        public static function howManyObjects() : Number {
            return x
        }
    }
}
