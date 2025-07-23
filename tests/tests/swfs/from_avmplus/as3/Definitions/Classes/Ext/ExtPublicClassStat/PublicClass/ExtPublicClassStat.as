/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



package PublicClass {

  import PublicClass.*;

public class ExtPublicClassStat extends ExtPublicClassStatInner {
    public static function testStatSubArray(a:Array) : Array { return ExtPublicClassStatInner.testStatSubArray(a); }
    public static function pubStatSubGetArray() : Array { return ExtPublicClassStatInner.pubStatSubGetArray(); }
    public static function pubStatSubSetArray(a:Array) { ExtPublicClassStatInner.pubStatSubSetArray(a); }
    public static function testStatSubPArray(a:Array) : Array { return ExtPublicClassStatInner.testStatSubPArray(a); }
    public static function pubStatSubGetSPArray() : Array { return ExtPublicClassStatInner.pubStatSubGetSPArray(); }
    public static function pubStatSubSetSPArray(a:Array) { ExtPublicClassStatInner.pubStatSubSetSPArray(a); }
}
}
