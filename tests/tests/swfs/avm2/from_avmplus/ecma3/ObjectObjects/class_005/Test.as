/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 14 Mar 2001
 *
 * SUMMARY: Testing the internal [[Class]] property of user-defined types.
 * See ECMA-262 Edition 3 13-Oct-1999, Section 8.6.2 re [[Class]] property.
 *
 * Same as class-001.js - but testing user-defined types here, not native types.
 * Therefore we expect the [[Class]] property to equal 'Object' in each case -
 *
 * The getJSClass() function we use is in a utility file, e.g. "shell.js"
 */
//-------------------------------------------------------------------------------------------------
// var SECTION = "class_005";
// var VERSION = "";
// var TITLE   = "Testing the internal [[Class]] property of user-defined types";
// var bug = "(none)";

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

    var status = '';
    var actual = '';
    var expect= '';

    Calf.prototype= new Cow();

    /*
     * We set the expect variable each time only for readability.
     * We expect 'Object' every time; see discussion above -
     */
    status = 'new Cow()';
    actual = getJSClass(new Cow());
    expect = 'Object';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new Calf()';
    actual = getJSClass(new Calf());
    expect = 'Object';
    array[item++] = Assert.expectEq( status, expect, actual);


    function Cow(name)
    {
      this.name=name;
    }


    function Calf(name)
    {
      this.name=name;
    }

    return array;
}
