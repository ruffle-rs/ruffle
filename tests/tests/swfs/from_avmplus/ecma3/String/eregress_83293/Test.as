/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Creation Date:   30 May 2001
 * Correction Date: 14 Aug 2001
 *
 * SUMMARY:  Regression test for bugs 83293, 103351
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=83293
 *     http://bugzilla.mozilla.org/show_bug.cgi?id=103351
 *     http://bugzilla.mozilla.org/show_bug.cgi?id=92942
 *
 *
 * ********************   CORRECTION !!!  *****************************
 *
 * When I originally wrote this test, I thought this was true:
 * str.replace(strA, strB) == str.replace(new RegExp(strA),strB).
 * See ECMA-262 Final Draft, 15.5.4.11 String.prototype.replace
 *
 * However, in http://bugzilla.mozilla.org/show_bug.cgi?id=83293
 * Jim Ley points out the ECMA-262 Final Edition changed on this.
 * String.prototype.replace (searchValue, replaceValue), if provided
 * a searchValue that is not a RegExp, is NO LONGER to replace it with
 *
 *                  new RegExp(searchValue)
 * but rather:
 *                  String(searchValue)
 *
 * This puts the replace() method at variance with search() and match(),
 * which continue to follow the RegExp conversion of the Final Draft.
 * It also makes most of this testcase, as originally written, invalid.
 **********************************************************************
 */
//-----------------------------------------------------------------------------
//     var SECTION = "eregress_104375";
//     var VERSION = "";
//     var bug = 103351; // <--- (Outgrowth of original bug 83293)

    var summ_OLD = 'Testing str.replace(strA, strB) == str.replace(new RegExp(strA),strB)';
//     var TITLE   = "Testing String.prototype.replace(x,y) when x is a string";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var status = '';
    var actual = '';
    var expect= '';
    var cnEmptyString = '';
    var str = 'abc';
    var strA = cnEmptyString;
    var strB = 'Z';

//////////////////////////  OK, LET'S START OVER //////////////////////////////

    status = 'Section 1 of test';
    actual = 'abc'.replace('a', 'Z');
    expect = 'Zbc';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 2 of test';
    actual = 'abc'.replace('b', 'Z');
    expect = 'aZc';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 3 of test';
    actual = 'abc'.replace(undefined, 'Z');
    expect = 'abc'; // String(undefined) == 'undefined'; no replacement possible
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 4 of test';
    actual = 'abc'.replace(null, 'Z');
    expect = 'abc'; // String(null) == 'null'; no replacement possible
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 5 of test';
    actual = 'abc'.replace(true, 'Z');
    expect = 'abc'; // String(true) == 'true'; no replacement possible
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 6 of test';
    actual = 'abc'.replace(false, 'Z');
    expect = 'abc'; // String(false) == 'false'; no replacement possible
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 7 of test';
    actual = 'aa$aa'.replace('$', 'Z');
    expect = 'aaZaa'; // NOT 'aa$aaZ' as in ECMA Final Draft; see above
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 8 of test';
    actual = 'abc'.replace('.*', 'Z');
    expect = 'abc';  // not 'Z' as in EMCA Final Draft
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section 9 of test';
    actual = 'abc'.replace('', 'Z');
    expect = 'Zabc';  // Still expect 'Zabc' for this
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}
