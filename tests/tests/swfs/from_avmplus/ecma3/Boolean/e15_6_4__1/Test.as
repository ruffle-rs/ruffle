/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var VERSION = "ECMA_1"
//     var SECTION = "15.6.4-1";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "typeof Boolean.prototype == typeof( new Object )",
                                            true,
                                            typeof Boolean.prototype == typeof( new Object ) );
    array[item++] = Assert.expectEq(  "typeof( Boolean.prototype )",
                                            "object",
                                            typeof(Boolean.prototype) );

    //save original toString
    var origToString:Function = Boolean.prototype.toString;

    Boolean.prototype.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq( 
                                    "Boolean.prototype.toString = Object.prototype.toString; Boolean.prototype.toString()",
                                    "[object Object]",
                                    Boolean.prototype.toString());
    array[item++] = Assert.expectEq(  "Boolean.prototype.valueOf()",
                                            false,
                                            Boolean.prototype.valueOf());

    //restore original toString
    Boolean.prototype.toString = origToString;

    return ( array );
}
