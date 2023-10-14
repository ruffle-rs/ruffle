/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 12 October 2001
 *
 * SUMMARY: Regression test for string.replace bug 104375
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=104375
 */
//-----------------------------------------------------------------------------
//     var SECTION = "eregress_104375";
//     var VERSION = "";
//     var bug = 104375;

//     var TITLE   = "Testing string.replace() with backreferences";


    var testcases = getTestCases();

function inSection(x) {
   return "Section "+x+" of test -";
}

function getTestCases() {
    var array = new Array();
    var item = 0;


    var UBound = 0;
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];


    /*
     * Use the regexp to replace 'uid=31' with 'uid=15'
     *
     * In the second parameter of string.replace() method,
     * "$1" refers to the first backreference: 'uid='
     */
    var str = 'uid=31';
    var re = /(uid=)(\d+)/;
    
    // try the numeric literal 15
    status = inSection(1);
    actual  = str.replace (re, "$1" + 15);
    expect = 'uid=15';
    array[item++] = Assert.expectEq( status, expect, actual);
    
    // try the string literal '15'
    status = inSection(2);
    actual  = str.replace (re, "$1" + '15');
    expect = 'uid=15';
    array[item++] = Assert.expectEq( status, expect, actual);
    
    // try a letter before the '15'
    status = inSection(3);
    actual  = str.replace (re, "$1" + 'A15');
    expect = 'uid=A15';
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}
