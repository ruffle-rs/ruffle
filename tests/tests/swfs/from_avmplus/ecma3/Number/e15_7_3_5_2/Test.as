/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.7.3.5-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.NEGATIVE_INFINITY";


    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   
                                    "delete( Number.NEGATIVE_INFINITY )",
                                    false,
                                    delete( Number.NEGATIVE_INFINITY ) );

    delete( Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(   
                                    "delete( Number.NEGATIVE_INFINITY ); Number.NEGATIVE_INFINITY",
                                    -Infinity,
                                    Number.NEGATIVE_INFINITY );
    return ( array );
}
