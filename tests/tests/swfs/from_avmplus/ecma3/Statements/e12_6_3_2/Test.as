/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "12.6.3-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var myResult;
    var origFoo = Boolean.prototype.foo;
    Boolean.prototype.foo = 34;
    array[item++] = Assert.expectEq(   
                                    "Boolean.prototype.foo = 34; for ( j in Boolean ) Boolean[j]",
                                    34,
                                    Boolean.prototype.foo );
    //restore
    Boolean.prototype.foo = origFoo;
    return ( array );
}
