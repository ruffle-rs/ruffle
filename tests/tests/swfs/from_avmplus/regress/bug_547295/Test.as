/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=547295
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Assert;
// var SECTION = "547295";
// var VERSION = "";
// var TITLE   = "XMLParser::unescape does not handle lowercase hex values";
// var bug = "547295";

var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:int = 0;
    var status:String = '';
    var actual:String = '';
    var expect:String= '';

    status = 'lowercase';
    var lowercase:XML = new XML ("<foo>bar &#x6a;</foo>");
    expect = "bar j";
    actual = lowercase;
    array[item++] = Assert.expectEq( status, expect, actual);


    status = 'UPPERCASE';
    var uppercase:XML = new XML ("<foo>bar &#x6A;</foo>");
    expect = "bar j";
    actual = uppercase;
    array[item++] = Assert.expectEq( status, expect, actual);



    return array;
}
