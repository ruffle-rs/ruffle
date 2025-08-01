/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import Package1.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

var c2:Class2 = new Class2();


var thisError = "no error";
try
{
    c2.getClass1ClassItem1();
}
catch(err)
{
    thisError = err.toString();
}
finally
{
    Assert.expectEq("attempt to access private variable of Class1 in Class2", "ReferenceError: Error #1069", Utils.referenceError(thisError));
}
