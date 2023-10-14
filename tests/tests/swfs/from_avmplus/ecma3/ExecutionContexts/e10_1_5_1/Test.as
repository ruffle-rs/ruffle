/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "10.5.1-1";
//     var VERSION = "ECMA_1";


    var testcases = getTestCases();


/*function test() {
    for ( tc=0; tc < testcases.length; tc++ ) {
        testcases[tc].passed = writeTestCaseResult(
                            testcases[tc].expect,
                            testcases[tc].actual,
                            testcases[tc].description +" = "+
                            testcases[tc].actual );

    }
    stopTest();
    return ( testcases );
}*/
function getTestCases() {
    var array = new Array();
    var item = 0;
    myObject = new Object();
    myObject.reason = "";

   // myObject = Assert.expectEq( "Global Code check", "", "" );

    if ( Object == null ) {
        myObject.reason += " Object == null" ;
    }
    if ( Function == null ) {
        myObject.reason += " Function == null";
    }
    if ( String == null ) {
        myObject.reason += " String == null";
    }
    if ( Array == null ) {
        myObject.reason += " Array == null";
    }
    if ( Number == null ) {
        myObject.reason += " Function == null";
    }
    if ( Math == null ) {
        myObject.reason += " Math == null";
    }
    if ( Boolean == null ) {
        myObject.reason += " Boolean == null";
    }
    if ( Date  == null ) {
        myObject.reason += " Date == null";
    }

    if ( NaN == null ) {
        myObject.reason += " NaN == null";
    }
    if ( Infinity == null ) {
        myObject.reason += " Infinity == null";
    }

/*
    if ( eval == null ) {
        myObject.reason += " eval == null";
    }
*/

    if ( myObject.reason != "" ) {
        myObject.actual = "fail";
    } else {
        myObject.actual = "pass";
    }
    myObject.expect = "pass";


    array[0] =  Assert.expectEq( "10.1.5.1 Global Code check", myObject.expect, myObject.actual);

    return ( array );
}
