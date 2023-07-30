/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.5.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Parameters to string methods should be declared Number not int";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
   
    var a = "abcdefg";

    array[item++] = Assert.expectEq(   
                                    "charAt(Infinity)",
                                    "",
                                    a.charAt(Infinity) );
    array[item++] = Assert.expectEq(   
                                    "a.charAt(4294967296)",
                                    "",
                                    a.charAt(4294967296) );
    array[item++] = Assert.expectEq(   
                                    "a.charAt(4294967296+1) ",
                                    "",
                                    a.charAt(4294967296+1)  );
    array[item++] = Assert.expectEq(   
                                    "a.indexOf('2',4294967296)",
                                    -1,
                                    a.indexOf('2',4294967296) );
    array[item++] = Assert.expectEq(   
                                    "a.charCodeAt(4294967296)",
                                    NaN,
                                    a.charCodeAt(4294967296));
    array[item++] = Assert.expectEq(   
                                    "a.substring(4294967296,4294967296+2)",
                                    "",
                                    a.substring(4294967296,4294967296+2));
    array[item++] = Assert.expectEq(   
                                    "a.substring(NaN,Infinity)",
                                    "abcdefg",
                                    a.substring(NaN,Infinity));

   
    return array;

}
