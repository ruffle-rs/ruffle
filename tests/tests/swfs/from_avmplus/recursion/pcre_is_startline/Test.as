/* -*- mode: java; tab-width: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//-----------------------------------------------------------------------------

// var SECTION = "pcre_is_startline";
// var VERSION = "";
// var TITLE   = "Shouldn't crash on regexps with many nested parentheses";

var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;

    var NO_BACKREFS = false;
    var DO_BACKREFS = true;

    //--------------------------------------------------

    testThis(500, NO_BACKREFS, 'a', 'b');
    testThis(500, DO_BACKREFS, 'a', 'b');

    //--------------------------------------------------

    /*
     * Creates a regexp pattern like ((..((a|b)))..)) + using non-capturing brackets instead
     * of capturing ones.
     * and tests str.search()
     *
     * It is essential for this test pattern to contain a a|b alternating construct.
     * Using such construct triggers the recursive execution of pcre_compile.cpp::is_startline.
     * If | is not used, there is no recursion at all, which is interesting because this means,
     * pcre is able to find out somehow, that the regex contains a | somewhere deep.
     * A regexp like /(((((a)))))(a|b)/ cause only a 1-2 level deep recursion of is_startline.
     *
     * It may worth playing with conditional and assertion brackets too.
     * */
    function testThis(numParens, doBackRefs, strAlt1, strAlt2)
    {
    var openParen = doBackRefs? '(' : '(?:';
    var closeParen = ')';
    var pattern = '';

    for (var i=0; i<numParens; i++)
        pattern += openParen;
    pattern += strAlt1 + "|" + strAlt2;
    for (i=0; i<numParens; i++)
        pattern += closeParen;

    try {
        var re = new RegExp(pattern);

        if (doBackRefs) {
        var res = strAlt1.search(re);
        array[item++] = Assert.expectEq( "strAlt1.search(re)", -1, res);
        } else {
        var res = strAlt1.search(re);
        array[item++] = Assert.expectEq( "strAlt1.search(re)", 0, res);
        }
    }
    catch (e: Error) {
        if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "strAlt1.search(re)", 0, 0);
        else
        throw(e);
    }
    }

    return array;
}
