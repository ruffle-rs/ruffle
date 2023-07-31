/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1084; // Element or attribute ("_") does not match QName production: QName::=(NCName':')?NCName.

//-----------------------------------------------------------
//-----------------------------------------------------------

try {
    var result = "no error";
    var x = "<::/>";
    var y = new XML(x);
} catch (err) {
    result = err.toString();
} finally {
    Assert.expectEq("Runtime Error", Utils.TYPEERROR + CODE, Utils.typeError(result));
}

//-----------------------------------------------------------
//-----------------------------------------------------------
