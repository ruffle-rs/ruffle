/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
    public dynamic class Subarray extends Array
    {
        public function Subarray(l) {
            super(l);
        }

        public override function get length():uint
        {
            return super.length;
        }

        public override function set length(newLength:uint)
        {
            super.length = newLength;
        }
    }
}
