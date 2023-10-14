/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.7.3.4-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.NaN";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    delete( Number.NaN );
    array[item++] = Assert.expectEq( "delete( Number.NaN ); Number.NaN",       NaN,        Number.NaN );
    array[item++] = Assert.expectEq(  "delete( Number.NaN )",                  false,      delete( Number.NaN ) );

    return ( array );
}
