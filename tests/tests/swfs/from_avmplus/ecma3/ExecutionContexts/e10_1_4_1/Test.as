/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "10.1.4.1";
//     var VERSION = "";
//     var TITLE   = "Entering An Execution Context";
//     var bug     = "none";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var y;
    var x = 1;
    z = 11;  // dynamic var z

    if (delete y)
        array[item++] = Assert.expectEq( "Expected *NOT* to be able to delete y", "fail", "fail");

    if (!delete z)
        array[item++] = Assert.expectEq( "Expected to be able to delete z", "fail", "fail");

    if (typeof x == "undefined")
        array[item++] = Assert.expectEq( "x did not remain defined after eval()", "fail", "fail");
    else if (x != 1)
        array[item++] = Assert.expectEq( "x did not retain it's value after eval()", "fail", "fail");


    if (delete x)
        array[item++] = Assert.expectEq( "Expected to be able to delete x", "fail", "fail");

    array[item++] = Assert.expectEq( "All tests passed", true, true);

    return ( array );
}
