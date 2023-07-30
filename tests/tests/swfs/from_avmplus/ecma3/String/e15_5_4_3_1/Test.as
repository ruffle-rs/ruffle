/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.3-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.valueOf";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "String.prototype.valueOf.length", 0,      String.prototype.valueOf.length );

    array[item++] = Assert.expectEq(    "String.prototype.valueOf()",        "",     String.prototype.valueOf() );
    array[item++] = Assert.expectEq(    "(new String()).valueOf()",          "",     (new String()).valueOf() );
    array[item++] = Assert.expectEq(    "(new String(\"\")).valueOf()",      "",     (new String("")).valueOf() );
    array[item++] = Assert.expectEq(    "(new String( String() )).valueOf()","",    (new String(String())).valueOf() );
    array[item++] = Assert.expectEq(    "(new String( \"h e l l o\" )).valueOf()",       "h e l l o",    (new String("h e l l o")).valueOf() );
    array[item++] = Assert.expectEq(    "(new String( 0 )).valueOf()",       "0",    (new String(0)).valueOf() );
    return ( array );
}
