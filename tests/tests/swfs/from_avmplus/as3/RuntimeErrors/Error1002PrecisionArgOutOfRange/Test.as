/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1002; // The precision argument must be between 0 and 20; _ is invalid.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var result = "no error";
    var n:Number = new Number(1);
    n.toPrecision(0);
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Runtime Error", Utils.RANGEERROR + CODE, Utils.rangeError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
