/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package LengthSpoofing
{
    public dynamic class SpoofingArray extends Array
    {
        var is_spoofing:Boolean;
        var spoofed_length:uint = 100;

        public function SpoofingArray()
        {
            super();
            is_spoofing = false;
        }

        override public function get length():uint
        {
            return ( this.is_spoofing )? spoofed_length: super.length;
        }

        public function set Spoofing(b:Boolean):void
        {
            this.is_spoofing = b;
        }

        public function set spoofedLength(x:uint):void
        {
            this.spoofedLength = x;
        }
    }
}
