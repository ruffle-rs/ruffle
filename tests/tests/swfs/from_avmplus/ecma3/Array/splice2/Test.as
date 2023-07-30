/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = 'As described in Netscape doc "Whats new in JavaScript 1.2"';
//     var VERSION = 'no version';
//     var TITLE = 'String:splice 2';
    var BUGNUMBER="123795";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    function mySplice(testArray, splicedArray, first, len, elements)
    {
        var removedArray  = [];
        var adjustedFirst = first;
        var adjustedLen   = len;

        if (adjustedFirst < 0) adjustedFirst = testArray.length + first;
        if (adjustedFirst < 0) adjustedFirst = 0;

        if (adjustedLen < 0) adjustedLen = 0;

        for (var i = 0; (i < adjustedFirst)&&(i < testArray.length); ++i)
            splicedArray.push(testArray[i]);

        if (adjustedFirst < testArray.length)
            for (i = adjustedFirst; (i < adjustedFirst + adjustedLen) && (i < testArray.length); ++i)
                removedArray.push(testArray[i]);

        for (var i = 0; i < elements.length; i++) splicedArray.push(elements[i]);

        for (var i = adjustedFirst + adjustedLen; i < testArray.length; i++)
            splicedArray.push(testArray[i]);

        return removedArray;
    }

    function exhaustiveSpliceTestWithArgs(testname, testArray)
    {
        var passed = true;
        var errorMessage;
        var reason = "";
        for (var first = -(testArray.length+2); first <= 2 + testArray.length; first++)
        {
            var actualSpliced   = [];
            var expectedSpliced = [];
            var actualRemoved   = [];
            var expectedRemoved = [];

            for (var len = 0; len < testArray.length + 2; len++)
            {
                actualSpliced   = [];
                expectedSpliced = [];

                for (var i = 0; i < testArray.length; ++i)
                    actualSpliced.push(testArray[i]);

                actualRemoved   = actualSpliced.splice(first,len,-97,new String("test arg"),[],9.8);
                expectedRemoved = mySplice(testArray,expectedSpliced,first,len,[-97,new String("test arg"),[],9.8]);

                var adjustedFirst = first;
                if (adjustedFirst < 0) adjustedFirst = testArray.length + first;
                if (adjustedFirst < 0) adjustedFirst = 0;


                if (  (String(actualSpliced) != String(expectedSpliced))
                    ||(String(actualRemoved) != String(expectedRemoved)))
                {
                    if (  (String(actualSpliced) == String(expectedSpliced))
                        &&(String(actualRemoved) != String(expectedRemoved)) )
                        {

                        if ( (expectedRemoved.length == 1)
                            &&(String(actualRemoved) == String(expectedRemoved[0]))) continue;
                        if ( expectedRemoved.length == 0  && actualRemoved == void 0 ) continue;
                        }

                    errorMessage =
                        "ERROR: 'TEST FAILED' ERROR: 'TEST FAILED' ERROR: 'TEST FAILED'\n" +
                        "             test: " + "a.splice(" + first + "," + len + ",-97,new String('test arg'),[],9.8)\n" +
                        "                a: " + String(testArray) + "\n" +
                        "   actual spliced: " + String(actualSpliced) + "\n" +
                        " expected spliced: " + String(expectedSpliced) + "\n" +
                        "   actual removed: " + String(actualRemoved) + "\n" +
                        " expected removed: " + String(expectedRemoved);
                    reason = reason + errorMessage;
                    passed = false;
                }
            }
        }
        var testcase = Assert.expectEq( testname, true, passed);
        if (!passed) testcase.reason = reason;
        return testcase;
    }


    var a = ['a','test string',456,9.34,new String("string object"),[],['h','i','j','k']];
    var b = [1,2,3,4,5,6,7,8,9,0];

    array[item++] = exhaustiveSpliceTestWithArgs("exhaustive splice w/2 optional args 1",a);
    array[item++] = exhaustiveSpliceTestWithArgs("exhaustive splice w/2 optional args 2",b);

    return ( array );
}
