/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=550958
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Assert;
// var SECTION = "550958";
// var VERSION = "";
// var TITLE   = "XML parse !DOCTYPE case insensitive";
// var bug = "550958";

var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:int = 0;
    var status:String = '';
    var actual:String = '';
    var expect:String= '';

    // Testing to make sure that the following exception is not thrown:
    // TypeError: Error #1090: XML parser failure: element is malformed" exception

    status = 'UPPERCASE';
    var upper:XML = new XML("<!DOCTYPE HTML>");
    expect = upper;
    actual = "";
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'lowercase';
    var lower:XML = new XML("<!doctype html>");
    expect = lower;
    actual = "";
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'mixedcase';
    var mixed:XML = new XML("<!DocType html>");
    expect = mixed;
    actual = "";
    array[item++] = Assert.expectEq( status, expect, actual);


    return array;
}
