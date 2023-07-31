/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "as3";
// var VERSION = "as3";
// var TITLE   = "The int Constructor Called as a Function";


var testcases = new Array();

testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq( "int()",                  0,          int() );
    array[item++] = Assert.expectEq( "int(void 0)",            0,  int(void 0) );
    array[item++] = Assert.expectEq( "int(null)",              0,          int(null) );
    array[item++] = Assert.expectEq( "int()",                  0,          int() );
    array[item++] = Assert.expectEq( "int(new int())",      0,          int( new int() ) );
    array[item++] = Assert.expectEq( "int(0)",                 0,          int(0) );
    array[item++] = Assert.expectEq( "int(1)",                 1,          int(1) );
    array[item++] = Assert.expectEq( "int(-1)",                -1,         int(-1) );
    array[item++] = Assert.expectEq( "int('string')",          0, int( "string") );
    array[item++] = Assert.expectEq( "int(new String())",      0,          int( new String() ) );
    array[item++] = Assert.expectEq( "int('')",                0,          int( "" ) );

    array[item++] = Assert.expectEq( "int(new MyObject(100))",  100,        int(new MyObject(100)) );

    return ( array );
}
/*function test() {
    for ( tc=0; tc < testcases.length; tc++ ) {
        testcases[tc].passed = writeTestCaseResult(
                            testcases[tc].expect,
                            testcases[tc].actual,
                            testcases[tc].description +" = "+
                            testcases[tc].actual );

        testcases[tc].reason += ( testcases[tc].passed ) ? "" : "wrong value ";
    }
    stopTest();
    return ( testcases );
}*/
function MyObject( value ) {
    this.value = value;
    this.valueOf = function() { return this.value; }
}
