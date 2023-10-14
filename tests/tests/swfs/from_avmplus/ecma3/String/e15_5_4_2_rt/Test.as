/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.tostring";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

   /* array[item++] = Assert.expectEq(  "String.prototype.toString.__proto__",  Function.prototype, String.prototype.toString.__proto__ );*/
    array[item++] = Assert.expectEq(   
                                    "String.prototype.toString() == String.prototype.valueOf()",
                                    true,
                                    String.prototype.toString() == String.prototype.valueOf() );

    array[item++] = Assert.expectEq(    "String.prototype.toString()",     "",     String.prototype.toString() );
    array[item++] = Assert.expectEq(    "String.prototype.toString.length",    0,  String.prototype.toString.length );


    array[item++] = Assert.expectEq(   
                                    "TESTSTRING = new String(),TESTSTRING.valueOf() == TESTSTRING.toString()",
                                    true,
                                    (TESTSTRING = new String(),TESTSTRING.valueOf() == TESTSTRING.toString()) );
    array[item++] = Assert.expectEq(   
                                    "TESTSTRING = new String(true),TESTSTRING.valueOf() == TESTSTRING.toString()",
                                    true,
                                    (TESTSTRING = new String(true),TESTSTRING.valueOf() == TESTSTRING.toString()) );
    array[item++] = Assert.expectEq(   
                                    "TESTSTRING = new String(false),TESTSTRING.valueOf() == TESTSTRING.toString()",
                                    true,
                                    (TESTSTRING = new String(false),TESTSTRING.valueOf() == TESTSTRING.toString()) );
    array[item++] = Assert.expectEq(   
                                    "TESTSTRING = new String(Math.PI),TESTSTRING.valueOf() == TESTSTRING.toString()",
                                    true,
                                    (TESTSTRING = new String(Math.PI),TESTSTRING.valueOf() == TESTSTRING.toString()) );
    array[item++] = Assert.expectEq(   
                                    "TESTSTRING = new String(),TESTSTRING.valueOf() == TESTSTRING.toString()",
                                    true,
                                    (TESTSTRING = new String(),TESTSTRING.valueOf() == TESTSTRING.toString()) );

    return ( array );
}
