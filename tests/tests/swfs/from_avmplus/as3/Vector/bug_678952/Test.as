/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

/*
Running this testcase and the entire as3/Vector/* acceptance tests will provide
almost full coverage of this change:
    http://hg.mozilla.org/tamarin-redux/diff/6f72616eadd7/core/Verifier.cpp

These testcases focus on covering all conditions of this line:
    bool maybeIntegerIndex = !attr && multiname.isRtname() && multiname.containsAnyPublicNamespace();

The only condition that is not being covered is the false branch of:
    multiname.containsAnyPublicNamespace().
 */

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = " ";
// var VERSION = "AS3";

class C
{
    var x: C = null;  // just some field to make the type nontrivial
}

var expected;
var err;
var a: Vector.<C> = new Vector.<C>;

expected = "ReferenceError: Error #1069";
err = "exception not thrown";
try {
    var f = a.foo;
}
catch (e:Error){
    err = e.toString();
}
// http://hg.mozilla.org/tamarin-redux/diff/6f72616eadd7/core/Verifier.cpp#l1.26
Assert.expectEq("ReferenceError for multiname.isRtname() failing",
  expected,
  Utils.parseError(err, expected.length));

expected = "ReferenceError: Error #1081";
err = "exception not thrown";
try {
    // attr -> false
    var g = a.@attr;
}
catch (e:Error){
    err = e.toString();
}
// http://hg.mozilla.org/tamarin-redux/diff/6f72616eadd7/core/Verifier.cpp#l1.26
Assert.expectEq("ReferenceError for !attr failing",
  expected,
  Utils.parseError(err, expected.length));
