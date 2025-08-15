/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticPropertyPackage {
    public class AccNSStatPropSubClassMeth extends BaseClass {

        public function getInt(): int {
            return ns1::i;
        }

        public function getBaseInt(): int {
            return BaseClass.ns1::i;
        }

    }
}
