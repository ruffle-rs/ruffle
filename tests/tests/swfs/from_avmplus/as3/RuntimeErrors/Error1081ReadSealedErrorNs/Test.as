/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
    import flash.display.MovieClip; public class Test extends MovieClip {}
}



import Main;
import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1081; // Property name not found on name and there is no default value.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var m = new Main();
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Reference Error", Utils.REFERENCEERROR + CODE, Utils.referenceError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
