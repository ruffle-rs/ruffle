/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.2.2-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor:  new Array( len )";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var arr;

/*  cn:  these tests are invalid for ES4, where there is no distinction between a number and a Number
       (i.e. 1000 === new Number(1000)
    array[item++] = Assert.expectEq(   "(new Array(new Number(1073741823))).length",   1,      (new Array(new Number(1073741823))).length );
    array[item++] = Assert.expectEq(   "(new Array(new Number(0))).length",            1,      (new Array(new Number(0))).length );
    array[item++] = Assert.expectEq(   "(new Array(new Number(1000))).length",         1,      (new Array(new Number(1000))).length );
*/
    array[item++] = Assert.expectEq(   "(new Array(new Number(1073741823))).length",   1073741823, (new Array(new Number(1073741823))).length );
    array[item++] = Assert.expectEq(   "(new Array(new Number(0))).length",            0,          (new Array(new Number(0))).length );
    array[item++] = Assert.expectEq(   "(new Array(new Number(1000))).length",         1000,       (new Array(new Number(1000))).length );

    array[item++] = Assert.expectEq(   "(new Array('mozilla, larryzilla, curlyzilla')).length", 1, (new Array('mozilla, larryzilla, curlyzilla')).length );
    array[item++] = Assert.expectEq(   "(new Array(true)).length",                     1,      (new Array(true)).length );
    array[item++] = Assert.expectEq(   "(new Array(false)).length",                    1,      (new Array(false)).length);
    array[item++] = Assert.expectEq(   "(new Array(new Boolean(true)).length",         1,      (new Array(new Boolean(true))).length );
    array[item++] = Assert.expectEq(   "(new Array(new Boolean(false)).length",        1,      (new Array(new Boolean(false))).length );
    return ( array );
}
