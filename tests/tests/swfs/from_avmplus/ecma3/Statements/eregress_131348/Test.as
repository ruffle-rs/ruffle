/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
*
* Date:    10 Apr 2002
* Revised: 14 July 2002
*
* SUMMARY: JS should NOT error on |for(i in undefined)|, |for(i in null)|
*
* ECMA-262 3rd Edition Final spec says such statements SHOULD error. See:
*
*               Section 12.6.4  The for-in Statement
*               Section 9.9     ToObject
*
*
* But SpiderMonkey has decided NOT to follow this; it's a bug in the spec.
* See http://bugzilla.mozilla.org/show_bug.cgi?id=131348
*
* Update: Rhino has also decided not to follow the spec on this
* See http://bugzilla.mozilla.org/show_bug.cgi?id=136893
*/
//-----------------------------------------------------------------------------
//     var SECTION = "eregress_131348";
//     var VERSION = "";
//     var TITLE   = "JS should not error on |for(i in undefined)|, |for(i in null)|";
// var bug = 131348;


//TO-DO: adding mission import
import com.adobe.test.Assert;

    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        

var UBound = 0;
// var bug = 131348;
var summary = 'JS should not error on |for(i in undefined)|, |for(i in null)|';
var TEST_PASSED = 'No error';
var TEST_FAILED = 'An error was generated!!!';
var status = '';
var statusitems = [];
var actual = '';
var actualvalues = [];
var expect= '';
var expectedvalues = [];


//TO-DO: replacing inSection
//status = inSection(1);
status = "statements string1";
expect = TEST_PASSED;
actual = TEST_PASSED;
try
{
  for (var i in undefined)
  {
    trace(i);
  }
}
catch(e)
{
  actual = TEST_FAILED;
}
//addThis();
array[item++] = Assert.expectEq( status, expect, actual);



//TO-DO: replacing inSection
//status = inSection(1);
status = "statements string2";
expect = TEST_PASSED;
actual = TEST_PASSED;
try
{
  for (var i in null)
  {
    trace(i);
  }
}
catch(e)
{
  actual = TEST_FAILED;
}
//addThis();
array[item++] = Assert.expectEq( status, expect, actual);



//TO-DO: replacing inSection
//status = inSection(1);
status = "statements string3";
expect = TEST_PASSED;
actual = TEST_PASSED;
/*
 * Variable names that cannot be looked up generate ReferenceErrors; however,
 * property names like obj.ZZZ that cannot be looked up are set to |undefined|
 *
 * Therefore, this should indirectly test | for (var i in undefined) |
 */
try
{
  for (var i in this.ZZZ)
  {
    trace(i);
  }
}
catch(e)
{
  actual = TEST_FAILED;
}
//addThis();
array[item++] = Assert.expectEq( status, expect, actual);



//TO-DO: replacing inSection
//status = inSection(1);
status = "statements string4";
expect = TEST_PASSED;
actual = TEST_PASSED;
/*
 * The result of an unsuccessful regexp match is the null value
 * Therefore, this should indirectly test | for (var i in null) |
 */
try
{
  for (var i in 'bbb'.match(/aaa/))
  {
    trace(i);
  }
}
catch(e)
{
  actual = TEST_FAILED;
}
//addThis();
array[item++] = Assert.expectEq( status, expect, actual);


    return array;
}

/*
function addThis()
{
  statusitems[UBound] = status;
  actualvalues[UBound] = actual;
  expectedvalues[UBound] = expect;
  UBound++;
}
*/

/*

function test()
{
  enterFunc('test');
  printBugNumber(bug);
  printStatus(summary);

  for (var i=0; i<UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
  }

  exitFunc ('test');
}
*/
