/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor Called as a Function";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var arr;

    array[item++] = Assert.expectEq(   
                                    "Array() +''",
                                    "",
                                    Array() +"" );

    array[item++] = Assert.expectEq(   
                                    "typeof Array()",
                                    "object",
                                    typeof Array() );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(); arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (arr = Array(), arr.getClass = Object.prototype.toString, arr.getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(); arr.toString == Array.prototype.toString",
                                    true,
                                    (arr = Array(), arr.toString == Array.prototype.toString ) );

    array[item++] = Assert.expectEq(   
                                    "Array().length",
                                    0,
                                    Array().length );


    array[item++] = Assert.expectEq(   
                                    "Array(1,2,3) +''",
                                    "1,2,3",
                                    Array(1,2,3) +"" );

    array[item++] = Assert.expectEq(   
                                    "typeof Array(1,2,3)",
                                    "object",
                                    typeof Array(1,2,3) );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(1,2,3); arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (arr = Array(1,2,3), arr.getClass = Object.prototype.toString, arr.getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(1,2,3); arr.toString == Array.prototype.toString",
                                    true,
                                    (arr = Array(1,2,3), arr.toString == Array.prototype.toString ) );

    array[item++] = Assert.expectEq(   
                                    "Array(1,2,3).length",
                                    3,
                                    Array(1,2,3).length );

    array[item++] = Assert.expectEq(   
                                    "typeof Array(12345)",
                                    "object",
                                    typeof Array(12345) );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(12345); arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (arr = Array(12345), arr.getClass = Object.prototype.toString, arr.getClass() ) );

    array[item++] = Assert.expectEq(   
                                    "var arr = Array(1,2,3,4,5); arr.toString == Array.prototype.toString",
                                    true,
                                    (arr = Array(1,2,3,4,5), arr.toString == Array.prototype.toString ) );

    array[item++] = Assert.expectEq(   
                                    "Array(12345).length",
                                    12345,
                                    Array(12345).length );

    return ( array );
}
