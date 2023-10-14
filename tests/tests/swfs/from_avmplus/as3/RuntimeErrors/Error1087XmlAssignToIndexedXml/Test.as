/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var CODE = 1087; // Assignment to indexed XML is not allowed.

//-----------------------------------------------------------
//-----------------------------------------------------------

var expected = "Error #" + CODE;
var result = "no error";
try {
    var xml = new XML("<x><a>1</a><b>2</b><c>3</c></x>");
    xml[0] = 5;
} catch (err) {
    result = Utils.grabError(err, err.toString());
} finally {
    Assert.expectEq("Runtime Error", expected, result);
}

//-----------------------------------------------------------
//-----------------------------------------------------------
