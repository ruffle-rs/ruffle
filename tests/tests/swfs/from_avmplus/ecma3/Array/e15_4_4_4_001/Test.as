/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
*
* Date:    19 September 2002
* SUMMARY: Testing Array.prototype.concat()
* See http://bugzilla.mozilla.org/show_bug.cgi?id=169795
*
*/
//-----------------------------------------------------------------------------
//     var SECTION = "15.4.4.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Testing Array.prototype.concat()";
//     var bug     = "169795";

package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

   // TODO: REVIEW AS4 CONVERSION ISSUE 
   // commented out all inSection(x) calls

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var status = '';
    var actual = '';
    var expect= '';
    var x;

    //status = inSection(1);
    x = "Hello";
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "concat(x).toString()", expect, actual );

    //status = inSection(2);
    x = 999;
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat(999).toString()", expect, actual );

    // status = inSection(3);
    x = /Hello/g;
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat(/Hello/g).toString()", expect, actual );

    // status = inSection(4);
    x = new Error("Hello");
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat(new Error('hello')).toString()", expect, actual );


    /*
    status = inSection(5);
    x = function() {return "Hello";};
    actual = [].concat(x).toString();
    expect = x.toString();
    addThis();
    */

    // status = inSection(6);
    x = [function() {return "Hello";}];
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat([function() {return 'Hello';}).toString()", expect, actual );

    // status = inSection(7);
    x = [1,2,3].concat([4,5,6]);
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat([1,2,3].concat([4,5,6]).toString()", expect, actual );

    // status = inSection(8);
    x = this;
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat(this).toString()", expect, actual );

    /*
     * The next two sections are by igor@icesoft.no; see
     * http://bugzilla.mozilla.org/show_bug.cgi?id=169795#c3
     */
    // status = inSection(9);
    x={length:0};
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat({length:0}).toString()", expect, actual );

    // status = inSection(10);
    x={length:2, 0:0, 1:1};
    actual = [].concat(x).toString();
    expect = x.toString();
    array[item++] = Assert.expectEq( "[].concat({length:2, 0:0, 1:1}).toString()", expect, actual );

    return ( array );
}
