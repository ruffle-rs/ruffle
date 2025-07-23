/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package DefaultClass {

  import DefaultClass.*

public class ExtDefaultClassStat extends ExtDefaultClassStatInner {
    public static function pubStatSubGetArray() : Array { return ExtDefaultClassStatInner.pubStatSubGetArray(); }
    public static function pubStatSubSetArray(a:Array) { ExtDefaultClassStatInner.pubStatSubSetArray(a); }
    public static function pubStatSubGetSPArray() : Array { return ExtDefaultClassStatInner.pubStatSubGetSPArray(); }
    public static function pubStatSubSetSPArray(a:Array) { ExtDefaultClassStatInner.pubStatSubSetSPArray(a); }
    public static function testStatSubArray(a:Array) : Array { return ExtDefaultClassStatInner.testStatSubArray(a); }
    public static function testStatSubPArray(a:Array) : Array { return ExtDefaultClassStatInner.testStatSubPArray(a); }
    public static function testPrivStatSubArray(a:Array) : Array { return ExtDefaultClassStatInner.testPrivStatSubArray(a); }
}
}
