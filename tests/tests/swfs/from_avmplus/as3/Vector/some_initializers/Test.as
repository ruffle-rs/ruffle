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
 Description:  The elements of this object are converted to strings and
 these strings are then concatenated, separated by comma
 characters. The result is the same as if the built-in join
 method were invoiked for this object with no argument.
 */

// var SECTION = "";
// var VERSION = "AS3";


function greaterThanTen(item, index, vec):Boolean {
  if (item > 10) {
    return true;
  }
  return false;
}

var errormsg="";
try {
  result=new <int>[].some();
} catch (e) {
  errormsg=e.toString();
}

Assert.expectEq(
  "some no checker",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));

var checker2="a string";
var errormsg="";
try {
  result=new<int>[1,2,3].some(checker2);
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "some checker not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

Assert.expectEq(
  "some empty vector result",
  false,
  new<int>[].some(greaterThanTen));

Assert.expectEq(
  "some vector with no match",
  false,
  new<int>[-3000,2,3,4,5,-1,9, 10].some(greaterThanTen));

Assert.expectEq(
  "some vector with match",
  true,
  new<int>[-3000,2,3,22,4,5,-1,9, 10, 3].some(greaterThanTen));

function noReturnValue(item, index, vec) {
  // do nothing
}

Assert.expectEq(
  "some function with no return value",
  false,
  new<String>['asdf','hello','out','there'].some(noReturnValue));
