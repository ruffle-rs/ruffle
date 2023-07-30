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
//     var SECTION = "uc_004";
//     var VERSION = "";
//     var TITLE = "Unicode Characters 1C-1F with regexps test.";

// TO-DO: commenting the function reference in shell.as  
    //printBugNumber (23612);


//     var bug = '(none)';


    var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;
    var ary = ["\u001Cfoo", "\u001Dfoo", "\u001Efoo", "\u001Ffoo"];
    
    for (var i in ary)
    {
        array[item++] = Assert.expectEq( "Unicode characters 1C-1F in regexps, ary[" +
                       i + "] did not match \\S test (it should not.)", 0, ary[Number(i)].search(/^\Sfoo$/));
        array[item++] = Assert.expectEq( "Unicode characters 1C-1F in regexps, ary[" +
                       i + "] matched \\s test (it should not.)", -1, ary[Number(i)].search(/^\sfoo$/));
    }
    return array;
}
