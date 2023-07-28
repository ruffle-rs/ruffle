/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.2.2-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor:  new Array( len )";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "new Array(0)",             "",                 (new Array(0)).toString() );
    array[item++] = Assert.expectEq(   "typeof new Array(0)",      "object",           (typeof new Array(0)) );
    array[item++] = Assert.expectEq(   "(new Array(0)).length",    0,                  (new Array(0)).length );
    array[item++] = Assert.expectEq(   "(new Array(0)).toString", "function Function() {}",    ((new Array(0)).toString).toString() );

    array[item++] = Assert.expectEq(    "new Array(1)",            "",                 (new Array(1)).toString() );
    array[item++] = Assert.expectEq(    "new Array(1).length",     1,                  (new Array(1)).length );
    array[item++] = Assert.expectEq(    "(new Array(1)).toString", "function Function() {}",   ((new Array(1)).toString).toString() );

    array[item++] = Assert.expectEq(   "(new Array(-0)).length",                       0,  (new Array(-0)).length );
    array[item++] = Assert.expectEq(   "(new Array(0)).length",                        0,  (new Array(0)).length );

    array[item++] = Assert.expectEq(   "(new Array(10)).length",           10,         (new Array(10)).length );
    array[item++] = Assert.expectEq(   "(new Array('1')).length",          1,          (new Array('1')).length );
    array[item++] = Assert.expectEq(   "(new Array(1000)).length",         1000,       (new Array(1000)).length );
    array[item++] = Assert.expectEq(   "(new Array('1000')).length",       1,          (new Array('1000')).length );

    array[item++] = Assert.expectEq(   "(new Array(4294967295)).length",   ToUint32(4294967295),   (new Array(4294967295)).length );

    array[item++] = Assert.expectEq(   "(new Array('8589934592')).length", 1,                      (new Array("8589934592")).length );
    array[item++] = Assert.expectEq(   "(new Array('4294967296')).length", 1,                      (new Array("4294967296")).length );
    array[item++] = Assert.expectEq(   "(new Array(1073741824)).length",   ToUint32(1073741824),   (new Array(1073741824)).length );

    return ( array );
}

function ToUint32( n ) {
    n = Number( n );
    var sign = ( n < 0 ) ? -1 : 1;

    if ( Math.abs( n ) == 0 || Math.abs( n ) == Number.POSITIVE_INFINITY) {
        return 0;
    }
    n = sign * Math.floor( Math.abs(n) )

    n = n % Math.pow(2,32);

    if ( n < 0 ){
        n += Math.pow(2,32);
    }

    return ( n );
}
