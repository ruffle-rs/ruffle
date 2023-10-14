/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "exception-002";
//     var VERSION = "js1_4";
//     var TITLE   = "Tests for JavaScript Standard Exceptions: ConstructError";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    Construct_1();

    function Construct_1() {
        result = "failed: no exception thrown";
        exception = null;

        try {
            result = new Math();
        } catch ( e ) {
            result = e.toString();
        } finally {
            array[item++] = Assert.expectEq(
              //    SECTION,
                "new Math()",
                Utils.TYPEERROR+1076,
                Utils.typeError( result ) );
        }
    }
    return array;
}
