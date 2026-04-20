/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticPropertyPackage {

    public class AccStatPropIntermediateSubClassMeth extends IntermediateClass {

        public function getString() : String {
            return string;
        }

        public function getBaseString() : String {
            return BaseClass.string;
        }


    }
}
