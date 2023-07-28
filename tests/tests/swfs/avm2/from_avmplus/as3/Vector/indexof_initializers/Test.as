/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import com.adobe.test.Assert;
/**
 File Name:    indexof.es
 Description:  indexOf(object,value,from=...)
 compares value with every vector element of object in increasing numerical index order, starting at the
 index from, stopping when an vector lement is equial to value by the === operator, From is rounded toward zero
 before use.  If from is negative, it is treated as object.length+from, returns vector index from first value or -1
 if no such element is found.
 *
 */

// var SECTION = ""
// var VERSION = "ECMA_1";



Assert.expectEq(    "indexOf empty vector",
  -1,
  new <int>[].indexOf(0));

Assert.expectEq(    "indexOf object not found",
  -1,
  new <int>[0,1,2,3,4,5,6,7,8,9].indexOf(10));

Assert.expectEq(    "indexOf single match found",
  4,
  new <int>[0,1,2,3,4,5,6,7,8,9].indexOf(4));

Assert.expectEq(    "indexOf first match found",
  4,
  new <int>[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9].indexOf(4));

Assert.expectEq(    "indexOf first match found setting start parameter",
  4,
  new <int>[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9].indexOf(4,2));

Assert.expectEq(    "indexOf start parameter greater than vector length",
  -1,
  new <int>[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9].indexOf(4,100));

Assert.expectEq(    "indexOf start parameter negative",
  -1,
  new <int>[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9].indexOf(4,-1));