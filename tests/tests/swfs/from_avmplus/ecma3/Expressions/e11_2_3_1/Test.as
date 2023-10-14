/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "e11_2_3_1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Function Calls";

    var testcases = getTestCases();

    
function getTestCases() {
    var array = new Array();
    var item = 0;

    // MemberExpression:  Identifier

    var OBJECT = true;

    array[item++] = Assert.expectEq( 
                                    "OBJECT.toString()",
                                    "true",
                                    OBJECT.toString() );

    // MemberExpression[ Expression]

    array[item++] = Assert.expectEq( 
                                    "(new Array())['length'].valueOf()",
                                    0,
                                    (new Array())["length"].valueOf() );

    // MemberExpression . Identifier
    array[item++] = Assert.expectEq( 
                                    "(new Array()).length.valueOf()",
                                    0,
                                    (new Array()).length.valueOf() );
    // new MemberExpression Arguments

    array[item++] = Assert.expectEq( 
                                    "(new Array(20))['length'].valueOf()",
                                    20,
                                    (new Array(20))["length"].valueOf() );
    return array;
}
