/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
1070    Method _ not found on _
*/

var CODE = 1070;

//-----------------------------------------------------------
//-----------------------------------------------------------

class A {}

class B extends A {
  function f() { super.f(); }
}

try {
    var z = "no error";
    new B().f();
} catch (err) {
    z = err.toString();
} finally {
    Assert.expectEq("Runtime Error", "ReferenceError: Error #" + CODE, Utils.referenceError(z));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
