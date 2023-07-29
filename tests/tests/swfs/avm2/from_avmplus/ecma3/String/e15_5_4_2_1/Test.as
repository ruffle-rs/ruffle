/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.2-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.toString";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq(    "String.prototype.toString()",        "",     String.prototype.toString() );
    array[item++] = Assert.expectEq(    "(new String()).toString()",          "",     (new String()).toString() );
    array[item++] = Assert.expectEq(    "(new String(\"\")).toString()",      "",     (new String("")).toString() );
    array[item++] = Assert.expectEq(    "(new String( String() )).toString()","",    (new String(String())).toString() );
    array[item++] = Assert.expectEq(   "(new String( \"h e l l o\" )).toString()",       "h e l l o",    (new String("h e l l o")).toString() );
    array[item++] = Assert.expectEq(    "(new String( 0 )).toString()",       "0",    (new String(0)).toString() );
    return ( array );
}
