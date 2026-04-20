/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// Access properties via super using . and []
package SuperPropsPkg {

    public dynamic class SuperBase {
        public var x : String = "base::staticX";
        var y : String = "base::staticY";
        function SuperBase() {
            this["x"] = "base::dynamicX";
        }
        public function get baseProp() : String { return y; }
        public function setBaseVal( y : String, z : String ) { this[y] = z; }
        public function getBaseVal( y : String ) : String { return this[y]; }
    }

}
