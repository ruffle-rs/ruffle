/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1074; // Illegal write to read-only property _ on _.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var z = "no error";
    Object = new Object();
    Object.valueOf = Number.prototype.valueOf;
} catch (err) {
    z = err.toString();
} finally {
    Assert.expectEq("Runtime Error", Utils.REFERENCEERROR + CODE, Utils.referenceError(z));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
