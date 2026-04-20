/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package DefaultClass {

  import DefaultClass.*

public class ExtDefaultClassPubStat extends ExtDefaultClassPubStatInner {
    // **************************
    // public static methods
    // **************************

    public static function setPubStatArray(a:Array) { DefaultClassInner.setPubStatArray(a); }
    public static function setPubStatBoolean( b:Boolean ) { DefaultClassInner.setPubStatBoolean(b); }

    public static function getPubStatArray() { return DefaultClassInner.getPubStatArray(); }

    // ***************************************
    // access public static method of parent
    // from public static method of sub class
    // ***************************************

    public static function pubStatSubGetArray() : Array { return ExtDefaultClassPubStatInner.pubStatSubGetArray(); }
    public static function pubStatSubSetArray(a:Array) { ExtDefaultClassPubStatInner.pubStatSubSetArray(a); }

    public static function testStatSubArray(a:Array) : Array { return ExtDefaultClassPubStatInner.testStatSubArray(a); }
    public static function testStatSubPArray(a:Array) : Array { return ExtDefaultClassPubStatInner.testStatSubPArray(a); }

    public static function pubStatSubGetSPArray() : Array { return ExtDefaultClassPubStatInner.pubStatSubGetSPArray(); }
    public static function pubStatSubSetSPArray(a:Array) { ExtDefaultClassPubStatInner.pubStatSubSetSPArray(a); }
}
}
