/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticProperty {

    public class AccStatPropSubClassInit extends BaseClass {
        public var aVar : String = BaseClass.x;
        public static var aStat : String = BaseClass.x;

        public var aVar2 : String = x;
        public static var aStat2 : String = x;
    }
}
