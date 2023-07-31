/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
1069    Property _ not found on _ and there is no default value.
*/

var CODE = 1069;

//-----------------------------------------------------------
//-----------------------------------------------------------

class C {}

try {
    var z = "no error";
    var c = new C();
    c.d();
} catch (err) {
    z = err.toString();
} finally {
    Assert.expectEq("Runtime Error", "ReferenceError: Error #" + CODE, Utils.referenceError(z));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
