/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
*
* Date:    25 Mar 2002
* SUMMARY: Array.prototype.sort() should not (re-)define .length
* See http://bugzilla.mozilla.org/show_bug.cgi?id=130451
*
* From the ECMA-262 Edition 3 Final spec:
*
* NOTE: The sort function is intentionally generic; it does not require that
* its |this| value be an Array object. Therefore, it can be transferred to
* other kinds of objects for use as a method. Whether the sort function can
* be applied successfully to a host object is implementation-dependent.
*
* The interesting parts of this testcase are the contrasting expectations for
* Brendan's test below, when applied to Array objects vs. non-Array objects.
*
*    Modified:      28th October 2004 (gasingh@macromedia.com)
*               Removed the occurence of new Function('abc').
*               This is being changed to function() { abc }.
*
*
*/
//-----------------------------------------------------------------------------
//     var SECTION = "eregress_130451";
//     var VERSION = "ECMA";
//     var TITLE   = "Array.prototype.sort() should not (re-)define .length";
//     


   // TODO: REVIEW AS4 CONVERSION ISSUE 
   // Commented out calls to insection

package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
var bug     = "130451";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var status = '';
    var actual = '';
    var expect= '';
    var arr = [];
    var cmp = function() {};

    /*
     * First: test Array.prototype.sort() on Array objects
     */
    // status = inSection(1);
    arr = [0,1,2,3];
    cmp = function(x,y) {return x-y;};
    actual = arr.sort(cmp).length;
    expect = 4;
    array[item++] = Assert.expectEq( "section 1", expect, actual );

    // status = inSection(2);
    arr = [0,1,2,3];
    cmp = function(x,y) {return y-x;};
    actual = arr.sort(cmp).length;
    expect = 4;
    array[item++] = Assert.expectEq( "section 2", expect, actual );

    // status = inSection(3);
    arr = [0,1,2,3];
    cmp = function(x,y) {return x-y;};
    arr.length = 1;
    actual = arr.sort(cmp).length;
    expect = 1;
    array[item++] = Assert.expectEq( "section 3", expect, actual );

    /*
     * This test is by Brendan. Setting arr.length to
     * 2 and then 4 should cause elements to be deleted.
     */
    arr = [0,1,2,3];
    cmp = function(x,y) {return x-y;};
    arr.sort(cmp);

    // status = inSection(4);
    actual = arr.join();
    expect = '0,1,2,3';
    array[item++] = Assert.expectEq( "section 4", expect, actual );

    // status = inSection(5);
    actual = arr.length;
    expect = 4;
    array[item++] = Assert.expectEq( "section 5", expect, actual );

    // status = inSection(6);
    arr.length = 2;
    actual = arr.join();
    expect = '0,1';
    array[item++] = Assert.expectEq( "section 6", expect, actual );

    // status = inSection(7);
    arr.length = 4;
    actual = arr.join();
    expect = '0,1,,';  //<---- see how 2,3 have been lost
    array[item++] = Assert.expectEq( "section 7", expect, actual );

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    // REMOVED OUT WHOLE as3Enabled BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    
    return ( array );
}
