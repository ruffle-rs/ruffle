/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=555544
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Utils;
import com.adobe.test.Assert;
class A {};

err = "no error";
// looking for ReferenceError: Error #1056: Cannot create property 10 on bug_555544.as$1.A.
try {
    var a = new A();
    a[10] = 0;

} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("bug555544", "Error #1056", err );
}


