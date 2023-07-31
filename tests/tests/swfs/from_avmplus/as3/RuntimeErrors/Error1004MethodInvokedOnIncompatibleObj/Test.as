/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1004; // Method '_' was invoked on an incompatible object.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var z = "no error";
    var object = new Object();
    object.valueOf = Number.prototype.valueOf;
    var result = object.valueOf();
} catch (err) {
    z = err.toString();
} finally {
    Assert.expectEq("Runtime Error", "TypeError: Error #" + CODE, Utils.typeError(z));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
