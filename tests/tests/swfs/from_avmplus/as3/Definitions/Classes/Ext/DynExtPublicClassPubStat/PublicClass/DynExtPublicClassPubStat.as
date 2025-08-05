/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package PublicClass {

        import PublicClass.*;

    public class DynExtPublicClassPubStat extends DynExtPublicClassPubStatInner {
    // **************************
    // public static methods
    // **************************

    public static function setPubStatArray(a:Array) { PublicClass.setPubStatArray(a); }
    public static function setPubStatBoolean( b:Boolean ) { PublicClass.setPubStatBoolean(b); }

    public static function getPubStatArray() { return PublicClass.getPubStatArray(); }

        // ***************************************
        // access public static property from
        // public static method of sub class
        // ***************************************

        public static function pubStatSubGetSPArray() : Array { return DynExtPublicClassPubStatInner.pubStatSubGetSPArray(); }
        public static function pubStatSubSetSPArray(a:Array) { DynExtPublicClassPubStatInner.pubStatSubSetSPArray(a); }

        // ***************************************
        // access public static method of parent
        // from public static method of sub class
        // ***************************************

        public static function pubStatSubGetArray() : Array { return DynExtPublicClassPubStatInner.pubStatSubGetArray(); }
        public static function pubStatSubSetArray(a:Array) { DynExtPublicClassPubStatInner.pubStatSubSetArray(a); }

    }
}
