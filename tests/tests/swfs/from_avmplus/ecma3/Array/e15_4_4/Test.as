/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the Array Prototype Object";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "Array.prototype.length",   0,          Array.prototype.length );
    array[item++] = Assert.expectEq(   "Array.length",   1,          Array.length );
//  verify that prototype object is an Array object.
    array[item++] = Assert.expectEq(   "typeof Array.prototype",    "object",   typeof Array.prototype );

    var tempToString = Array.prototype.toString;
    array[item++] = Assert.expectEq( 
                                    "Array.prototype.toString = Object.prototype.toString; Array.prototype.toString()",
                                    "[object Array]",
                                    (Array.prototype.toString = Object.prototype.toString, Array.prototype.toString()) );
    
    // revert Array.prototype.toString back to original for ATS tests
    Array.prototype.toString = tempToString;
    
    return ( array );
}
