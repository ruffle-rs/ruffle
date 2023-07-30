/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.1.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Constructor Called as a Function";


    var testcases = getTestCases();

function ToUint32( n ) {
    n = Number( n );
    if( isNaN(n) || n == 0 || n == Number.POSITIVE_INFINITY ||
        n == Number.NEGATIVE_INFINITY ) {
        return 0;
    }
    var sign = n < 0 ? -1 : 1;

    return ( sign * ( n * Math.floor( Math.abs(n) ) ) ) % Math.pow(2, 32);
}

function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;

    array[item++] = Assert.expectEq(   "typeof Array(1,2)",        "object",           typeof Array(1,2) );
    array[item++] = Assert.expectEq(   "(Array(1,2)).toString",    "function Function() {}",    ((Array(1,2)).toString).toString() );


    var thisErr:String = "no error";
    var arr:Array = Array(1,2,3);
    arr.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq( 
                                    "var arr = Array(1,2,3); arr.toString = Object.prototype.toString; arr.toString()","[object Array]",arr.toString());

    array[item++] = Assert.expectEq(   "(Array(1,2)).length",      2,                  (Array(1,2)).length );
    array[item++] = Assert.expectEq(   "var arr = (Array(1,2)), arr[0]",  1,           (arr = (Array(1,2)), arr[0] ) );
    array[item++] = Assert.expectEq(   "var arr = (Array(1,2)), arr[1]",  2,           (arr = (Array(1,2)), arr[1] ) );
    array[item++] = Assert.expectEq(   "var arr = (Array(1,2)), String(arr)",  "1,2",  (arr = (Array(1,2)), String(arr) ) );

    return ( array );
}

