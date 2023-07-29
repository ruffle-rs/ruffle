/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.8-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.split";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var TEST_STRING = "";
    var EXPECT = new Array();

    // this.toString is the empty string.

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); s.split().length",
                                    1,
                                    (s = new String(), s.split().length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); s.split()[0]",
                                    "",
                                    (s = new String(), s.split()[0] ) );

    // this.toString() is the empty string, separator is specified.

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); s.split('').length",
                                    1,
                                    (s = new String(), s.split('').length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); s.split(' ').length",
                                    1,
                                    (s = new String(), s.split(' ').length ) );

    // this to string is " "
    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split().length",
                                    1,
                                    (s = new String(' '), s.split().length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split()[0]",
                                    " ",
                                    (s = new String(' '), s.split()[0] ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split('').length",
                                    1,
                                    (s = new String(' '), s.split('').length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split('')[0]",
                                    " ",
                                    (s = new String(' '), s.split('')[0] ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split(' ').length",
                                    2,
                                    (s = new String(' '), s.split(' ').length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String(' '); s.split(' ')[0]",
                                    "",
                                    (s = new String(' '), s.split(' ')[0] ) );

    array[item++] = Assert.expectEq(   
                                    "\"\".split(\"\").length",
                                    1,
                                    ("".split("")).length );

    array[item++] = Assert.expectEq(   
                                    "\"\".split(\"x\").length",
                                    1,
                                    ("".split("x")).length );

    array[item++] = Assert.expectEq(   
                                    "\"\".split(\"x\")[0]",
                                    "",
                                    ("".split("x"))[0] );

    return array;
}

function Split( string, separator ) {
    string = String( string );

    var A = new Array();

    if ( arguments.length < 2 ) {
        A[0] = string;
        return A;
    }

    separator = String( separator );

    var str_len = String( string ).length;
    var sep_len = String( separator ).length;

    var p = 0;
    var k = 0;

    if ( sep_len == 0 ) {
        for ( ; p < str_len; p++ ) {
            A[A.length] = String( string.charAt(p) );
        }
    }
    return A;
}
