/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import Package1.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;


var c1 = new Class1();

var thisError = "no error";

try
{
    c1.item1 = "updated";
}
catch(err)
{
    thisError = err.toString();
}
finally
{
    Assert.expectEq("Attempt to modified class variable in an object instance", "ReferenceError: Error #1056", Utils.referenceError(thisError));
}

