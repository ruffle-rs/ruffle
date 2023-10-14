/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = 'As described in Netscape doc "Whats new in JavaScript 1.2"';
//     var VERSION = 'no version';
//     var TITLE = 'String:slice';


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    function mySlice(a, from, to)
    {
        var from2       = from;
        var to2         = to;
        var returnArray = [];
        var i;

        if (from2 < 0) from2 = a.length + from;
        if (to2 < 0)   to2   = a.length + to;

        if ((to2 > from2)&&(to2 > 0)&&(from2 < a.length))
        {
            if (from2 < 0)        from2 = 0;
            if (to2 > a.length) to2 = a.length;

            for (i = from2; i < to2; ++i) returnArray.push(a[i]);
        }
        return returnArray;
    }

    // This function tests the slice command on an Array
    // passed in. The arguments passed into slice range in
    // value from -5 to the length of the array + 4. Every
    // combination of the two arguments is tested. The expected
    // result of the slice(...) method is calculated and
    // compared to the actual result from the slice(...) method.
    // If the Arrays are not similar false is returned.
    function exhaustiveSliceTest(testname, a)
    {
        var x = 0;
        var y = 0;
        var errorMessage;
        var reason = "";
        var passed = true;

        for (x = -(2 + a.length); x <= (2 + a.length); x++)
            for (y = (2 + a.length); y >= -(2 + a.length); y--)
            {
                var b  = a.slice(x,y);
                var c = mySlice(a,x,y);

                if (String(b) != String(c))
                {
                    errorMessage =
                        "ERROR: 'TEST FAILED' ERROR: 'TEST FAILED' ERROR: 'TEST FAILED'\n" +
                        "            test: " + "a.slice(" + x + "," + y + ")\n" +
                        "               a: " + String(a) + "\n" +
                        "   actual result: " + String(b) + "\n" +
                        " expected result: " + String(c) + "\n";
                    reason = reason + errorMessage;
                    passed = false;
                }
            }
        var testCase = Assert.expectEq( testname, true, passed);
        if (passed == false)
            testCase.reason = reason;
        return testCase;
    }

    var a = ['a','test string',456,9.34,new String("string object"),[],['h','i','j','k']];
    var b = [1,2,3,4,5,6,7,8,9,0];

    array[item++] = exhaustiveSliceTest("exhaustive slice test 1", a);
    array[item++] = exhaustiveSliceTest("exhaustive slice test 2", b);

    return ( array );
}
