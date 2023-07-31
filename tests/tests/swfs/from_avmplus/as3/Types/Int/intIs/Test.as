/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "Types: int";
//     var VERSION = "as3";
//     var TITLE   = "x is int";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var c = 0;
    array[item++] = Assert.expectEq( "var c=0;c is int",true, c is int );
    array[item++] = Assert.expectEq( "0 is int",true, 0 is int );

    var d = 1;
    array[item++] = Assert.expectEq( "var d=1;d is int",true, d is int );
    array[item++] = Assert.expectEq( "1 is int",true, 1 is int );

    var e = uint.MAX_VALUE;
    array[item++] = Assert.expectEq( "var e=uint.MAX_VALUE;e is int",false, e is int );
    array[item++] = Assert.expectEq( "uint.MAX_VALUE is int",false, uint.MAX_VALUE is int );
    array[item++] = Assert.expectEq( "4294967295 is int",false, 4294967295 is int );

    var f:int = -1;
    array[item++] = Assert.expectEq( "var f=-1;f is int",true, f is int );
    array[item++] = Assert.expectEq( "-1 is int",true, -1 is int );

    var g:int = int.MAX_VALUE;
    array[item++] = Assert.expectEq( "var g=int.MAX_VALUE;g is int",true, g is int );
    array[item++] = Assert.expectEq( "int.MAX_VALUE is int",true, int.MAX_VALUE is int );
    array[item++] = Assert.expectEq( "2147483647 is int",true, 2147483647 is int );
    
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
