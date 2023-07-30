/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "instaceof_002";
//     var VERSION = "ECMA_2";

//     var TITLE   = "The Call Constructor";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var b = new Boolean();

    array[item++] = Assert.expectEq( 
                                    "var b = new Boolean(); b instanceof Boolean",
                                    true,
                                    b instanceof Boolean );

    array[item++] = Assert.expectEq( 
                                    "b instanceof Object",
                                    true,
                                    b instanceof Object );

    array[item++] = Assert.expectEq( 
                                    "b instanceof Array",
                                    false,
                                    b instanceof Array );

    array[item++] = Assert.expectEq( 
                                    "true instanceof Boolean",
                                    true,
                                    true instanceof Boolean );

    array[item++] = Assert.expectEq( 
                                    "Boolean instanceof Object",
                                    true,
                                    Boolean instanceof Object );
    return (array);
}
