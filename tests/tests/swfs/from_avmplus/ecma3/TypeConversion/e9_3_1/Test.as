/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.3-1";
//     var VERSION = "ECMA_1";
    var TYPE = "number";
    var testcases = getTestCases();


function test() {
    for ( tc=0; tc < testcases.length; tc++ ) {
        testcases[tc].passed = writeTestCaseResult(
                            testcases[tc].expect,
                            testcases[tc].actual,
                            testcases[tc].description +" = "+
                            testcases[tc].actual );

        testcases[tc].passed = writeTestCaseResult(
                                TYPE,
                                typeof(testcases[tc].actual),
                                "typeof( " + testcases[tc].description +
                                " ) = " + typeof(testcases[tc].actual) )
                                ? testcases[tc].passed
                                : false;

        testcases[tc].reason += ( testcases[tc].passed ) ? "" : "wrong value ";
    }
    stopTest();
    return ( testcases );
}
function getTestCases() {
    var array = new Array();
    var item = 0;

    // object is Number
    array[item++] = Assert.expectEq(    "Number(new Number())",          0,              Number(new Number())  );
    array[item++] = Assert.expectEq(    "Number(new Number(Number.NaN))",Number.NaN,     Number(new Number(Number.NaN)) );
    array[item++] = Assert.expectEq(    "Number(new Number(0))",         0,              Number(new Number(0)) );
    array[item++] = Assert.expectEq(    "Number(new Number(null))",      0,              Number(new Number(null)) );
//    array[item++] = Assert.expectEq(    "Number(new Number(void 0))",    Number.NaN,     Number(new Number(void 0)) );
    array[item++] = Assert.expectEq(    "Number(new Number(true))",      1,              Number(new Number(true)) );
    array[item++] = Assert.expectEq(    "Number(new Number(false))",     0,              Number(new Number(false)) );

    // object is boolean

    array[item++] = Assert.expectEq(    "Number(new Boolean(true))",     1,  Number(new Boolean(true)) );
    array[item++] = Assert.expectEq(    "Number(new Boolean(false))",    0,  Number(new Boolean(false)) );

    // object is array
    array[item++] = Assert.expectEq(    "Number(new Array(2,4,8,16,32))",      Number.NaN,     Number(new Array(2,4,8,16,32)) );

    return ( array );
}
