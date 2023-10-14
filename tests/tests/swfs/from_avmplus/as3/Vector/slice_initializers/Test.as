/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION="";
// var VERSION = "ECMA_1";



Assert.expectEq(
  "slice no args on empty vector",
  "",
  new<String>[].slice().toString());

Assert.expectEq(
  "slice startIndex only",
  "6,7,8,9",
  new<int>[0,1,2,3,4,5,6,7,8,9].slice(6).toString());

Assert.expectEq(
  "slice -1 to -1",
  "",
  new<int>[0,1,2,3,4,5,6,7,8,9].slice(-1,-1).toString());

Assert.expectEq(
  "slice -2 to -1",
  "8",
  new<int>[0,1,2,3,4,5,6,7,8,9].slice(-2,-1).toString());

Assert.expectEq(
  "verify return type",
  true,
  new<Number>[3.14,2.73,9999,.0001,1e13].slice(3,-1) is Vector.<Number>)
