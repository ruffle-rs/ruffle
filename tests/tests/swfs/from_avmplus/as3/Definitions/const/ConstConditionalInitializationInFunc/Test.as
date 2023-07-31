/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "conditional initialization of const inside a function";       // Provide ECMA section title or a description
var BUGNUMBER = "";

function getNumber(cond:Boolean):Number
{
    const num1:Number = (cond)? 3 : -3;
    return num1;
}



Assert.expectEq("Conditional initiailization of const inside a function", -3, getNumber(false));

