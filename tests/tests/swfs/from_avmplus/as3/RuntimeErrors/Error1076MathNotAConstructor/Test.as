/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
1076    Math is not a constructor.
*/

var CODE = 1076;

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var z = "no error";
    new Math();
} catch (err) {
    z = err.toString();
} finally {
    Assert.expectEq("Runtime Error", "TypeError: Error #" + CODE, Utils.typeError(z));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
