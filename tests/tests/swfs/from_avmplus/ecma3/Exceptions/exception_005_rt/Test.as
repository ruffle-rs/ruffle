/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "exception-005";
//     var VERSION = "js1_4";
//     var TITLE   = "Tests for JavaScript Standard Exceptions: ToObjectError";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    ToObject_1();

    function ToObject_1() {
        result = "failed: no exception thrown";
        exception = null;

        try {
           result = foo["bar"];
        } catch ( e ) {
            result = "passed:  threw exception",
            exception = e.toString();
        } finally {
            array[item++] = Assert.expectEq(
              //    SECTION,
                "foo[\"bar\"] ",
                Utils.REFERENCEERROR+1065,
                Utils.referenceError( exception ) );
        }
    }
    return array;
}
