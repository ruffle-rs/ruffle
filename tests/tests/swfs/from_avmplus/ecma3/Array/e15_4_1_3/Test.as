/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.1.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Constructor Called as a Function:  Array()";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   
                                    "typeof Array()",
                                    "object",
                                    typeof Array() );

    var MYARR;

    array[item++] = Assert.expectEq(   
                                    "MYARR = new Array();MYARR.getClass = Object.prototype.toString;MYARR.getClass()",
                                    "[object Array]",
                                    (MYARR = Array(),MYARR.getClass = Object.prototype.toString,MYARR.getClass()) );

    array[item++] = Assert.expectEq(   
                                    "(Array()).length",
                                    0,          (
                                    Array()).length );

    array[item++] = Assert.expectEq(   
                                    "Array().toString()",
                                    "",
                                    Array().toString() );


    return ( array );
}
