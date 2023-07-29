/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
*
* Date:    09 September 2002
* SUMMARY: Test for TypeError on invalid default string value of object
* See ECMA reference at http://bugzilla.mozilla.org/show_bug.cgi?id=167325
*
*/
//-----------------------------------------------------------------------------
// var SECTION = "e8_6_2_6_001";
// var VERSION = "";
// var TITLE   = "Test for TypeError on invalid default string value of object";
// var bug = "167325";
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

var testcases = getTestCases();

   // TODO: REVIEW AS4 CONVERSION ISSUE 
 // Adding this function getJSClass directly to file rather than in Utils

 function getJSClass(obj)
{
  if (isObject(obj))
    return findClass(findType(obj));
  return cnNoObject;
}
function isObject(obj)
{
  return obj instanceof Object;
}

function findType(obj)
{
  var cnObjectToString = Object.prototype.toString;
  return cnObjectToString.apply(obj);
}
// given '[object Number]',  return 'Number'
function findClass(sType)
{
  var re =  /^\[.*\s+(\w+)\s*\]$/;
  var a = sType.match(re);

  if (a && a[1])
    return a[1];
  return cnNoClass;
}

function getTestCases() {
    var array = new Array();
    var item = 0;

    var TEST_PASSED = 'TypeError';
    var TEST_FAILED = 'Generated an error, but NOT a TypeError!';
    var TEST_FAILED_BADLY = 'Did not generate ANY error!!!';
    var status = '';
    var actual = '';
    var expect= '';


    //status = inSection(1);
    expect = TEST_PASSED;
    actual = TEST_FAILED_BADLY;
    /*
     * This should generate a TypeError. See ECMA reference
     * at http://bugzilla.mozilla.org/show_bug.cgi?id=167325
     */
    try
    {
      var obj = {toString: function() {return new Object();}}
      obj == 'abc';
    }
    catch(e)
    {
      if (e instanceof TypeError)
        actual = TEST_PASSED;
      else
        actual = TEST_FAILED;
    }
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}
