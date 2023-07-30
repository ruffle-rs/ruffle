/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the String Prototype object";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "typeof String.prototype",   "object",   typeof String.prototype );
    array[item++] = Assert.expectEq(  "String.prototype.valueOf()",
                                            "",
                                            (String.prototype.valueOf()).toString() );
    var origStringGetClass = String.prototype.getClass;
    array[item++] = Assert.expectEq( 
                                  "String.prototype.getClass = Object.prototype.toString; String.prototype.getClass()",
                                  "[object Object]",
                                  (String.prototype.getClass = Object.prototype.toString, String.prototype.getClass()) );
    array[item++] = Assert.expectEq(  "String.prototype +''",       "",        String.prototype + '' );
    array[item++] = Assert.expectEq(  "String.prototype.length",    undefined,         String.prototype.length );
    array[item++] = Assert.expectEq(  "String.length",    1,         String.length );

    //restore
    String.prototype.getClass = origStringGetClass;
    return ( array );
}

