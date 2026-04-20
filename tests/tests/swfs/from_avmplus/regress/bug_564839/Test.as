/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=557275
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Assert;
// var SECTION = "564839";
// var VERSION = "";
// var TITLE   = "Strings that end in charCode(0) convert to Number differently from 10.0 ";
// var bug = "564839";

var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:int = 0;
    var status:String = '';
    var actual:String = '';
    var expect:String= '';

    var str = "";

    str = "4"; str += String.fromCharCode(52);
    status = "new Number " + escape(str);
    expect = '44';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0);
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0); str += "4";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4.2"; str += String.fromCharCode(0);
    status = "new Number " + escape(str);
    expect = '4.2';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0); str += ".2";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4."; str += String.fromCharCode(0); str += "2";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4.2"; str += String.fromCharCode(0);
    status = "new Number " + escape(str);
    expect = '4.2';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4e5"; str += String.fromCharCode(0);
    status = "new Number " + escape(str);
    expect = '400000';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4e"; str += String.fromCharCode(0); str += "5";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0); str += "e5";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4.2e5"; str += String.fromCharCode(0);
    status = "new Number " + escape(str);
    expect = '420000';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4.2e"; str += String.fromCharCode(0); str += "5";
    status = "new Number " + escape(str);
    expect = '4.2';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4.2"; str += String.fromCharCode(0); str += "e5";
    status = "new Number " + escape(str);
    expect = '4.2';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4."; str += String.fromCharCode(0); str += "2e5";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0); str += ".2e5";
    status = "new Number " + escape(str);
    expect = '4';
    actual = new Number(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "44"; str += String.fromCharCode(0);
    status = "parseInt " + escape(str);
    expect = '44';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "4"; str += String.fromCharCode(0); str += "4";
    status = "parseInt " + escape(str);
    expect = '4';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "0x44"; str += String.fromCharCode(0);
    status = "parseInt " + escape(str);
    expect = '68';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "0x4"; str += String.fromCharCode(0); str += "4";
    status = "parseInt " + escape(str);
    expect = '4';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "0x"; str += String.fromCharCode(0); str += "44";
    status = "parseInt " + escape(str);
    expect = 'NaN';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    str = "0"; str += String.fromCharCode(0); str += "x44";
    status = "parseInt " + escape(str);
    expect = '0';
    actual = parseInt(str);
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}
