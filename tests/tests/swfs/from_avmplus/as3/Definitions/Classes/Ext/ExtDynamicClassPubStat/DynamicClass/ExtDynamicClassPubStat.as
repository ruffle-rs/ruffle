/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package DynamicClass {

 import DynamicClass.*;

public class ExtDynamicClassPubStat extends ExtDynamicClassPubStatInner {
    static function setStatArray(a:Array) { DynamicClassInner.setStatArray(a); }
    static function setStatBoolean( b:Boolean ) { DynamicClassInner.setStatBoolean(b); }
    static function getStatArray() { return DynamicClassInner.getStatArray(); }
    public static function setPubStatArray(a:Array) { DynamicClassInner.setPubStatArray(a); }
    public static function setPubStatBoolean( b:Boolean ) { DynamicClassInner.setPubStatBoolean(b); }
    public static function getPubStatArray() { return DynamicClassInner.getPubStatArray(); }
    public static function testStatSubSetArray(a:Array) : Array { return ExtDynamicClassPubStatInner.testStatSubSetArray(a); }
    public static function pubStatSubGetArray() : Array { return ExtDynamicClassPubStatInner.pubStatSubGetArray(); }
    public static function pubStatSubSetArray(a:Array) { ExtDynamicClassPubStatInner.pubStatSubSetArray(a); }
    public static function testStatSubSetDPArray(a:Array) : Array { return ExtDynamicClassPubStatInner.testStatSubSetDPArray(a); }
    public static function pubStatSubGetSPArray() : Array { return ExtDynamicClassPubStatInner.pubStatSubGetSPArray(); }
    public static function pubStatSubSetSPArray(a:Array) { ExtDynamicClassPubStatInner.pubStatSubSetSPArray(a); }
}

}
