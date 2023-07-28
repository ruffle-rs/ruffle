/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.2.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Array Constructor:  new Array()";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var arr;
    array[item++] = Assert.expectEq(   "new   Array() +''",        "",                 (new Array()) +"" );
    array[item++] = Assert.expectEq(   "typeof new Array()",       "object",           (typeof new Array()) );
    array[item++] = Assert.expectEq(   
                                    "var arr = new Array(); arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (arr = new Array(), arr.getClass = Object.prototype.toString, arr.getClass() ) );

    array[item++] = Assert.expectEq(   "(new Array()).length",     0,                  (new Array()).length );
    array[item++] = Assert.expectEq(   "(new Array()).toString == Array.prototype.toString",   true,       (new Array()).toString == Array.prototype.toString );

    // TODO: REVIEW AS4 CONVERSION ISSUE 
    // COMMENT OUT WHOLE BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    /* if (!as3Enabled) {
        array[item++] = Assert.expectEq(   "(new Array()).join  == Array.prototype.join",          true,       (new Array()).join  == Array.prototype.join );
        array[item++] = Assert.expectEq(   "(new Array()).reverse == Array.prototype.reverse",     true,       (new Array()).reverse  == Array.prototype.reverse );
        array[item++] = Assert.expectEq(   "(new Array()).sort  == Array.prototype.sort",          true,       (new Array()).sort  == Array.prototype.sort );
     }
    */
    return ( array );
}
