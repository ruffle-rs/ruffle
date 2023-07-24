/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import com.adobe.test.Assert;

/**
 File Name:    in.es
 Description:  test 'in' keyword.
 the exception is a current issue with properties defined in the prototype.
 *
 */

// var SECTION = " ";
// var VERSION = "AS3";

Assert.expectEq(    "in value valid index",
  true,
  (0 in new <int>["zero","one","two","three","four","five"]));
Assert.expectEq(    "in value for empty vector",
  false,
  (0 in new <*>[]));

Assert.expectEq(    "in value valid index does not exist",
  false,
  (6 in new <*>[]));

Assert.expectEq(    "in value valid index in string form",
  true,
  ("2" in new <int>["zero","one","two","three","four","five"]));

err1="no exception";
try {
  Assert.expectEq(    "in value is push function index ",
    true,
    ("push" in new <*>[]));
  Assert.expectEq(    "in value is concat function index ",
    true,
    ("concat" in new <*>[]));
  Assert.expectEq(    "in value negative number index ",
    false,
    (-2 in new <*>[]));
  Assert.expectEq(    "in value decimal index",
    false,
    (1.1 in new <*>[]));
  Assert.expectEq(    "in value decimal in string index",
    false,
    ("1.1" in new <*>[]));
  Assert.expectEq(    "in value valid string",
    false,
    ("string" in new <*>[]));
} catch(e) {
  err1=e.toString();
  Assert.expectEq(    "in throws exception for invalid vector indexes",
    "no exception",
    err1);
}