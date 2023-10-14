/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.2.1-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor:  new Array( item0, item1, ...)";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();

    var ARGUMENTS = ""
    var TEST_LENGTH = Math.pow(2,10); //Math.pow(2,32);

    for ( var index = 0; index < TEST_LENGTH; index++ ) {
        ARGUMENTS += index;
        ARGUMENTS += (index == (TEST_LENGTH-1) ) ? "" : ",";
    }

    var TEST_ARRAY = ARGUMENTS.split(",");

    var item;
    for ( item = 0; item < TEST_LENGTH; item++ ) {
        array[item] = Assert.expectEq(  "TEST_ARRAY["+item+"]",     item+"",    TEST_ARRAY[item] );
    }

    array[item++] = Assert.expectEq(  "new Array( ["+TEST_LENGTH+" arguments] ) +''",  ARGUMENTS,          TEST_ARRAY +"" );
    array[item++] = Assert.expectEq(  "TEST_ARRAY.toString", "function Function() {}", (TEST_ARRAY.toString).toString());
    array[item++] = Assert.expectEq(  "TEST_ARRAY.join", "function Function() {}", (TEST_ARRAY.join).toString() );
    array[item++] = Assert.expectEq(  "TEST_ARRAY.sort", "function Function() {}", (TEST_ARRAY.sort).toString() );
    array[item++] = Assert.expectEq(  "TEST_ARRAY.reverse", "function Function() {}", (TEST_ARRAY.reverse).toString());
    array[item++] = Assert.expectEq(  "TEST_ARRAY.length", TEST_LENGTH,  TEST_ARRAY.length);

    TEST_ARRAY.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq( 
                                "TEST_ARRAY.toString = Object.prototype.toString; TEST_ARRAY.toString()",
                                "[object Array]",
                                TEST_ARRAY.toString());

    return ( array );
}
