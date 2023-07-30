/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// TODO: REVIEW AS4 CONVERSION ISSUE 
//     var SECTION = "number-001";
//     var VERSION = "JS1_4";
//     var TITLE   = "Exceptions for Number.toString()";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    var result = "Failed";
    var exception = "No exception";
    var expect = "Passed";


    try {
        object= new Object();
        
        object.toString = Number.prototype.toString;
        result = object.toString();
        expect = result;
    } catch ( e:Error ) {
        result = expect;
        exception = (e.toString()).substring(0,18);
    }

    array[item++] = Assert.expectEq(
        // SECTION,
        "object = new Object(); object.toString = Number.prototype.toString; object.toString()" +
        " (threw " + exception +")",
        expect,
        result );

    return array;
}
