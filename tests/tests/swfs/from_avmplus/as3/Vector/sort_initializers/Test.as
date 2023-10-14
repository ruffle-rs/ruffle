/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
// TODO: REVIEW AS4 CONVERSION ISSUE
/**
 File Name:          sort.as
 ECMA Section:       Vector.sort(comparefn)
 Description:


 Author:             christine@netscape.com 12 Nov 1997
 Updated:            dschaffe@adobe.com 2 Nov 2007
 */


// var SECTION = "";
// var VERSION = "ECMA_1";
// var TITLE   = "Vector.sort(comparefn)";


var errormsg="";
try {
  new<int>[4,92,1].sort()
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



Assert.expectEq(
  "sort vector",
  "-12,2,17,56,999",
  new<Number>[999,2,56,-12,17].sort(Compare).toString());



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
  if ( x < y ) {
    return -1;
  }
  if ( x > y ) {
    return 1;
  }
  return 0;
}
