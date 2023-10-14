/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1056; // Cannot create property _ on _.

//-----------------------------------------------------------
//-----------------------------------------------------------

class C {}

try {
    var result = "no error";
    var c:C = new C();
    c.c = 0;
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Runtime Error", Utils.REFERENCEERROR + CODE, Utils.referenceError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
