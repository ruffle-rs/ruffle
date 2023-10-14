/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1066; // The form function('function body') is not supported.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var result = "no error";
    var f = new Function( "a","b", "c", "return f.length");
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Runtime Error", "EvalError: Error #" + CODE, Utils.typeError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
