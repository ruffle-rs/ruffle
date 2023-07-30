/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.7.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the Number Prototype Object";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var origToString:Function = Number.prototype.toString;
    
    Number.prototype.toString=Object.prototype.toString;
    array[item++] = Assert.expectEq( 
                                  "Number.prototype.toString=Object.prototype.toString;Number.prototype.toString()",
                                  "[object Object]",
                                  Number.prototype.toString() );
    array[item++] = Assert.expectEq(  "typeof Number.prototype",                           "object",    typeof Number.prototype );
    array[item++] = Assert.expectEq(  "Number.prototype.valueOf()",                        0,          Number.prototype.valueOf() );

    //restore original prototype
    Number.prototype.toString = origToString;
    
    return ( array );
}
