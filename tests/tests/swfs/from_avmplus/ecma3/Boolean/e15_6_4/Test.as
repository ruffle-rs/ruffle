/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.6.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the Boolean Prototype Object";


    var testcases = new Array();

    //@ TO - DO
    //variable tc need to be defined
    var tc;


    // false because prototype is now of type object
    testcases[tc++] = Assert.expectEq( 
                                    "Boolean.prototype=([object Object])",
                                    "false",
                                    Boolean.prototype+"");

    //save original toString
    var origToString:Function = Boolean.prototype.toString;

    Boolean.prototype.toString = Object.prototype.toString;

    testcases[tc++] = Assert.expectEq( 
                                    "Boolean.prototype.toString = Object.prototype.toString; Boolean.prototype.toString()",
                                    "[object Object]", // because all prototypes are "Object" in es4
                                   Boolean.prototype.toString());


    //restore original toString
    Boolean.prototype.toString = origToString;
