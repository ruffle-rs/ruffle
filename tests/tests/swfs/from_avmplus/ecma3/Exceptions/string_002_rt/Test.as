/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 
    // TODO: REVIEW AS4 CONVERSION ISSUE 
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
    var SECTION = "string-002";
//     var VERSION = "JS1_4";
//     var TITLE   = "String.prototype.valueOf";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";

    try {
        var OBJECT =new Object();
        OBJECT.valueOf = String.prototype.valueOf;
        result = OBJECT.valueOf();
    } catch ( e ) {
        result = expect;
        exception = e.toString();
    }

    array[item++] = Assert.expectEq(
 
        "OBJECT = new Object; OBJECT.valueOf = String.prototype.valueOf;"+
        "result = OBJECT.valueOf();" +
        " (threw " + exception +")",
        expect,
        result );

    return array;
}

