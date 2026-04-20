/* -*- mode: java; tab-width: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//-----------------------------------------------------------------------------

// var SECTION = "pcre_is_anchored";
// var VERSION = "";
// var TITLE   = "Shouldn't crash on regexps with many nested parentheses";

var testcases = getTestCases();

// The tests succeed either if they finish normally or if they exit by stack
// overflow (on limited stack systems).
function getTestCases()
{
    var array = new Array();
    var item = 0;

    var NO_BACKREFS = false;
    var DO_BACKREFS = true;

    //--------------------------------------------------

    testThis(500, NO_BACKREFS, 'a');
    testThis(500, DO_BACKREFS, 'a');

    //--------------------------------------------------

    /*
     * Creates a regexp pattern like ((...((a))...))
     * and tests str.search(), str.match(), str.replace()
     * */
    function testThis(numParens, doBackRefs, str)
    {
    var openParen = doBackRefs? '(' : '(?:';
    var closeParen = ')';
    var pattern = '';

    for (var i=0; i<numParens; i++) {pattern += openParen;}
    pattern += str;
    for (i=0; i<numParens; i++) {pattern += closeParen;}

    try {
        var re = new RegExp(pattern);

        if (doBackRefs) {
        var res = str.search(re);
        array[item++] = Assert.expectEq( "str.search(re)", -1, res);
        } else {
        var res = str.search(re);
        array[item++] = Assert.expectEq( "str.search(re)", 0, res);
        }
    }
    catch (e: Error) {
        if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "str.search(re)", 0, 0);
        else
        throw(e);
    }
    }
    return array;
}
