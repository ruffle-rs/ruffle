/* -*- mode: java; tab-width: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//-----------------------------------------------------------------------------

// var SECTION = "pcre_could_be_empty_branch";
// var VERSION = "";
// var TITLE   = "Shouldn't crash on regexps with many nested parentheses embedded in ()* constructs";

var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;

    // setting this to < 407 results in evaluating the 'search' to 0, from 407, it is -1
    var NUM_PAREN = 406;

    var pattern = '';
    for (var i = 0; i < NUM_PAREN; i++) pattern += '(';
    for (var i = 0; i < NUM_PAREN; i++) pattern += 'a)';
    pattern += '*';

    try {
    var re = new RegExp(pattern);

    var res = "aaaaa".search(re);
    array[item++] = Assert.expectEq( "'aaaaa'.search(re)", 0, res);
    }
    catch (e: Error) {
    if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "'aaaaa'.search(re)", 0, 0);
    else
        throw(e);
    }
    return array;
}
