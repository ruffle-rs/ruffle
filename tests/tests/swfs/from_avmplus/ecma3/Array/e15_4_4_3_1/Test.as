/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 12 Mar 2001
 *
 *
 * SUMMARY: Testing Array.prototype.toLocaleString()
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=56883
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=58031
 *
 * By ECMA3 15.4.4.3, myArray.toLocaleString() means that toLocaleString()
 * should be applied to each element of the array, and the results should be
 * concatenated with an implementation-specific delimiter. For example:
 *
 *  myArray[0].toLocaleString()  +  ','  +  myArray[1].toLocaleString()  +  etc.
 *
 * In this testcase toLocaleString is a user-defined property of each array element;
 * therefore it is the function that should be invoked. This function increments a
 * global variable. Therefore the end value of this variable should be myArray.length.
 */
//-------------------------------------------------------------------------------------------------
// var SECTION = "15.4.4.3.1";
// var VERSION = "ECMA";
 var TITLE = 'Testing Array.prototype.toLocaleString()';
// var bug = 56883;

var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var actual = '';
    var expect = '';
    var n = 0;
    var obj = {toLocaleString: function() {n++}};
    var myArray = [obj, obj, obj];

    myArray.toLocaleString();
    actual = n;
    expect = 3; // (see explanation above)


   // TODO: REVIEW AS4 CONVERSION ISSUE 
   // Uncommented TITLE above
 

    array[item++] = Assert.expectEq( TITLE, 3, n);

    return ( array );
}
