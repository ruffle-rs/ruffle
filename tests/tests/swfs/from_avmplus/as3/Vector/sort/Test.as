/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

/**
 File Name:          sort.as
 ECMA Section:       Vector.sort(comparefn)
 Description:


 Author:             christine@netscape.com 12 Nov 1997
 Updated:            dschaffe@adobe.com 2 Nov 2007
 */

// TODO: REVIEW AS4 CONVERSION ISSUE
// var SECTION = "";
// var VERSION = "ECMA_1";
// var TITLE   = "Vector.sort(comparefn)";


// sort int vector
var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=9-i;
var v2=v1.sort(Compare);
CheckItems(v1,v2,"sort()");

// sort uint vector
var v1=new Vector.<uint>();
for (var i=0;i<10;i++) v1[i]=9-i;
var v2=v1.sort(Compare);
CheckItems(v1,v2,"sort()");

// sort string vector
var vs1=new Vector.<String>();
for (var i=0;i<10;i++) vs1[i]="string"+(9-i);
var vs2=vs1.sort(Compare);
CheckItems(vs1,vs2,"sort()");

var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=9-i;
var errormsg="";
try {
  v1.sort()
} catch (e) {
  errormsg=e.toString();
}

//if (as3Enabled) {
//  Assert.expectEq(
// "sort vector without setting compare function throws exception",
// "ArgumentError: Error #1063",
//  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));
//} else {
//  Assert.expectEq(
// "sort vector without setting compare function throws exception",
// "TypeError: Error #1034",
//  Utils.parseError(errormsg,"TypeError: Error #1034".length));
//}

class TestClass {
  private var myVal:Object;
  public function TestClass(v:Object):void {
    myVal = v;
  }
  public function toString():String {
    return myVal.toString();
  }

  public function get val():Object {
    return myVal;
  }
}

function testClassReverseSort(x:TestClass, y:TestClass) {
  if (x.val < y.val)
    return 1
  if (x.val == y.val)
    return 0
  return -1
}

var testClassVector = new Vector.<TestClass>();
for (var i=0; i<20; i++) {
  testClassVector.push(new TestClass(i));
}
// push one duplicate value
testClassVector.push(new TestClass(12));

Assert.expectEq("Sort a custom vector",
  "19,18,17,16,15,14,13,12,12,11,10,9,8,7,6,5,4,3,2,1,0",
  testClassVector.sort(testClassReverseSort).toString()
);

// Since we are not sorting on val, does an alpha sort
Assert.expectEq("Custom vector sort - object (alpha) sort",
  "0,1,10,11,12,12,13,14,15,16,17,18,19,2,3,4,5,6,7,8,9",
  testClassVector.sort(function(x,y) {if (x>y) return 1; if (x==y) return 0; return -1}).toString()
);

// regular sort returning non-standard values
var mySortFunction:Function = function (x,y):Number {
  if (x.val < y.val)
    return -Infinity;
  if (x.val > y.val)
    return uint.MAX_VALUE;
  return undefined;
}

Assert.expectEq("Custom vector sort using sort function with non-standard values",
  "0,1,2,3,4,5,6,7,8,9,10,11,12,12,13,14,15,16,17,18,19",
  testClassVector.sort(mySortFunction).toString()
);


function CheckItems( A, E, desc) {
  Assert.expectEq(
    desc+" after sort, compare lengths",
    E.length,
    A.length);
  Assert.expectEq(
    desc+ " after sort, compare items",
    E.toString(),
    A.toString());

}
function Sort( a ) {
  for ( i = 0; i < a.length; i++ ) {
    for ( j = i+1; j < a.length; j++ ) {
      var lo = a[i];
      var hi = a[j];
      var c = Compare( lo, hi );
      if ( c == 1 ) {
        a[i] = hi;
        a[j] = lo;
      }
    }
  }
  return a;
}
function Compare( x, y ) {
  if ( x == void 0 && y == void 0  && typeof x == "undefined" && typeof y == "undefined" ) {
    return +0;
  }
  if ( x == void 0  && typeof x == "undefined" ) {
    return 1;
  }
  if ( y == void 0 && typeof y == "undefined" ) {
    return -1;
  }
  x = String(x);
  y = String(y);
  if ( x < y ) {
    return -1;
  }
  if ( x > y ) {
    return 1;
  }
  return 0;
}
