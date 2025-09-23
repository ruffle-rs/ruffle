/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=557933
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Assert;
// var SECTION = "561191";
// var VERSION = "";
// var TITLE   = "unescape bugginess";
// var bug = "561191";

var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:int = 0;

    var tests = [
        { xml: "<a foo='bar'/>",        expect: '<a foo="bar"/>' },
        { xml: "<a foo='&bar'/>",        expect: '<a foo="&amp;bar"/>' },
        { xml: "<a foo='bar&'/>",        expect: '<a foo="bar&amp;"/>' },
        { xml: "<a foo='bar&;'/>",        expect: '<a foo="bar&amp;;"/>' },
        { xml: "<a foo='b&a;r'/>",        expect: '<a foo="b&amp;a;r"/>' },
        { xml: "<a foo='&b;ar&;'/>",        expect: '<a foo="&amp;b;ar&amp;;"/>' },
        { xml: "<a foo='&b;ar&;'/>",        expect: '<a foo="&amp;b;ar&amp;;"/>' },
        { xml: "<a foo='&123b;ar&;'/>",        expect: '<a foo="&amp;123b;ar&amp;;"/>' },
        { xml: "<a foo='&bxxx;ar&;'/>",        expect: '<a foo="&amp;bxxx;ar&amp;;"/>' },
        { xml: "<a foo='&bx;ar&;'/>",        expect: '<a foo="&amp;bx;ar&amp;;"/>' },
        { xml: "<a foo='&by;ar&;'/>",        expect: '<a foo="&amp;by;ar&amp;;"/>' },
        { xml: "<a foo='&#33;'/>",        expect: '<a foo="!"/>' },
        { xml: "<a foo='&#3;'/>",        expect: '<a foo="' + String.fromCharCode(3) + '"/>' },
        { xml: "<a foo='&#x3;'/>",        expect: '<a foo="' + String.fromCharCode(3) + '"/>' },
        { xml: "<a foo='&#x33;'/>",        expect: '<a foo="3"/>' },
        { xml: "<a foo='&#33;ar&;'/>",        expect: '<a foo="!ar&amp;;"/>' },
        { xml: "<a foo='&#3;ar&;'/>",        expect: '<a foo="' + String.fromCharCode(3) + 'ar&amp;;"/>' },
        { xml: "<a foo='&#x3;ar&;'/>",        expect: '<a foo="' + String.fromCharCode(3) + 'ar&amp;;"/>' },
        { xml: "<a foo='&#x33;ar&;'/>",        expect: '<a foo="3ar&amp;;"/>' },
    ]

    for (var i = 0; i < tests.length; ++i)
    {
        var x:XML = new XML(tests[i].xml);
        array[item++] = Assert.expectEq( tests[i].xml, tests[i].expect, x.toXMLString());
    }

    return array;
}
