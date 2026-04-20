/* -*- mode: java; tab-width: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//-----------------------------------------------------------------------------

// var SECTION = "pcre_find_fixedlength";
// var VERSION = "";
// var TITLE   = "Shouldn't crash on regexps with many nested parentheses embedded in look-behind assertions";

var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;

    var NO_BACKREFS = false;
    var DO_BACKREFS = true;

    //--------------------------------------------------

    testThis(500, NO_BACKREFS, 'blah', 'hello', 'goodbye');
    testThis(500, DO_BACKREFS, 'blah', 'hello', 'goodbye');

    // constructs not allowed
    testThis2("(?<=a{3,})bla", "bbbla", -1);
    testThis2("(?<=a?)bla", "bla", -1);
    testThis2("(?<=a?)bla", "abla", -1);
    testThis2("(?<=a+)bla", "abla", -1);
    testThis2("(?<=a*)bla", "abla", -1);

    // why is this working?
    testThis2("(?<=a{3}|b{2})bla", "aaabla", 3);
    testThis2("(?<=a{3}|b{2})bla", "bbbla", 2);

    // there one has lookbehind assertion of non-fixed length
    testThis2("(?<=(a{3}|b{2}))bla", "bbbla", -1);
    testThis2("(?<=(((a{3}|b{2}))))bla", "bbbla", -1);
    testThis2("(?<=(((((a{3}|b{2}))))))bla", "bbbla", -1);

    // fixed length one
    testThis2("(?<=(a{3}|b{2}b))bla", "bbbbla", 3);

    // have pattern where a capturing group is followed by a character
    // this is a test for having a work item propagate back its fixed length to the parent
    // and the parent's branch is not finished yet
    testThis2("(?<=(x(a|b)c|bbb))bla", "xacbla", 3);
    testThis2("(?<=(x(a|b)c|bbb))bla", "bbbbla", 3);
    testThis2("(?<=a(x(a|b)c|b(b|x)b)a)bla", "abxbabla", 5);

    //--------------------------------------------------

    /*
     * Creates a regexp pattern like (?<=(((((((((blah))))))))))hello
     * and tests str.search(), str.match(), str.replace()
     *
     * function used to create regexps that usage the stack exhaustively
     * */
    function testThis(numParens, doBackRefs, strLookbehind, strOriginal, strReplace)
    {
    var openParen = doBackRefs? '(' : '(?:';
    var closeParen = ')';
    var pattern = '';

    pattern += '(?<=';

    for (var i=0; i<numParens; i++) {pattern += openParen;}
    pattern += strLookbehind;
    for (i=0; i<numParens; i++) {pattern += closeParen;}
    pattern += closeParen;
    pattern += strOriginal;

    // We don't know who is going to compile the RE, so just wrap the whole thing.
    // We're not testing search, match, or replace, just RE compilation, so don't
    // worry about handling individual cases.
    try {
        var re = new RegExp(pattern);

        strOriginal = strLookbehind + strOriginal;

        if (doBackRefs) {
        var res = strOriginal.search(re);
        array[item++] = Assert.expectEq( "strOriginal.search(re)", -1, res);

        res = strOriginal.match(re);
        array[item++] = Assert.expectEq( "strOriginal.match(re)", null, res);

        res = strOriginal.replace(re, strReplace);
        array[item++] = Assert.expectEq( "strOriginal.replace(re, strReplace)", "blahhello", res);
        } else {
        var res = strOriginal.search(re);
        array[item++] = Assert.expectEq( "strOriginal.search(re)", 4, res);

        res = strOriginal.match(re);
        //Get the first element to compare
        res = res[0];
        array[item++] = Assert.expectEq( "strOriginal.match(re)", 'hello', res);

        res = strOriginal.replace(re, strReplace);
        array[item++] = Assert.expectEq( "strOriginal.replace(re, strReplace)", "blahgoodbye", res);
        }
    }
    catch (e: Error) {
        if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "strOriginal.search/match/replace(re)", 0, 0);
        else
        throw(e);
    }
    }

    /*
     * Creates a regex from the pattern and applies search to the strSubject with the regexp created
     *
     * expected is number that is the result of the search function
     * */
    function testThis2(strPattern, strSubject, expected)
    {
    var regex = new RegExp(strPattern);
    var result = strSubject.search(regex);

    array[item++] = Assert.expectEq( "\"" + strSubject + "\".search(new RegExp(" + strPattern + "))", expected, result);
    }

    return array;
}
