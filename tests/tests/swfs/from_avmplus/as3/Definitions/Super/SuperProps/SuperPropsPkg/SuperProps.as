/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// Access properties via super using . and []
package SuperPropsPkg {

    public dynamic class SuperProps extends SuperBase {
        public function get inheritedProp() : String { return y; }
        public function get superPropDot() : String { return super.y; }
        public function get superPropIndex() : String { return super["x"]; }

        public function set superPropDot(val) : void { super.y = val; }
        public function set superPropIndex(val) : void { super["x"] = val; }

        public function setDerivedVal( y : String, z : String ) { this[y] = z; }
        public function getDerivedVal( y : String ) : String { return this[y]; }
        public function getSuperVal( y : String ) : String { return super[y]; }
    }

}
