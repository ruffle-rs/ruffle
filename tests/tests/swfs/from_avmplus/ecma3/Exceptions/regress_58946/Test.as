/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *This test arose from Bugzilla bug 58946.
 *The bug was filed when we got the following error (see code below):
 *
 *                          "ReferenceError: e is not defined"
 *
 *There was no error if we replaced "return e" in the code below with "print(e)".
 *There should be no error with "return e", either  -
 */
//-------------------------------------------------------------------------------------------------
var THE_ERROR = "return_error";
// var SECTION = "exception regression";
// var VERSION = "ECMA_3";


var stat = "Testing a return statement inside a catch block inside a function";

    // TODO: REVIEW AS4 CONVERSION ISSUE 
//printStatus (stat);
var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;

    var thisError = throwError();
    
    array[item++] = Assert.expectEq( "throwError()", THE_ERROR, thisError);
    
    return array;
}

function throwError()
{
    try
    {
        throw THE_ERROR;
    }
    catch(e)
    {
        return e;
    }
}
