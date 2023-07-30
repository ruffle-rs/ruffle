/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 Description:
 15.4.5.2 length
 The length property of this Array object is always numerically greater
 than the name of every property whose name is an array index.
 */

// var SECTION = "15.4.5.2-1";
// var VERSION = "ECMA_1";
// var TITLE   = "Vector.length - initializers";



Assert.expectEq(    "length of empty vector",
  0,
  new <Object>[].length);

Assert.expectEq(    "length of initialized vector of size 9",
  9,
  new <int>[1,2,3,4,5,6,7,8,9].length);

Assert.expectEq(    "vector initializer is not-fixed size",
  false,
  new <int>[1,2,3,4,5,6,7,8,9].fixed);
