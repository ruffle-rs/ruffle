/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticPropertyPackage {
    public class AccStatPropViaSubClassWIntermediate extends IntermediateClass {

        public function getString() : String {
            // modified for d359, scorfield, 8/1/05:
            //return IntermediateClass.string;
            // should be visible without qualification:
            return string;
        }
    }
}
