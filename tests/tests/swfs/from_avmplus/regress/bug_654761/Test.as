/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import com.adobe.test.Assert;
import com.adobe.test.Utils;

function foo()
{
    var i = null;
    var k:int = 1;
    var b = (k in i); // should throw
    return b;
}
var errorMsg:String="No error";
try {
    foo();
} catch (e) {
    errorMsg=Utils.grabError(e,e.toString());
}

Assert.expectEq("regression test bug 654761","Error #1009",errorMsg);

