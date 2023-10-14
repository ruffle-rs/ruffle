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


//     var SECTION = "10.1.3";
//     var VERSION = "";
//     var TITLE   = "Variable Instantiation";
//     var bug     = "17290";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    function f()
    {
        var x;

        return typeof x;

        function x()
        {
            return 7;
        }
    }

    array[item++] = Assert.expectEq( "Declaration precedence test", "function", f());

    return ( array );
}
