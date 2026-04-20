/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticPropertyPackage {

    public class AccStatPropSubClassStatMeth extends BaseClass {

        public static function getInt(): int {
            return i;
        }

        public static function getBaseInt(): int {
            return BaseClass.i;
        }

    }
}
