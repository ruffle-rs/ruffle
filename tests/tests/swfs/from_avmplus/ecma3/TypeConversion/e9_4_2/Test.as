/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.4-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "ToInteger";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // some special cases
    td = new Date(Number.NaN);
    array[item++] = Assert.expectEq(   "td = new Date(Number.NaN); td.valueOf()",  Number.NaN, td.valueOf() );
    td = new Date(Infinity);
    array[item++] = Assert.expectEq(   "td = new Date(Infinity); td.valueOf()",    Number.NaN, td.valueOf() );
    td = new Date(-Infinity);
    array[item++] = Assert.expectEq(   "td = new Date(-Infinity); td.valueOf()",   Number.NaN, td.valueOf() );
    td = new Date(-0);
    array[item++] = Assert.expectEq(   "td = new Date(-0); td.valueOf()",          -0,         td.valueOf() );
    td = new Date(0);
    array[item++] = Assert.expectEq(   "td = new Date(0); td.valueOf()",           0,          td.valueOf() );

    // value is not an integer
    td = new Date(3.14159);
    array[item++] = Assert.expectEq(   "td = new Date(3.14159); td.valueOf()",     3,          td.valueOf() );
    td = new Date(Math.PI);
    array[item++] = Assert.expectEq(   "td = new Date(Math.PI); td.valueOf()",     3,          td.valueOf() );
    td = new Date(-Math.PI);
    array[item++] = Assert.expectEq(   "td = new Date(-Math.PI);td.valueOf()",     -3,         td.valueOf() );
    td = new Date(3.14159e2);
    array[item++] = Assert.expectEq(   "td = new Date(3.14159e2); td.valueOf()",   314,        td.valueOf());
    td = new Date(.692147e1);
    array[item++] = Assert.expectEq(   "td = new Date(.692147e1); td.valueOf()",   6,          td.valueOf() );
    td = new Date(-.692147e1);
    array[item++] = Assert.expectEq(   "td = new Date(-.692147e1);td.valueOf()",   -6,         td.valueOf() );

    // value is not a number
    td = new Date(true);
    array[item++] = Assert.expectEq(   "td = new Date(true); td.valueOf()",        1,          td.valueOf() );
    td = new Date(false);
    array[item++] = Assert.expectEq(   "td = new Date(false); td.valueOf()",       0,          td.valueOf());
    td = new Date(new Number(Math.PI));
    array[item++] = Assert.expectEq(   "td = new Date(new Number(Math.PI)); td.valueOf()",  3, td.valueOf() );
    td = new Date(new Number(Math.PI));
    array[item++] = Assert.expectEq(   "td = new Date(new Number(Math.PI)); td.valueOf()",  3, td.valueOf() );

    // edge cases
    td = new Date(8.64e15);
    array[item++] = Assert.expectEq(   "td = new Date(8.64e15); td.valueOf()",     8.64e15,    td.valueOf() );
    td = new Date(-8.64e15);
    array[item++] = Assert.expectEq(   "td = new Date(-8.64e15); td.valueOf()",    -8.64e15,   td.valueOf() );
    td = new Date(8.64e-15);
    array[item++] = Assert.expectEq(   "td = new Date(8.64e-15); td.valueOf()",    0,          td.valueOf() );
    td = new Date(-8.64e-15);
    array[item++] = Assert.expectEq(   "td = new Date(-8.64e-15); td.valueOf()",   0,          td.valueOf() );

    return ( array );
}
