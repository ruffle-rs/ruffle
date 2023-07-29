/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.14";
//     var VERSION = "";
//     var TITLE   = "String.prototype.split https://bugzilla.mozilla.org/show_bug.cgi?id=504567";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   
                                    "'ab'.split(/a*/)",
                                    ",b",
                                    'ab'.split(/a*/).toString() );
    
    // array[item++] = Assert.expectEq(
    //                                 "'abab'.split(/a*/)",
    //                                 ",b,b",
    //                                 'abab'.split(/a*/).toString() );
    //
    // array[item++] = Assert.expectEq(
    //                                 "'babab'.split(/a*/)",
    //                                 "b,b,b",
    //                                 'babab'.split(/a*/).toString() );
   
    return array;

}
