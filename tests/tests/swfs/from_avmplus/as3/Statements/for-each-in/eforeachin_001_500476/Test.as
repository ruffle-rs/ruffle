/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     TODO: REVIEW AS4 CONVERSION ISSUE
//     var SECTION = "forin-001";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The for each in  statement";
    var BUGNUMBER="https://bugzilla.mozilla.org/show_bug.cgi?id=500476";


    var tc = 0;
    var testcases = new Array();

    ForIn_7({ length:4, company:"netscape", year:2000, 0:"zero" });


    function ForIn_7( object ) {
        var result1 = 0;
        var result2 = 0;
        var result3 = 0;
        var result4 = 0;
        var i = 0;
        var property = new Array();

        //bigredbird:
            for each( property[i++] in object ) {
                result2++;
                continue;
                result4++;
            }

        testcases[tc++] = Assert.expectEq(
            //SECTION,
            "verify statements in for loop are evaluated",
            true,
            result2 == i );

        testcases[tc++] = Assert.expectEq(
            //SECTION,
            "verify continue of labeled for each in loop",
            true,
            result4 == 0 );

    }
