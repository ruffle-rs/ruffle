/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "exception-003";
//     var VERSION = "js1_4";
//     var TITLE   = "Tests for Actionscript Standard Exceptions: TargetError";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var expectedError = 1056;
    if (true) {       // TODO: REVIEW AS4 CONVERSION ISSUE   
        expectedError = 1037;
    }

    Target_1();

    function Target_1() {
        result = "failed: no exception thrown";
        exception = null;

        try {

            string ="hi";
            string.toString = Boolean.prototype.toString;
            string.toString();

        } catch ( e ) {
            //result = "passed:  threw exception",
            result = e.toString();
        } finally {
            array[item++] = Assert.expectEq(
              //    SECTION,
                "string = new String(\"hi\");"+
                "string.toString = Boolean.prototype.toString" +
                "string.toString()",
                Utils.REFERENCEERROR+expectedError,
                Utils.referenceError( result ) );
        }
    }
    return array;
}
