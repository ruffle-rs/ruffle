/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Regression Tests";
// var VERSION = "";
// var TITLE = "Bug 483783: crash when creating large strings";


var myString:String = "";
for(var j:Number = 0; j < 3000000; j++)
    myString += "a";

Assert.expectEq("Verify large string is created",
            3000000,
            myString.length
            );

