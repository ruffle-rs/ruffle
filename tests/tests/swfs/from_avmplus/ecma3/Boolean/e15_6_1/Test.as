/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.6.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Boolean constructor called as a function: Boolean( value ) and Boolean()";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "Boolean(1)",         true,   Boolean(1) );
    array[item++] = Assert.expectEq(    "Boolean(0)",         false,  Boolean(0) );
    array[item++] = Assert.expectEq(    "Boolean(-1)",        true,   Boolean(-1) );
    array[item++] = Assert.expectEq(    "Boolean('1')",       true,   Boolean("1") );
    array[item++] = Assert.expectEq(    "Boolean('0')",       true,   Boolean("0") );
    array[item++] = Assert.expectEq(    "Boolean('-1')",      true,   Boolean("-1") );
    array[item++] = Assert.expectEq(    "Boolean(true)",      true,   Boolean(true) );
    array[item++] = Assert.expectEq(    "Boolean(false)",     false,  Boolean(false) );

    array[item++] = Assert.expectEq(    "Boolean('true')",    true,   Boolean("true") );
    array[item++] = Assert.expectEq(    "Boolean('false')",   true,   Boolean("false") );
    array[item++] = Assert.expectEq(    "Boolean(null)",      false,  Boolean(null) );

    array[item++] = Assert.expectEq(    "Boolean(-Infinity)", true,   Boolean(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "Boolean(NaN)",       false,  Boolean(Number.NaN) );
    array[item++] = Assert.expectEq(    "Boolean(void(0))",   false,  Boolean( void(0) ) );
    array[item++] = Assert.expectEq(    "Boolean(x=0)",       false,  Boolean( x=0 ) );
    array[item++] = Assert.expectEq(    "Boolean(x=1)",       true,   Boolean( x=1 ) );
    array[item++] = Assert.expectEq(    "Boolean(x=false)",   false,  Boolean( x=false ) );
    array[item++] = Assert.expectEq(    "Boolean(x=true)",    true,   Boolean( x=true ) );
    array[item++] = Assert.expectEq(    "Boolean(x=null)",    false,  Boolean( x=null ) );
    array[item++] = Assert.expectEq(    "Boolean()",          false,  Boolean() );
//    array[item++] = Assert.expectEq(    "Boolean(var someVar)",     false,  Boolean( someVar ) );

    return ( array );
}
