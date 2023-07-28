/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
*
* Date:    22 Jan 2002
* SUMMARY: Testing Error.prototype.toString()
*
* Note that ECMA-262 3rd Edition Final, Section 15.11.4.4 states that
* Error.prototype.toString() returns an implementation-dependent string.
* Therefore any testcase on this property is somewhat arbitrary.
*
* However, d-russo@ti.com pointed out that Rhino was returning this:
*
*               js> err = new Error()
*               undefined: undefined
*
*               js> err = new Error("msg")
*               undefined: msg
*
*
* We expect Rhino to return what SpiderMonkey currently does:
*
*               js> err = new Error()
*               Error
*
*               js> err = new Error("msg")
*               Error: msg
*
*
* i.e. we expect err.toString() === err.name if err.message is not defined;
* otherwise, we expect err.toString() === err.name + ': ' + err.message.
*
* See also ECMA 15.11.4.2, 15.11.4.3
*/
//-----------------------------------------------------------------------------

package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "e15_11_4_4_1";
var UBound = 0;
// var bug = '(none)';
var summary = 'Testing Error.prototype.toString()';
var status = '';
var statusitems = [];
var actual = '';
var actualvalues = [];
var expect= '';
var expectedvalues = [];
var EMPTY_STRING = '';
var EXPECTED_FORMAT = 0;

var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    status = "new Error('msg1')";
    var err1 = new Error('msg1');
    actual = examineThis(err1);
    expect = EXPECTED_FORMAT;
    array[item++] = Assert.expectEq( status, expect, actual);
    array[item++] = Assert.expectEq( "new Error('msg1')", "Error: msg1", (new Error('msg1')).toString() );

    status = "new Error(err1)";
    var err2 = new Error(err1);
    actual = examineThis(err2);
    expect = EXPECTED_FORMAT;
    array[item++] = Assert.expectEq( status, expect, actual);
    array[item++] = Assert.expectEq( "new Error(err1)", "Error: Error: msg1", (new Error(err1)).toString() );

    status = "new Error()";
    var err3 = new Error();
    actual = examineThis(err3);
    expect = EXPECTED_FORMAT;
    array[item++] = Assert.expectEq( status, expect, actual);
    array[item++] = Assert.expectEq(  "new Error()", "Error", (new Error()).toString() );

    status = "new Error('')";
    var err4 = new Error(EMPTY_STRING);
    actual = examineThis(err4);
    expect = EXPECTED_FORMAT;
    array[item++] = Assert.expectEq( status, expect, actual);
    array[item++] = Assert.expectEq( "new Error('')", "Error", (new Error('')).toString() );

    // now generate a run-time error -
    status = "run-time error";
    try
    {
        throw new Error();
    }
    catch(err5)
    {
     actual = examineThis(err5);
    }
    expect = EXPECTED_FORMAT;
    array[item++] = Assert.expectEq( status, expect, actual);
    
    return array;
}


/*
 * Searches err.toString() for err.name + ':' + err.message,
 * with possible whitespace on each side of the colon sign.
 *
 * We allow for no colon in case err.message was not provided by the user.
 * In such a case, SpiderMonkey and Rhino currently set err.message = '',
 * as allowed for by ECMA 15.11.4.3. This makes |pattern| work in this case.
 *
 * If this is ever changed to a non-empty string, e.g. 'undefined',
 * you may have to modify |pattern| to take that into account -
 *
 */
function examineThis(err)
{
  var pattern = err.name + '\\s*:?\\s*' + err.message;
  return err.toString().search(RegExp(pattern));
}


