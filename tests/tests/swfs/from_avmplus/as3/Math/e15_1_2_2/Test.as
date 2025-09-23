/* -*- Mode: js; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "15.1.2.2 parseInt()";
// var VERSION = "";
// var TITLE = "Tests based on code coverage";


var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;


    // other js engines produce 1228503247364293600 for this testcase, and c produces 1228503247364293699
    // Bug 602912: https://bugzilla.mozilla.org/show_bug.cgi?id=602912
    array[item++] = Assert.expectEq(   "radix 32 > 53bits of precisions", 1228503247364293699, parseInt("1234567890123", 32) );
    array[item++] = Assert.expectEq(   "radix 8 > 53bits of precisions", 23528930381028800, parseInt("1234567000123456700", 8) ); // other engines 188231443048230400
    array[item++] = Assert.expectEq(   "radix 4 > 53bits of precisions", 5.120083167990638e+23, parseInt("1230123000123012300012301230001230123000", 4) );
    array[item++] = Assert.expectEq(   "radix 2 > 53bits of precisions", 1.1805916207174113e+21, parseInt("1111111111111111111111111111111111111111111111111111111111111111111111", 2) );


    return ( array );
}
