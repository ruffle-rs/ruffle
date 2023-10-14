/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "int.MIN_VALUE";
//     var VERSION = "AS3";
//     var TITLE   = "int.MIN_VALUE";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var MIN_VAL = -2147483648;

    array[item++] = Assert.expectEq(   "int.MIN_VALUE",     MIN_VAL,    int.MIN_VALUE );

    array[item++] = Assert.expectEq(   "delete( int.MIN_VALUE )",                       false,      delete( int.MIN_VALUE) );
    delete( int.MIN_VALUE );
    array[item++] = Assert.expectEq(   "delete( int.MIN_VALUE ); int.MIN_VALUE",     MIN_VAL,    int.MIN_VALUE );

    // moved to error folder, now gives compiler error in strict
    /*int.MIN_VALUE=0;
    array[item++] = Assert.expectEq(
                    //SECTION,
                    "int.MIN_VALUE=0; int.MIN_VALUE",
                    int.MIN_VALUE,
                    int.MIN_VALUE );
*/
    var string = '';
    for ( var prop in int ) {
        string += ( prop == 'MIN_VALUE' ) ? prop : '';
    }

    array[item++] = Assert.expectEq(
                    //SECTION,
                    "var string = ''; for ( prop in int ) { string += ( prop == 'MIN_VALUE' ) ? prop : '' } string;",
                    "",
                    string
                    );

    return ( array );
}
/*function test() {
    for ( tc = 0; tc < testcases.length; tc++ ) {
        testcases[tc].passed = writeTestCaseResult(
                            testcases[tc].expect,
                            testcases[tc].actual,
                            testcases[tc].description +" = "+ testcases[tc].actual );
        testcases[tc].reason += ( testcases[tc].passed ) ? "" : "delete should not be allowed "
    }
    stopTest();
    return ( testcases );
}*/
