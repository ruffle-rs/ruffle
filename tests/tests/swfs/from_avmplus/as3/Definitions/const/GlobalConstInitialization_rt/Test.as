/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
 
// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "Initialize a global const after its initializer";       // Provide ECMA section title or a description
var BUGNUMBER = "";


const num1;

var thisError:String = "no error";

try
{
    num1 = 20;
}
catch(err:Error)
{
    thisError = err.toString();
}
finally
{
    Assert.expectEq("Initialize a global const after its initializer.", "ReferenceError: Error #1074", Utils.referenceError(thisError));
}

