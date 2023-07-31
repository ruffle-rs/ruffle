/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1003; // The radix argument must be between 2 and 36; got _.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var result = "no error";
    var n:Number = new Number(1);
    n.toString(1);
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Runtime Error", Utils.RANGEERROR + CODE, Utils.rangeError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
